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
    for result in [
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
    ] {
        if let Ok(mut items) = result {
            all_news.append(&mut items);
        }
    }

    let response = if all_news.is_empty() {
        json!({
            "items": get_fallback_news(),
            "cached_at": Utc::now().to_rfc3339(),
            "source": "fallback"
        })
    } else {
        json!({
            "items": all_news,
            "cached_at": Utc::now().to_rfc3339(),
            "sources": [
                "hackernews", "korben", "github", "cncf",
                "aws", "gcp", "azure",
                "kubernetes", "fluxcd", "rust"
            ]
        })
    };

    // Store in S3 cache (fire and forget)
    let response_clone = response.clone();
    tokio::spawn(async move {
        let _ = store_cached_news(response_clone).await;
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
        .map_err(|e| format!("S3 get error: {}", e))?;

    let body = result
        .body
        .collect()
        .await
        .map_err(|e| format!("Body read error: {}", e))?;

    let cached: Value = serde_json::from_slice(&body.into_bytes())
        .map_err(|e| format!("JSON parse error: {}", e))?;

    // Check if cache is still valid (7 days)
    if let Some(cached_at_str) = cached["cached_at"].as_str() {
        if let Ok(cached_at) = DateTime::parse_from_rfc3339(cached_at_str) {
            let age = Utc::now().signed_duration_since(cached_at.with_timezone(&Utc));
            if age < Duration::days(CACHE_DAYS) {
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
        .map_err(|e| format!("S3 put error: {}", e))?;

    Ok(())
}

async fn create_s3_client() -> Result<S3Client, String> {
    let endpoint = std::env::var("S3_ENDPOINT").map_err(|_| "S3_ENDPOINT not set")?;
    let region = std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());

    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(aws_sdk_s3::config::Region::new(region))
        .endpoint_url(endpoint)
        .load()
        .await;

    Ok(S3Client::new(&config))
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

    // Take top 5
    for &story_id in story_ids.iter().take(5) {
        if let Ok(story_res) = client
            .get(&format!(
                "https://hacker-news.firebaseio.com/v0/item/{}.json",
                story_id
            ))
            .send()
            .await
        {
            if let Ok(story) = story_res.json::<Value>().await {
                news_items.push(json!({
                    "title": story["title"],
                    "url": story["url"],
                    "score": story["score"],
                    "time": story["time"],
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

    let mut news_items = Vec::new();
    let lines: Vec<&str> = xml_content.lines().collect();

    let mut current_item = json!({});
    let mut in_item = false;

    for line in lines {
        let line = line.trim();

        if line.contains("<item>") || line.contains("<entry>") {
            in_item = true;
            current_item = json!({});
        } else if (line.contains("</item>") || line.contains("</entry>")) && in_item {
            if !current_item["title"].is_null() {
                current_item["source"] = json!(source_name);
                current_item["icon"] = json!(icon);
                news_items.push(current_item.clone());
            }
            in_item = false;
        } else if in_item {
            if let Some(title) = extract_xml_content(line, "title") {
                current_item["title"] = json!(clean_html(&title));
            } else if let Some(link) = extract_xml_content(line, "link") {
                current_item["url"] = json!(link);
            } else if line.contains("<link") && line.contains("href=") {
                // Atom feed link format
                if let Some(href_start) = line.find("href=\"") {
                    let after_href = &line[href_start + 6..];
                    if let Some(href_end) = after_href.find('"') {
                        current_item["url"] = json!(&after_href[..href_end]);
                    }
                }
            } else if let Some(pub_date) = extract_xml_content(line, "pubDate") {
                current_item["time"] = json!(pub_date);
            } else if let Some(published) = extract_xml_content(line, "published") {
                current_item["time"] = json!(published);
            }
        }

        if news_items.len() >= 5 {
            break;
        }
    }

    Ok(news_items)
}

fn extract_xml_content(line: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);

    if let Some(start) = line.find(&start_tag) {
        if let Some(end) = line.find(&end_tag) {
            let content_start = start + start_tag.len();
            if content_start < end {
                return Some(line[content_start..end].trim().to_string());
            }
        }
    }
    None
}

fn clean_html(text: &str) -> String {
    // Remove CDATA
    let text = text.replace("<![CDATA[", "").replace("]]>", "");

    // Basic HTML entity decoding
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#039;", "'")
        .trim()
        .to_string()
}

fn get_fallback_news() -> Vec<Value> {
    vec![
        json!({
            "title": "Kubernetes 1.29 Released with Enhanced Security Features",
            "url": "https://kubernetes.io/blog/2024/12/11/kubernetes-v1-29-release/",
            "source": "kubernetes",
            "icon": "📰",
            "time": "2024-12-11"
        }),
        json!({
            "title": "Docker Desktop 4.26 Introduces New Container Management Tools",
            "url": "https://www.docker.com/blog/docker-desktop-4-26/",
            "source": "docker",
            "icon": "🐳",
            "time": "2024-12-10"
        }),
    ]
}
