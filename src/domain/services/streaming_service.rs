use aws_sdk_s3::Client as S3Client;
use chrono::Utc;
use reqwest::Client;
use serde_json::{json, Value};

const CACHE_PREFIX: &str = "streaming/";
const BASE_URL: &str = "https://cinestream.info";

pub async fn get_streaming_data() -> Result<Value, String> {
    tracing::debug!("🎬 Streaming service: Attempting to get data from cache");

    match get_aggregated_streaming().await {
        Ok(data) if !data["items"].as_array().map(|a| a.is_empty()).unwrap_or(true) => {
            tracing::info!("✅ Streaming cache HIT");
            Ok(data)
        }
        _ => {
            tracing::warn!("⚠️ Streaming cache MISS - fetching fresh data");
            fetch_fresh_streaming().await
        }
    }
}

pub async fn force_refresh() -> Result<Value, String> {
    fetch_fresh_streaming().await
}

async fn fetch_fresh_streaming() -> Result<Value, String> {
    tracing::info!("🔄 Fetching fresh streaming movies from Cinestream...");

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("{}/films-ajoutes-recemment/1", BASE_URL);
    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to fetch cinestream: HTTP {}", response.status()));
    }

    let html = response.text().await.map_err(|e| e.to_string())?;
    let new_items = parse_cinestream_html(&html);

    if new_items.is_empty() {
        return Err("No movies parsed from cinestream".to_string());
    }

    // Attempt to load existing data to merge
    let mut all_items = match get_aggregated_streaming().await {
        Ok(data) => data["items"].as_array().cloned().unwrap_or_default(),
        Err(e) => {
            tracing::warn!("⚠️ Could not load existing streaming data for merging: {}. Starting fresh collection.", e);
            Vec::new()
        }
    };

    // Merge new items (avoid duplicates by URL)
    let initial_count = all_items.len();
    for item in new_items {
        if let Some(url) = item["url"].as_str() {
            if !all_items.iter().any(|existing| existing["url"].as_str() == Some(url)) {
                all_items.insert(0, item);
            }
        }
    }
    
    let added_count = all_items.len() - initial_count;
    tracing::info!("✅ Added {} new movies to the collection (Total: {})", added_count, all_items.len());

    // Limit collection size to 500 items
    if all_items.len() > 500 {
        all_items.truncate(500);
    }

    let response_data = json!({
        "items": all_items,
        "cached_at": Utc::now().to_rfc3339()
    });

    // Cache the result in S3
    if let Ok(s3_client) = create_s3_client().await {
        let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "kusanagi".to_string());
        let key = format!("{}latest.json", CACHE_PREFIX);
        
        let json_bytes = serde_json::to_vec(&response_data).unwrap_or_default();
        
        match s3_client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(json_bytes.into())
            .content_type("application/json")
            .send()
            .await 
        {
            Ok(_) => tracing::info!("💾 Streaming collection updated in S3"),
            Err(e) => tracing::error!("❌ Failed to save streaming collection to S3: {}", e),
        }
    }

    Ok(response_data)
}

fn parse_cinestream_html(html: &str) -> Vec<Value> {
    let mut movies = Vec::new();
    
    // Cinestream uses a grid of <article> elements for movies
    let mut current_pos = 0;
    while let Some(article_start) = html[current_pos..].find("<article") {
        let absolute_start = current_pos + article_start;
        if let Some(article_end) = html[absolute_start..].find("</article>") {
            let absolute_end = absolute_start + article_end + 10;
            let article_content = &html[absolute_start..absolute_end];
            
            if let Some(movie) = parse_movie_article(article_content) {
                movies.push(movie);
            }
            
            current_pos = absolute_end;
        } else {
            break;
        }
    }
    
    movies
}

fn parse_movie_article(content: &str) -> Option<Value> {
    // Extract title
    let title = extract_simple_content(content, "text-lg text-foreground font-bold", "</span>")?;

    // Extract URL
    let url_start = content.find("href=\"")? + 6;
    let url_end = content[url_start..].find("\"")?;
    let path = &content[url_start..url_start + url_end];
    let url = if path.starts_with("http") {
        path.to_string()
    } else {
        format!("{}{}", BASE_URL, path)
    };

    // Extract Poster
    let poster_url = content.find("<img").and_then(|img_pos| {
        content[img_pos..].find("src=\"").and_then(|src_pos| {
            let src_start = img_pos + src_pos + 5;
            content[src_start..].find("\"").map(|src_end| {
                content[src_start..src_start + src_end].replace("&amp;", "&")
            })
        })
    });

    // Extract Year
    let year = extract_simple_content(content, "text-muted-foreground\">", "</span>")
        .unwrap_or_else(|| "N/A".to_string());

    // Extract Genres
    let genres = extract_simple_content(content, "truncate-multiline\">", "</span>")
        .unwrap_or_default();

    // Extract Language and Quality (Top badges)
    // <div class="absolute top-1 left-1 ..."><span>TrueFrench</span></div>
    // <div class="absolute top-1 right-1 ..."><span>HDLight</span></div>
    
    let mut language = "Unknown".to_string();
    let mut quality = "HD".to_string();
    
    if let Some(lang_pos) = content.find("top-1 left-1") {
        if let Some(span_pos) = content[lang_pos..].find("<span>") {
            let start = lang_pos + span_pos + 6;
            if let Some(end) = content[start..].find("</span>") {
                language = content[start..start + end].to_string();
            }
        }
    }
    
    if let Some(quality_pos) = content.find("top-1 right-1") {
        if let Some(span_pos) = content[quality_pos..].find("<span>") {
            let start = quality_pos + span_pos + 6;
            if let Some(end) = content[start..].find("</span>") {
                quality = content[start..start + end].to_string();
            }
        }
    }

    Some(json!({
        "title": title,
        "url": url,
        "poster_url": poster_url,
        "year": year,
        "genres": genres,
        "language": language,
        "quality": quality,
        "source": "Cinestream"
    }))
}

fn extract_simple_content(html: &str, class_marker: &str, end_tag: &str) -> Option<String> {
    if let Some(marker_pos) = html.find(class_marker) {
        let content_start = marker_pos + class_marker.len();
        // Skip > if we just searched for a class
        let start = if html[content_start..].starts_with(">") {
            content_start + 1
        } else {
            content_start
        };
        
        if let Some(end_pos) = html[start..].find(end_tag) {
            return Some(html[start..start + end_pos].trim().to_string());
        }
    }
    None
}

async fn get_aggregated_streaming() -> Result<Value, String> {
    let s3_client = create_s3_client().await?;
    let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "kusanagi".to_string());
    let key = format!("{}latest.json", CACHE_PREFIX);

    let result = s3_client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let body = result.body.collect().await.map_err(|e| e.to_string())?;
    let val: Value = serde_json::from_slice(&body.into_bytes()).map_err(|e| e.to_string())?;
    Ok(val)
}

async fn create_s3_client() -> Result<S3Client, String> {
    let endpoint = std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://192.168.0.170:9010".to_string());
    let region = std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let access_key = std::env::var("S3_ACCESS_KEY").map_err(|_| "S3_ACCESS_KEY not set".to_string())?;
    let secret_key = std::env::var("S3_SECRET_KEY").map_err(|_| "S3_SECRET_KEY not set".to_string())?;

    let credentials = aws_sdk_s3::config::Credentials::new(access_key, secret_key, None, None, "custom");
    let s3_config = aws_sdk_s3::config::Builder::new()
        .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
        .region(aws_sdk_s3::config::Region::new(region))
        .endpoint_url(&endpoint)
        .credentials_provider(credentials)
        .force_path_style(true)
        .build();

    Ok(S3Client::from_conf(s3_config))
}
