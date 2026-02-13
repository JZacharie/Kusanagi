use aws_config::BehaviorVersion;
use aws_sdk_s3::Client as S3Client;
use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use serde_json::{json, Value};

const CACHE_KEY: &str = "news/cache.json";
const CACHE_DAYS: i64 = 7;

pub async fn get_news() -> Result<Value, String> {
    // Try to get from S3 cache first
    if let Ok(cached) = get_cached_news().await {
        return Ok(cached);
    }

    // Cache miss or expired, fetch fresh news
    fetch_fresh_news().await
}

pub async fn force_refresh() -> Result<Value, String> {
    fetch_fresh_news().await
}

async fn fetch_fresh_news() -> Result<Value, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let mut all_news = Vec::new();

    // Fetch from all sources concurrently (3 batches to avoid overwhelming)
    // Batch 1: Original sources
    let (hn_news, korben_news, github_news, cncf_news) = tokio::join!(
        fetch_hackernews(&client),
        fetch_korben(&client),
        fetch_github_trending(&client),
        fetch_cncf(&client)
    );

    // Batch 2: Cloud providers
    let (aws_news, aws_new_news, gcp_news, azure_news) = tokio::join!(
        fetch_aws_blog(&client),
        fetch_aws_new(&client),
        fetch_gcp(&client),
        fetch_azure(&client)
    );

    // Batch 3: Kubernetes & Rust
    let (k8s_news, fluxcd_news, rust_news, inside_rust_news, twir_news) = tokio::join!(
        fetch_kubernetes(&client),
        fetch_fluxcd(&client),
        fetch_rust_blog(&client),
        fetch_inside_rust(&client),
        fetch_this_week_in_rust(&client)
    );

    // Aggregate all results
    for mut items in [
        hn_news,
        korben_news,
        github_news,
        cncf_news,
        aws_news,
        aws_new_news,
        gcp_news,
        azure_news,
        k8s_news,
        fluxcd_news,
        rust_news,
        inside_rust_news,
        twir_news,
    ]
    .into_iter()
    .flatten()
    {
        all_news.append(&mut items);
    }

    // Filter news older than CACHE_DAYS (7 days)
    let now = Utc::now();
    all_news.retain(|item| {
        if let Some(published_at_str) = item["published_at"].as_str() {
            if let Ok(published_at) = DateTime::parse_from_rfc3339(published_at_str) {
                let age = now.signed_duration_since(published_at.with_timezone(&Utc));
                return age < Duration::days(CACHE_DAYS);
            }
        }
        // Keep items without date or with invalid date to avoid empty feed,
        // but ideally everything should have a date.
        // For now, let's keep them and assume they are recent enough if fetch succeeded.
        // Actually, let's be strict if we can, but fallback to keeping if parsing fails?
        // Let's keep them if we can't parse, just in case.
        true
    });

    // Sort by date descending
    all_news.sort_by(|a, b| {
        let date_a = a["published_at"].as_str().unwrap_or("");
        let date_b = b["published_at"].as_str().unwrap_or("");
        date_b.cmp(date_a)
    });

    let response = if all_news.is_empty() {
        json!({
            "items": get_fallback_news(),
            "cached_at": Utc::now().to_rfc3339(),
            "source": "fallback"
        })
    } else {
        // Collect unique sources from items for dynamic frontend
        let mut sources: Vec<String> = all_news
            .iter()
            .filter_map(|item| item["source"].as_str().map(|s| s.to_string()))
            .collect();
        sources.sort();
        sources.dedup();

        json!({
            "items": all_news,
            "cached_at": Utc::now().to_rfc3339(),
            "sources": sources
        })
    };

    // Store in S3 cache (fire and forget)
    let response_clone = response.clone();
    tokio::spawn(async move {
        if let Err(e) = store_cached_news(response_clone).await {
            tracing::error!("Failed to update news cache in S3: {}", e);
        }
    });

    Ok(response)
}

async fn get_cached_news() -> Result<Value, String> {
    let s3_client = create_s3_client().await?;
    let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "kusanagi".to_string());

    let result = s3_client
        .get_object()
        .bucket(&bucket)
        .key(CACHE_KEY)
        .send()
        .await
        .map_err(|e| {
            tracing::warn!("S3 get error: {}", e);
            format!("S3 get error: {}", e)
        })?;

    let body = result.body.collect().await.map_err(|e| {
        tracing::debug!("S3 get_object failed: {:?}", e);
        format!("S3 get error: {:?}", e)
    })?;

    let cached: Value = serde_json::from_slice(&body.into_bytes())
        .map_err(|e| format!("JSON parse error: {}", e))?;

    // Check if cache is still valid (1 hour)
    if let Some(cached_at_str) = cached["cached_at"].as_str() {
        if let Ok(cached_at) = DateTime::parse_from_rfc3339(cached_at_str) {
            let age = Utc::now().signed_duration_since(cached_at.with_timezone(&Utc));
            if age < Duration::hours(1) {
                return Ok(cached);
            }
        }
    }

    Err("Cache expired".to_string())
}

async fn store_cached_news(data: Value) -> Result<(), String> {
    let s3_client = create_s3_client().await?;
    let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "kusanagi".to_string());

    let json_bytes =
        serde_json::to_vec(&data).map_err(|e| format!("JSON serialize error: {}", e))?;

    s3_client
        .put_object()
        .bucket(&bucket)
        .key(CACHE_KEY)
        .body(json_bytes.into())
        .content_type("application/json")
        .send()
        .await
        .map_err(|e| {
            tracing::error!("S3 put_object failed: {:?}", e);
            format!("S3 put error: {:?}", e)
        })?;

    Ok(())
}

async fn create_s3_client() -> Result<S3Client, String> {
    let endpoint =
        std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://192.168.0.170:9010".to_string());
    let region = std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let access_key = std::env::var("S3_ACCESS_KEY")
        .map_err(|_| "S3_ACCESS_KEY environment variable not set".to_string())?;
    let secret_key = std::env::var("S3_SECRET_KEY")
        .map_err(|_| "S3_SECRET_KEY environment variable not set".to_string())?;

    let credentials = aws_sdk_s3::config::Credentials::new(
        access_key,
        secret_key,
        None, // session token
        None, // expiry
        "custom",
    );

    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(aws_sdk_s3::config::Region::new(region))
        .endpoint_url(endpoint)
        .load()
        .await;

    Ok(S3Client::from_conf(
        aws_sdk_s3::config::Builder::from(&config)
            .credentials_provider(credentials)
            .force_path_style(true)
            .build(),
    ))
}

async fn fetch_hackernews(client: &Client) -> Result<Vec<Value>, String> {
    let response = client
        .get("https://hacker-news.firebaseio.com/v0/topstories.json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let story_ids = response
        .json::<Vec<u64>>()
        .await
        .map_err(|e| e.to_string())?;

    let mut news_items = Vec::new();

    // Take top 50
    for &story_id in story_ids.iter().take(50) {
        if let Ok(story_res) = client
            .get(format!(
                "https://hacker-news.firebaseio.com/v0/item/{}.json",
                story_id
            ))
            .send()
            .await
        {
            if let Ok(story) = story_res.json::<Value>().await {
                // Convert unix timestamp to RFC3339
                let time = story["time"].as_i64().unwrap_or(0);
                let published_at = if let Some(dt) = DateTime::from_timestamp(time, 0) {
                    dt.to_rfc3339()
                } else {
                    Utc::now().to_rfc3339()
                };

                news_items.push(json!({
                    "title": story["title"],
                    "url": story["url"],
                    "score": story["score"],
                    "published_at": published_at,
                    "source": "hackernews",
                    "icon": "🟠"
                }));
            }
        }
    }

    Ok(news_items)
}

async fn fetch_korben(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://korben.info/feed", "korben", "🔵").await
}

async fn fetch_github_trending(client: &Client) -> Result<Vec<Value>, String> {
    // GitHub trending doesn't have an official RSS, using a community service
    fetch_rss_feed(
        client,
        "https://mshibanami.github.io/GitHubTrendingRSS/daily/all.xml",
        "github",
        "🟣",
    )
    .await
}

async fn fetch_cncf(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://www.cncf.io/feed/", "cncf", "📰").await
}

// Cloud Providers
async fn fetch_aws_blog(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(
        client,
        "https://aws.amazon.com/blogs/aws/feed/",
        "aws",
        "☁️",
    )
    .await
}

async fn fetch_aws_new(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://aws.amazon.com/new/feed/", "aws-new", "🆕").await
}

async fn fetch_gcp(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(
        client,
        "https://blog.google/products/google-cloud/rss/",
        "gcp",
        "☁️",
    )
    .await
}

async fn fetch_azure(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(
        client,
        "https://azurecomcdn.azureedge.net/en-us/blog/feed/",
        "azure",
        "☁️",
    )
    .await
}

// Kubernetes & Cloud Native
async fn fetch_kubernetes(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://kubernetes.io/feed.xml", "kubernetes", "☸️").await
}

async fn fetch_fluxcd(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://fluxcd.io/blog/index.xml", "fluxcd", "🔄").await
}

// Rust Programming
async fn fetch_rust_blog(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://blog.rust-lang.org/feed.xml", "rust", "🦀").await
}

async fn fetch_inside_rust(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(
        client,
        "https://blog.rust-lang.org/inside-rust/feed.xml",
        "inside-rust",
        "🔧",
    )
    .await
}

async fn fetch_this_week_in_rust(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(
        client,
        "https://this-week-in-rust.org/rss.xml",
        "twir",
        "📰",
    )
    .await
}

async fn fetch_rss_feed(
    client: &Client,
    url: &str,
    source_name: &str,
    icon: &str,
) -> Result<Vec<Value>, String> {
    let response = client.get(url).send().await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let xml_content = response.text().await.map_err(|e| e.to_string())?;
    // Remove BOM if present
    let xml_content = xml_content.trim_start_matches('\u{feff}');

    let mut news_items = Vec::new();

    // Determine if RSS or Atom
    // This is a naive heuristic but works for most standard feeds
    let is_atom = xml_content.contains("<feed")
        || xml_content.contains("xmlns=\"http://www.w3.org/2005/Atom\"");
    let item_tag = if is_atom { "entry" } else { "item" };

    // Find all items
    // We scan the parsing manually to handle multi-line and minified XML
    let mut current_pos = 0;
    while let Some(start_tag_pos) = xml_content[current_pos..].find(&format!("<{}", item_tag)) {
        let absolute_start = current_pos + start_tag_pos;

        // Find the closure of the opening tag (could be <item> or <item attributes...>)
        if let Some(open_tag_end) = xml_content[absolute_start..].find('>') {
            let _content_start = absolute_start + open_tag_end + 1;

            // Find end tag
            let end_tag = format!("</{}>", item_tag);
            if let Some(end_tag_pos) = xml_content[absolute_start..].find(&end_tag) {
                let absolute_end = absolute_start + end_tag_pos;

                // Extract the full item content block
                let item_block = &xml_content[absolute_start..absolute_end];

                // Parse fields within this block
                if let Some(item) = parse_item_block(item_block, source_name, icon, is_atom) {
                    news_items.push(item);
                }

                // Move past this item
                current_pos = absolute_end + end_tag.len();
            } else {
                // Malformed or truncated, break, or try to continue?
                // If we can't find the end tag, we can't parse this item safely.
                // Just advance past the start tag to avoid infinite loop
                current_pos = absolute_start + 1;
            }
        } else {
            break;
        }

        if news_items.len() >= 50 {
            break;
        }
    }

    Ok(news_items)
}

fn parse_item_block(block: &str, source_name: &str, icon: &str, is_atom: bool) -> Option<Value> {
    let title = extract_tag_content(block, "title").map(|s| clean_html(&s))?; // Title is required

    let url = if is_atom {
        // Atom links: <link href="..." />
        extract_attr(block, "link", "href").or_else(|| extract_tag_content(block, "id"))
    } else {
        // RSS links: <link>...</link>
        extract_tag_content(block, "link")
    };

    // Attempt to extract description/summary
    let description = if is_atom {
        extract_tag_content(block, "summary").or_else(|| extract_tag_content(block, "content"))
    } else {
        extract_tag_content(block, "description")
    };

    // Clean description (strip HTML tags for safety and UI consistency)
    let clean_desc = description.map(|d| strip_tags(&clean_html(&d)));

    // Extract tags/categories
    let mut tags = Vec::new();
    // Simple scan for all <category> tags
    let mut tag_scan_pos = 0;
    while let Some(cat_pos) = block[tag_scan_pos..].find("<category") {
        let absolute_cat = tag_scan_pos + cat_pos;
        // Check if it's self closing or textual
        // Atom: <category term="foo" />
        // RSS: <category>foo</category>

        if is_atom {
            // Find end of this tag
            if let Some(tag_end) = block[absolute_cat..].find('>') {
                let tag_fragment = &block[absolute_cat..absolute_cat + tag_end + 1];
                if let Some(term) = extract_attr_from_fragment(tag_fragment, "term") {
                    if !term.trim().is_empty() {
                        tags.push(term);
                    }
                }
                tag_scan_pos = absolute_cat + tag_end;
            } else {
                tag_scan_pos = absolute_cat + 1;
            }
        } else {
            if let Some(content) = extract_tag_content(&block[absolute_cat..], "category") {
                let content = clean_html(&content);
                if !content.is_empty() {
                    tags.push(content);
                }
            }
            tag_scan_pos = absolute_cat + 9; // Skip <category
        }
    }

    // Date parsing
    let date_str = if is_atom {
        extract_tag_content(block, "published").or_else(|| extract_tag_content(block, "updated"))
    } else {
        extract_tag_content(block, "pubDate")
    };

    let published_at = if let Some(d) = date_str {
        if let Ok(dt) = DateTime::parse_from_rfc2822(&d) {
            dt.to_rfc3339()
        } else if let Ok(dt) = DateTime::parse_from_rfc3339(&d) {
            dt.to_rfc3339()
        } else {
            Utc::now().to_rfc3339()
        }
    } else {
        Utc::now().to_rfc3339()
    };

    Some(json!({
        "title": title,
        "url": url.unwrap_or_default(),
        "description": clean_desc,
        "published_at": published_at,
        "source": source_name,
        "icon": icon,
        "tags": tags
    }))
}

// Helper to extract content between <tag> and </tag>
fn extract_tag_content(xml: &str, tag_name: &str) -> Option<String> {
    let start_tag = format!("<{}", tag_name);
    // We need to handle <tag> and <tag attr="...">

    let start_pos = xml.find(&start_tag)?;

    // Find the end of the opening tag >
    let open_tag_end = xml[start_pos..].find('>')?;
    let content_start = start_pos + open_tag_end + 1;

    let end_tag = format!("</{}>", tag_name);
    let end_pos = xml[content_start..].find(&end_tag)?;

    Some(
        xml[content_start..content_start + end_pos]
            .trim()
            .to_string(),
    )
}

// Helper to extract attribute value: <tag ... attr="value" ...>
fn extract_attr(xml: &str, tag_name: &str, attr_name: &str) -> Option<String> {
    let start_tag = format!("<{}", tag_name);
    let start_pos = xml.find(&start_tag)?;

    // Find end of this tag
    let tag_end_pos = xml[start_pos..].find('>')?;
    let tag_fragment = &xml[start_pos..start_pos + tag_end_pos + 1];

    extract_attr_from_fragment(tag_fragment, attr_name)
}

fn extract_attr_from_fragment(fragment: &str, attr_name: &str) -> Option<String> {
    let attr_search = format!("{}=\"", attr_name);
    if let Some(attr_pos) = fragment.find(&attr_search) {
        let val_start = attr_pos + attr_search.len();
        if let Some(val_end) = fragment[val_start..].find('"') {
            return Some(fragment[val_start..val_start + val_end].to_string());
        }
    }
    // Try single quotes
    let attr_search_sq = format!("{}='", attr_name);
    if let Some(attr_pos) = fragment.find(&attr_search_sq) {
        let val_start = attr_pos + attr_search_sq.len();
        if let Some(val_end) = fragment[val_start..].find('\'') {
            return Some(fragment[val_start..val_start + val_end].to_string());
        }
    }
    None
}

fn clean_html(text: &str) -> String {
    // Remove CDATA wrappers
    let text = text.replace("<![CDATA[", "").replace("]]>", "");

    // Decode entities
    let text = text
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#039;", "'")
        .replace("&nbsp;", " ");

    text.trim().to_string()
}

fn strip_tags(text: &str) -> String {
    let mut result = String::new();
    let mut inside_tag = false;

    for c in text.chars() {
        if c == '<' {
            inside_tag = true;
        } else if c == '>' {
            inside_tag = false;
        } else if !inside_tag {
            result.push(c);
        }
    }
    result
}

fn get_fallback_news() -> Vec<Value> {
    let now = Utc::now().to_rfc3339();
    vec![
        json!({
            "title": "Kubernetes 1.32 Released with Enhanced Security Features",
            "url": "https://kubernetes.io/blog/",
            "source": "kubernetes",
            "icon": "☸️",
            "published_at": &now,
            "description": "The latest release of Kubernetes brings new security features and stability improvements.",
            "tags": ["kubernetes", "release", "security"]
        }),
        json!({
            "title": "CNCF Announces New Cloud Native Projects",
            "url": "https://www.cncf.io/announcements/",
            "source": "cncf",
            "icon": "📰",
            "published_at": &now,
            "description": "New projects joining the CNCF ecosystem to enhance cloud native capabilities.",
            "tags": ["cncf", "cloud-native", "announcement"]
        }),
    ]
}
