use aws_sdk_s3::Client as S3Client;
use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use serde_json::{json, Value};

use crate::domain::entities::llm::{LlmConfig, LlmProvider};
use crate::domain::services::llm_service::LlmService;
use futures::stream::{FuturesUnordered, StreamExt};
use std::sync::Arc;
use tokio::sync::Semaphore;

const CACHE_PREFIX: &str = "news/";
const CACHE_DAYS: i64 = 7;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub async fn get_news() -> Result<Value, String> {
    tracing::debug!("📰 News service: Attempting to get news from daily caches");

    match get_aggregated_news().await {
        Ok(news) if !news["items"].as_array().map(|a| a.is_empty()).unwrap_or(true) => {
            tracing::info!("✅ News cache aggregate HIT - returning consolidated data");
            Ok(news)
        }
        _ => {
            tracing::warn!("⚠️  News cache aggregate MISS/Empty - fetching fresh news");
            fetch_fresh_news().await
        }
    }
}

pub async fn force_refresh() -> Result<Value, String> {
    fetch_fresh_news().await
}

async fn fetch_fresh_news() -> Result<Value, String> {
    tracing::info!("🔄 Fetching fresh news from all sources...");

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Kusanagi/0.3.0 (https://github.com/JZacharie/Kusanagi)")
        .build()
        .map_err(|e| e.to_string())?;

    let mut all_news = Vec::new();

    // Fetch from all sources concurrently (Organized in batches to avoid overwhelming)
    
    // Batch 1: Tech & Security (Original + New Tech)
    let (hn_news, korben_news, github_news, cncf_news, stepsecurity_news, lemagit_news, usine_digitale_news, frandroid_news) = tokio::join!(
        fetch_hackernews(&client),
        fetch_korben(&client),
        fetch_github_trending(&client),
        fetch_cncf(&client),
        fetch_stepsecurity(&client),
        fetch_lemagit(&client),
        fetch_usine_digitale(&client),
        fetch_frandroid(&client)
    );

    // Batch 2: Tech (More) & General News
    let (journal_du_geek_news, silicon_news, next_ink_news, zdnet_news, lemonde_news, franceinfo_news, figaro_news, liberation_news) = tokio::join!(
        fetch_journal_du_geek(&client),
        fetch_silicon(&client),
        fetch_next_ink(&client),
        fetch_zdnet(&client),
        fetch_lemonde(&client),
        fetch_franceinfo(&client),
        fetch_lefigaro(&client),
        fetch_liberation(&client)
    );

    // Batch 3: Cloud providers & ONU
    let (aws_news, aws_new_news, gcp_news, azure_news, onu_news) = tokio::join!(
        fetch_aws_blog(&client),
        fetch_aws_new(&client),
        fetch_gcp(&client),
        fetch_azure(&client),
        fetch_onu(&client)
    );

    // Batch 4: Kubernetes, Rust & Lyon Local
    let (k8s_news, fluxcd_news, rust_news, inside_rust_news, twir_news, progres_news, rue89lyon_news, influx_news, lyoncapitale_news, grandlyon_news) = tokio::join!(
        fetch_kubernetes(&client),
        fetch_fluxcd(&client),
        fetch_rust_blog(&client),
        fetch_inside_rust(&client),
        fetch_this_week_in_rust(&client),
        fetch_leprogres(&client),
        fetch_rue89lyon(&client),
        fetch_linflux(&client),
        fetch_lyoncapitale(&client),
        fetch_grandlyon(&client)
    );

    // Aggregate all results
    for mut items in [
        hn_news,
        korben_news,
        github_news,
        cncf_news,
        stepsecurity_news,
        lemagit_news,
        usine_digitale_news,
        frandroid_news,
        journal_du_geek_news,
        silicon_news,
        next_ink_news,
        zdnet_news,
        lemonde_news,
        franceinfo_news,
        figaro_news,
        liberation_news,
        aws_news,
        aws_new_news,
        gcp_news,
        azure_news,
        onu_news,
        k8s_news,
        fluxcd_news,
        rust_news,
        inside_rust_news,
        twir_news,
        progres_news,
        rue89lyon_news,
        influx_news,
        lyoncapitale_news,
        grandlyon_news,
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

    // Translate news items to French and store them incrementally in S3
    let s3_client = create_s3_client().await.ok();
    let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "kusanagi-news".to_string());
    
    if let Ok(endpoint) = std::env::var("S3_ENDPOINT") {
        tracing::info!("📦 Using S3 bucket for news: '{}' (endpoint: {})", bucket, endpoint);
    } else {
        tracing::info!("📦 Using S3 bucket for news: '{}' (default endpoint)", bucket);
    }
    
    let s3_info = s3_client.map(|c| (c, bucket));
    all_news = translate_news_items(all_news, s3_info).await;

    // Aggregated news is ready, but we need to COMMIT per day
    let response = if all_news.is_empty() {
        tracing::warn!("⚠️  No news fetched - using fallback mock data");
        json!({
            "items": get_fallback_news(),
            "cached_at": Utc::now().to_rfc3339(),
            "source": "fallback"
        })
    } else {
        // Collect unique sources
        let mut sources: Vec<String> = all_news
            .iter()
            .filter_map(|item| item["source"].as_str().map(|s| s.to_string()))
            .collect();
        sources.sort();
        sources.dedup();

        tracing::info!(
            "📊 Fetched {} news items from {} sources",
            all_news.len(),
            sources.len()
        );

        json!({
            "items": all_news,
            "cached_at": Utc::now().to_rfc3339(),
            "sources": sources
        })
    };

    // Commit to daily files (deprecated, but keeping briefly to avoid breaking before everything is swapped)
    // commit_news_to_daily_files(all_news).await;

    Ok(response)
}

async fn store_individual_news_item(
    s3_client: &S3Client,
    bucket: &str,
    item: Value,
) -> Result<(), String> {
    let published_at = item["published_at"].as_str().unwrap_or("");
    let url = item["url"].as_str().unwrap_or("");

    if published_at.is_empty() || url.is_empty() {
        return Err("Missing published_at or url for news item".to_string());
    }

    let dt = DateTime::parse_from_rfc3339(published_at)
        .map_err(|e| format!("Invalid date format: {}", e))?;
    let day = dt.format("%Y-%m-%d").to_string();

    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let url_hash = hasher.finish();

    let key = format!("{}{}/{:x}.json", CACHE_PREFIX, day, url_hash);

    let data = json!({
        "item": item,
        "stored_at": Utc::now().to_rfc3339()
    });

    let json_bytes = serde_json::to_vec(&data).map_err(|e| format!("JSON serialize error: {}", e))?;

    s3_client
        .put_object()
        .bucket(bucket)
        .key(&key)
        .body(json_bytes.into())
        .content_type("application/json")
        .send()
        .await
        .map_err(|e| {
            let endpoint = std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "unknown".to_string());
            let err_details = format!("{:?}", e);
            format!("S3 put error for bucket '{}' key '{}' at {}: {}", bucket, key, endpoint, err_details)
        })?;

    Ok(())
}


async fn get_aggregated_news() -> Result<Value, String> {
    let now = Utc::now();
    let mut all_items = Vec::new();

    let s3_client = create_s3_client().await?;
    let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "kusanagi".to_string());

    for i in 0..CACHE_DAYS {
        let day = (now - Duration::days(i)).format("%Y-%m-%d").to_string();
        let prefix = format!("{}{}/", CACHE_PREFIX, day);

        let list_future = s3_client
            .list_objects_v2()
            .bucket(&bucket)
            .prefix(&prefix)
            .send();

        match tokio::time::timeout(std::time::Duration::from_secs(5), list_future).await {
            Ok(Ok(output)) => {
                let mut tasks = FuturesUnordered::new();
                /*
                for object in output.contents.unwrap_or_default() {
                    if let Some(key) = object.key {
                        let client = s3_client.clone();
                        let b = bucket.clone();
                        tasks.push(tokio::spawn(async move {
                            match tokio::time::timeout(std::time::Duration::from_secs(3), get_cached_file(&client, &b, &key)).await {
                                Ok(res) => res,
                                Err(_) => Err("S3 fetch timeout".to_string()),
                            }
                        }));
                    }
                }
                */

                while let Some(result) = tasks.next().await {
                    if let Ok(Ok(file_val)) = result {
                        if let Some(item) = file_val["item"].as_object() {
                            all_items.push(json!(item));
                        } else if let Some(items) = file_val["items"].as_array() {
                            // Support legacy daily aggregated files if they still exist
                            all_items.extend(items.iter().cloned());
                        }
                    }
                }
            }
            Ok(Err(e)) => {
                tracing::warn!("⚠️  S3 list error: {}", e);
            }
            Err(_) => {
                tracing::warn!("⚠️  S3 list timeout for prefix {}", prefix);
            }
        }
    }

    // Sort by date descending
    all_items.sort_by(|a, b| {
        let date_a = a["published_at"].as_str().unwrap_or("");
        let date_b = b["published_at"].as_str().unwrap_or("");
        date_b.cmp(date_a)
    });

    let mut sources: Vec<String> = all_items
        .iter()
        .filter_map(|item| item["source"].as_str().map(|s| s.to_string()))
        .collect();
    sources.sort();
    sources.dedup();

    Ok(json!({
        "items": all_items,
        "cached_at": Utc::now().to_rfc3339(),
        "sources": sources
    }))
}

async fn get_cached_file(s3_client: &S3Client, bucket: &str, key: &str) -> Result<Value, String> {
    let result = s3_client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| {
            let endpoint = std::env::var("S3_ENDPOINT").unwrap_or_default();
            let err_msg = format!("S3 get error for {} at {}: {}", key, endpoint, e);
            tracing::error!("❌ {}", err_msg);
            err_msg
        })?;

    let body = result.body.collect().await.map_err(|e| format!("S3 body error: {}", e))?;
    let val: Value = serde_json::from_slice(&body.into_bytes()).map_err(|e| format!("JSON parse error: {}", e))?;
    Ok(val)
}


// get_cached_news and store_cached_news removed in favor of daily file helpers

async fn create_s3_client() -> Result<S3Client, String> {
    let endpoint =
        std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://192.168.0.170:9010".to_string());
    let region = std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    
    tracing::debug!("🛠️  Creating S3 client: endpoint={}, region={}", endpoint, region);

    let access_key = std::env::var("S3_ACCESS_KEY")
        .map_err(|_| "S3_ACCESS_KEY environment variable not set".to_string())?;
    let secret_key = std::env::var("S3_SECRET_KEY")
        .map_err(|_| "S3_SECRET_KEY environment variable not set".to_string())?;

    let credentials = aws_sdk_s3::config::Credentials::new(
        access_key, secret_key, None, // session token
        None, // expiry
        "custom",
    );

    let s3_config = aws_sdk_s3::config::Builder::new()
        .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
        .region(aws_sdk_s3::config::Region::new(region.clone()))
        .endpoint_url(&endpoint)
        .credentials_provider(credentials)
        .force_path_style(true)
        .build();

    tracing::info!("✅ S3 Client initialized: endpoint={}, region={}, bucket={}", endpoint, region, std::env::var("S3_BUCKET").unwrap_or_else(|_| "kusanagi-news".to_string()));

    Ok(S3Client::from_conf(s3_config))
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

async fn fetch_stepsecurity(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(
        client,
        "https://www.stepsecurity.io/blog/rss.xml",
        "stepsecurity",
        "🛡️",
    )
    .await
}

// --- General News ---

async fn fetch_lemonde(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://www.lemonde.fr/rss/une.xml", "lemonde", "🗞️").await
}

async fn fetch_franceinfo(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://www.francetvinfo.fr/titres.rss", "franceinfo", "📻").await
}

async fn fetch_lefigaro(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://www.lefigaro.fr/rss/figaro_actualites.xml", "lefigaro", "🗞️").await
}

async fn fetch_liberation(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://www.liberation.fr/arc/outboundfeeds/rss-all/category/politique/?outputType=xml", "liberation", "🗳️").await
}

async fn fetch_onu(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://news.un.org/fr/rss-feeds", "onu", "🇺🇳").await
}

// --- IT & Technology ---

async fn fetch_lemagit(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://www.lemagit.fr/rss", "lemagit", "💻").await
}

async fn fetch_usine_digitale(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://www.usine-digitale.fr/rss", "usine-digitale", "🤖").await
}

async fn fetch_frandroid(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://www.frandroid.com/feed", "frandroid", "📱").await
}

async fn fetch_journal_du_geek(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://www.journaldugeek.com/feed", "journaldugeek", "🎮").await
}

async fn fetch_silicon(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://www.silicon.fr/feed", "silicon", "💾").await
}

async fn fetch_next_ink(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://next.ink/feed/", "next-ink", "🖋️").await
}

async fn fetch_zdnet(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://www.zdnet.com/rssfeeds/", "zdnet", "📡").await
}

// --- Lyon Local News ---

async fn fetch_leprogres(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://www.leprogres.fr/rss", "leprogres", "🦁").await
}

async fn fetch_rue89lyon(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://www.rue89lyon.fr/feed/", "rue89lyon", "🛣️").await
}

async fn fetch_linflux(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "http://www.linflux.com/category/lyon-et-region/feed/", "linflux", "📚").await
}

async fn fetch_lyoncapitale(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://www.lyoncapitale.fr/rss", "lyoncapitale", "🏙️").await
}

async fn fetch_grandlyon(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://www.grandlyon.com/flux-rss", "grandlyon", "🏢").await
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
    let response = client.get(url).send().await.map_err(|e| {
        tracing::error!("❌ Failed to fetch RSS feed from {}: {}", url, e);
        e.to_string()
    })?;

    if !response.status().is_success() {
        tracing::error!("❌ RSS feed {} returned status {}", url, response.status());
        return Err(format!("HTTP {}", response.status()));
    }

    let xml_content = response.text().await.map_err(|e| {
        tracing::error!("❌ Failed to get text from {}: {}", url, e);
        e.to_string()
    })?;
    tracing::debug!("📰 Fetched {} bytes from {}", xml_content.len(), url);
    // Remove BOM if present
    let xml_content = xml_content.trim_start_matches('\u{feff}');

    let mut news_items = Vec::new();

    // Determine if RSS or Atom
    // Use a more robust check: Atom feeds must have <feed as the root tag or default namespace
    let is_atom = xml_content.contains("<feed") && !xml_content.contains("<rss");
    let item_tag = if is_atom { "entry" } else { "item" };
    tracing::debug!("📰 Parsing {} as {} (is_atom: {})", url, item_tag, is_atom);

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
                } else {
                    tracing::warn!("⚠️ Failed to parse item block from {}", source_name);
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

    tracing::info!(
        "✅ Found {} items from source {}",
        news_items.len(),
        source_name
    );
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

async fn translate_news_items(items: Vec<Value>, s3_info: Option<(S3Client, String)>) -> Vec<Value> {
    tracing::info!("🌐 Translating {} news items to French via LiteLLM...", items.len());

    let config = LlmConfig {
        provider: LlmProvider::Litellm,
        base_url: std::env::var("NEWS_LLM_URL")
            .or_else(|_| std::env::var("LLM_BASE_URL"))
            .unwrap_or_else(|_| "http://ip.zacharie.org:4000".to_string()),
        api_key: std::env::var("NEWS_LLM_API_KEY")
            .ok()
            .or_else(|| std::env::var("LLM_API_KEY").ok())
            .or_else(|| Some("sk-_RvgpIOa1V3lXLs3Ok3Rxw".to_string())),
        model: std::env::var("NEWS_LLM_MODEL")
            .or_else(|_| std::env::var("LLM_MODEL"))
            .unwrap_or_else(|_| "gpt-oss-120b".to_string()),
        temperature: 0.3,
        max_tokens: 2000,
        ..Default::default()
    };

    let debug_mode = std::env::var("NEWS_LLM_DEBUG").unwrap_or_default() == "true";
    let concurrency = std::env::var("NEWS_LLM_CONCURRENCY")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(3);

    tracing::info!("🚀 Translation config: model={}, concurrency={}, debug={}", config.model, concurrency, debug_mode);

    let llm_service = Arc::new(LlmService::with_config(config));
    let semaphore = Arc::new(Semaphore::new(concurrency)); // Max concurrent requests
    let s3_info = s3_info.map(|(c, b)| (Arc::new(c), Arc::new(b)));

    let mut tasks = FuturesUnordered::new();

    for mut item in items.into_iter() {
        let llm_service = Arc::clone(&llm_service);
        let semaphore = Arc::clone(&semaphore);
        let s3_info = s3_info.clone();

        tasks.push(tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            let title = item["title"].as_str().unwrap_or("").to_string();
            let description = item["description"].as_str().unwrap_or("").to_string();

            if !title.is_empty() {
                let prompt = format!("Translate the following news title to French. Return ONLY the translated text, no comments, no quotes:\n\n{}", title);
                if debug_mode {
                    tracing::info!("📝 [LLM DEBUG] Prompt Title: {}", prompt);
                }
                match llm_service.complete(&prompt).await {
                    Ok(translated_title) => {
                        if debug_mode {
                            tracing::info!("✨ [LLM DEBUG] Translated Title: {}", translated_title);
                        }
                        item["title"] = json!(translated_title);
                    }
                    Err(e) => {
                        tracing::error!("❌ [LLM ERROR] Title translation failed: {}", e);
                    }
                }
            }

            if !description.is_empty() {
                let prompt = format!("Translate the following news description to French. Return ONLY the translated text, no comments, no quotes:\n\n{}", description);
                if debug_mode {
                    tracing::info!("📝 [LLM DEBUG] Prompt Desc: {}", prompt);
                }
                match llm_service.complete(&prompt).await {
                    Ok(translated_desc) => {
                        if debug_mode {
                            tracing::info!("✨ [LLM DEBUG] Translated Desc: {}", translated_desc);
                        }
                        item["description"] = json!(translated_desc);
                    }
                    Err(e) => {
                        tracing::error!("❌ [LLM ERROR] Description translation failed: {}", e);
                    }
                }
            }

            // Store the individual item in S3 (non-blocking)
            if let Some((s3_client, bucket)) = s3_info.clone() {
                let s3_client_inner = s3_client.clone();
                let bucket_inner = bucket.clone();
                let item_inner = item.clone(); // Use `item` after potential modifications
                
                let url_for_log = item_inner["url"].as_str().unwrap_or("unknown").to_string();
                
                tokio::spawn(async move {
                    if let Err(e) = store_individual_news_item(&s3_client_inner, &bucket_inner, item_inner).await {
                        tracing::error!("❌ Background S3 store failed: {}", e);
                    } else {
                        tracing::debug!("✅ News item stored in S3: {}", url_for_log);
                    }
                });
            }

            item
        }));
    }

    let mut translated_items = Vec::new();
    while let Some(result) = tasks.next().await {
        if let Ok(item) = result {
            translated_items.push(item);
        }
    }

    // Re-sort since FuturesUnordered can change order
    translated_items.sort_by(|a, b| {
        let date_a = a["published_at"].as_str().unwrap_or("");
        let date_b = b["published_at"].as_str().unwrap_or("");
        date_b.cmp(date_a)
    });

    translated_items
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
