use crate::infrastructure::s3_utils::configure_insecure_s3;
use aws_sdk_s3::Client as S3Client;
use chrono::Utc;
use reqwest::Client;
use serde_json::{json, Value};

const CACHE_PREFIX: &str = "streaming/";
const BASE_URL: &str = "https://cinestream.info";
const JUSTWATCH_BASE: &str = "https://www.justwatch.com";

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub async fn get_streaming_data() -> Result<Value, String> {
    tracing::debug!("🎬 Streaming service: Attempting to get data from cache");

    match get_aggregated_streaming().await {
        Ok(data)
            if !data["items"]
                .as_array()
                .map(|a| a.is_empty())
                .unwrap_or(true) =>
        {
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
    tracing::info!("🔄 Fetching fresh streaming data from all sources...");

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    let mut new_items: Vec<Value> = Vec::new();

    // --- Source 1: Cinestream ---
    let cinestream_url = format!("{}/films-populaires/1", BASE_URL);
    match client.get(&cinestream_url).send().await {
        Ok(response) if response.status().is_success() => {
            if let Ok(html) = response.text().await {
                let items = parse_cinestream_html(&html);
                tracing::info!("🎬 Cinestream: parsed {} movies", items.len());
                new_items.extend(items);
            }
        }
        Ok(response) => tracing::warn!("⚠️ Cinestream returned HTTP {}", response.status()),
        Err(e) => tracing::warn!("⚠️ Failed to fetch Cinestream: {}", e),
    }

    // --- Source 2: JustWatch Films ---
    let jw_films_url = format!("{}/fr/films", JUSTWATCH_BASE);
    match client.get(&jw_films_url).send().await {
        Ok(response) if response.status().is_success() => {
            if let Ok(html) = response.text().await {
                let items = parse_justwatch_html(&html, "film");
                tracing::info!("🎬 JustWatch Films: parsed {} movies", items.len());
                new_items.extend(items);
            }
        }
        Ok(response) => tracing::warn!("⚠️ JustWatch Films returned HTTP {}", response.status()),
        Err(e) => tracing::warn!("⚠️ Failed to fetch JustWatch Films: {}", e),
    }

    // --- Source 3: JustWatch Series ---
    let jw_series_url = format!("{}/fr/series", JUSTWATCH_BASE);
    match client.get(&jw_series_url).send().await {
        Ok(response) if response.status().is_success() => {
            if let Ok(html) = response.text().await {
                let items = parse_justwatch_html(&html, "serie");
                tracing::info!("📺 JustWatch Series: parsed {} series", items.len());
                new_items.extend(items);
            }
        }
        Ok(response) => tracing::warn!("⚠️ JustWatch Series returned HTTP {}", response.status()),
        Err(e) => tracing::warn!("⚠️ Failed to fetch JustWatch Series: {}", e),
    }

    if new_items.is_empty() {
        return Err("No items parsed from any source".to_string());
    }

    // Attempt to load existing data to merge
    let mut all_items = match get_aggregated_streaming().await {
        Ok(data) => data["items"].as_array().cloned().unwrap_or_default(),
        Err(e) => {
            tracing::warn!("⚠️ Could not load existing streaming data for merging: {}. Starting fresh collection.", e);
            Vec::new()
        }
    };

    let s3_client = create_s3_client().await.ok();
    let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "kusanagi-news".to_string());

    // Merge new items (avoid duplicates by URL and by normalized title)
    let initial_count = all_items.len();
    for item in new_items {
        let item_url = item["url"].as_str().unwrap_or_default();
        let item_title_norm = normalize_title(item["title"].as_str().unwrap_or_default());

        let is_duplicate = all_items.iter().any(|existing| {
            // Duplicate by URL
            if existing["url"].as_str() == Some(item_url) {
                return true;
            }
            // Duplicate by normalized title (cross-source dedup)
            let existing_title_norm =
                normalize_title(existing["title"].as_str().unwrap_or_default());
            !item_title_norm.is_empty() && item_title_norm == existing_title_norm
        });

        if !is_duplicate {
            all_items.insert(0, item);
        }
    }

    // Process all items to ensure posters are cached and proxied
    if let Some(s3) = s3_client.as_ref() {
        for item in all_items.iter_mut() {
            if let Some(poster_url) = item["poster_url"].as_str() {
                // If not already proxied, try to cache it
                if !poster_url.starts_with("/api/streaming/poster/") {
                    if let Some(url) = item["url"].as_str() {
                        let mut hasher = DefaultHasher::new();
                        url.hash(&mut hasher);
                        let url_hash = format!("{:x}", hasher.finish());

                        match cache_poster(s3, &bucket, &url_hash, poster_url, &client).await {
                            Ok(proxy_path) => {
                                item["poster_url"] = json!(proxy_path);
                                // Also store/update individual JSON metadata with proxied URL
                                let _ = store_individual_movie(s3, &bucket, &url_hash, item).await;
                            }
                            Err(e) => {
                                tracing::warn!("⚠️ Failed to cache poster for {}: {}", url, e)
                            }
                        }
                    }
                }
            }
        }
    }

    let added_count = all_items.len() - initial_count;
    tracing::info!(
        "✅ Added {} new items to the collection (Total: {})",
        added_count,
        all_items.len()
    );

    // Limit collection size to 500 items
    if all_items.len() > 500 {
        all_items.truncate(500);
    }

    let response_data = json!({
        "items": all_items,
        "cached_at": Utc::now().to_rfc3339()
    });

    // Cache the result in S3 (latest aggregation)
    if let Some(s3) = s3_client {
        let key = format!("{}latest.json", CACHE_PREFIX);
        let json_bytes = serde_json::to_vec(&response_data).unwrap_or_default();

        match s3
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

async fn cache_poster(
    s3: &S3Client,
    bucket: &str,
    hash: &str,
    poster_url: &str,
    http_client: &Client,
) -> Result<String, String> {
    let key = format!("{}posters/{}.jpg", CACHE_PREFIX, hash);

    // Check if poster already exists
    if s3
        .head_object()
        .bucket(bucket)
        .key(&key)
        .send()
        .await
        .is_ok()
    {
        return Ok(format!("/api/streaming/poster/{}", hash));
    }

    // Ensure absolute URL
    let absolute_url = if poster_url.starts_with("/") && !poster_url.starts_with("//") {
        format!("{}{}", BASE_URL, poster_url)
    } else if poster_url.starts_with("//") {
        format!("https:{}", poster_url)
    } else {
        poster_url.to_string()
    };

    // Download poster
    let response = http_client
        .get(absolute_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;

    // Store in S3
    s3.put_object()
        .bucket(bucket)
        .key(&key)
        .body(bytes.into())
        .content_type("image/jpeg")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("/api/streaming/poster/{}", hash))
}

async fn store_individual_movie(
    s3: &S3Client,
    bucket: &str,
    hash: &str,
    item: &Value,
) -> Result<(), String> {
    let day = Utc::now().format("%Y-%m-%d").to_string();
    let key = format!("{}{}/{}.json", CACHE_PREFIX, day, hash);

    let json_bytes = serde_json::to_vec(item).map_err(|e| e.to_string())?;

    s3.put_object()
        .bucket(bucket)
        .key(key)
        .body(json_bytes.into())
        .content_type("application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn get_poster_data(hash: &str) -> Result<(Vec<u8>, String), String> {
    let s3 = create_s3_client().await?;
    let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "kusanagi-news".to_string());
    let key = format!("{}posters/{}.jpg", CACHE_PREFIX, hash);

    let result = s3
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let body = result.body.collect().await.map_err(|e| e.to_string())?;
    Ok((body.into_bytes().to_vec(), "image/jpeg".to_string()))
}

fn parse_cinestream_html(html: &str) -> Vec<Value> {
    let mut movies = Vec::new();

    // Cinestream uses Next.js with flight data.
    // We look for self.__next_f.push scripts which contain the movie data.
    let mut current_pos = 0;
    while let Some(script_start) = html[current_pos..].find("self.__next_f.push([1,\"") {
        let absolute_start = current_pos + script_start + 23;
        if let Some(script_end) = html[absolute_start..].find("\"])") {
            let absolute_end = absolute_start + script_end;
            let content = &html[absolute_start..absolute_end];

            // The content is escaped JSON-like data. We look for patterns like:
            // \"title\": \"...\", \"href\": \"...\", \"src\": \"...\"

            // This is a naive but effective way to extract from Next.js flight data strings
            // A more robust way would be to properly unescape and parse, but flight data is complex.

            // Try to find movie patterns in this chunk
            if content.contains("\\\"title\\\"") && content.contains("\\\"href\\\"") {
                if let Some(movie) = parse_movie_from_json_chunk(content) {
                    movies.push(movie);
                }
            }

            current_pos = absolute_end;
        } else {
            break;
        }
    }

    // Fallback to old article parsing if Next.js data wasn't found or parsed nothing
    if movies.is_empty() {
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
    }

    movies
}

fn parse_movie_from_json_chunk(content: &str) -> Option<Value> {
    // Helper to extract \"key\":\"value\"
    let extract = |key: &str| -> Option<String> {
        let marker = format!("\\\\\\\"{}\\\\\\\":\\\\\\\"", key);
        if let Some(start_pos) = content.find(&marker) {
            let start = start_pos + marker.len();
            if let Some(end_pos) = content[start..].find("\\\\\\\"") {
                return Some(content[start..start + end_pos].replace("\\\\", ""));
            }
        }
        None
    };

    let title = extract("title")?;
    let path = extract("href")?;
    let poster_url = extract("src");

    let url = if path.starts_with("http") {
        path
    } else {
        format!("{}{}", BASE_URL, path)
    };

    // Try to find language and quality nearby
    let mut language = "Unknown".to_string();
    let mut quality = "HD".to_string();

    if content.contains("TrueFrench") {
        language = "TrueFrench".to_string();
    } else if content.contains("VOSTFR") {
        language = "VOSTFR".to_string();
    }

    if content.contains("HDLight") {
        quality = "HDLight".to_string();
    } else if content.contains("WEB-DL") {
        quality = "WEB-DL".to_string();
    }

    Some(json!({
        "title": title,
        "url": url,
        "poster_url": poster_url,
        "year": "2025", // Hardcoded fallback for now as Next.js data is noisy
        "genres": "",
        "language": language,
        "quality": quality,
        "source": "Cinestream",
        "content_type": "film"
    }))
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
                let path = content[src_start..src_start + src_end].replace("&amp;", "&");
                if path.starts_with("/") && !path.starts_with("//") {
                    format!("{}{}", BASE_URL, path)
                } else if path.starts_with("//") {
                    format!("https:{}", path)
                } else {
                    path
                }
            })
        })
    });

    // Extract Year
    let year = extract_simple_content(content, "text-muted-foreground\">", "</span>")
        .unwrap_or_else(|| "N/A".to_string());

    // Extract Genres
    let genres =
        extract_simple_content(content, "truncate-multiline\">", "</span>").unwrap_or_default();

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
        "source": "Cinestream",
        "content_type": "film"
    }))
}

fn extract_simple_content(html: &str, class_marker: &str, end_tag: &str) -> Option<String> {
    if let Some(marker_pos) = html.find(class_marker) {
        let after_marker = marker_pos + class_marker.len();

        // Find the closing '>' of the current tag
        if let Some(tag_end) = html[after_marker..].find('>') {
            let start = after_marker + tag_end + 1;

            if let Some(end_pos) = html[start..].find(end_tag) {
                let content = html[start..start + end_pos].trim();
                // Sanitize: remove any leading '>' artifacts
                let cleaned = content.trim_start_matches('>').trim();
                if !cleaned.is_empty() {
                    return Some(cleaned.to_string());
                }
            }
        }
    }
    None
}

/// Parse JustWatch listing HTML (films or series)
fn parse_justwatch_html(html: &str, content_type: &str) -> Vec<Value> {
    let mut items = Vec::new();
    let link_prefix = if content_type == "serie" {
        "/fr/serie/"
    } else {
        "/fr/film/"
    };

    // Find all title-list-grid__item--link anchors
    let mut current_pos = 0;
    while let Some(link_pos) = html[current_pos..].find("title-list-grid__item--link") {
        let abs_link_pos = current_pos + link_pos;

        // On the same <a> tag, href appears AFTER the class attribute
        if let Some(href_relative) = html[abs_link_pos..].find("href=\"") {
            let href_start = abs_link_pos + href_relative + 6;
            if let Some(href_end) = html[href_start..].find('"') {
                let path = &html[href_start..href_start + href_end];

                if path.starts_with(link_prefix) || path.contains(link_prefix) {
                    // Compute block once for img extraction
                    let mut search_end = std::cmp::min(abs_link_pos + 5000, html.len());
                    while !html.is_char_boundary(search_end) && search_end < html.len() {
                        search_end += 1;
                    }
                    if !html.is_char_boundary(search_end) {
                        search_end = html.len();
                    }
                    let block = &html[abs_link_pos..search_end];

                    // Try data-title from parent div, fallback to img alt
                    let title = extract_data_title(html, abs_link_pos)
                        .or_else(|| extract_img_alt(block).map(|t| t.trim().to_string()))
                        .filter(|t| !t.is_empty());

                    if let Some(title) = title {
                        let poster_url = extract_img_src(block);
                        let url = format!("{}{}", JUSTWATCH_BASE, path);

                        items.push(json!({
                            "title": title,
                            "url": url,
                            "poster_url": poster_url,
                            "year": "N/A",
                            "genres": "",
                            "language": "N/A",
                            "quality": "N/A",
                            "source": "JustWatch",
                            "content_type": content_type
                        }));
                    }
                }
            }
        }

        current_pos = abs_link_pos + 30;
        while current_pos < html.len() && !html.is_char_boundary(current_pos) {
            current_pos += 1;
        }
    }

    items
}

/// Extract the alt attribute from the first <img> tag in the block
fn extract_img_alt(block: &str) -> Option<String> {
    let img_pos = block.find("<img")?;
    let after_img = &block[img_pos..];
    let alt_pos = after_img.find("alt=\"")?;
    let alt_start = alt_pos + 5;
    let alt_end = after_img[alt_start..].find('"')?;
    let alt = &after_img[alt_start..alt_start + alt_end];
    // Decode HTML entities
    let decoded = alt
        .replace("&amp;", "&")
        .replace("&#x27;", "'")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">");
    Some(decoded)
}

/// Extract the src attribute from the first <img> tag in the block
fn extract_img_src(block: &str) -> Option<String> {
    let img_pos = block.find("<img")?;
    let after_img = &block[img_pos..];
    let src_pos = after_img.find("src=\"")?;
    let src_start = src_pos + 5;
    let src_end = after_img[src_start..].find('"')?;
    let src = &after_img[src_start..src_start + src_end];
    Some(src.replace("&amp;", "&"))
}

/// Extract the title from the parent div's data-title attribute
fn extract_data_title(html: &str, abs_link_pos: usize) -> Option<String> {
    // Search backward from the link to find parent div with data-title="..."
    let search_start = abs_link_pos.saturating_sub(1000);
    let before = &html[search_start..abs_link_pos];
    if let Some(last_div) = before.rfind("title-list-grid__item\"") {
        let div_start = search_start + last_div;
        let after_class = &html[div_start..abs_link_pos];
        if let Some(data_title_pos) = after_class.find("data-title=\"") {
            let title_start = div_start + data_title_pos + 12;
            if let Some(title_end) = html[title_start..].find('"') {
                let title = &html[title_start..title_start + title_end];
                if !title.is_empty() {
                    return Some(title.to_string());
                }
            }
        }
    }
    None
}

/// Normalize a title for deduplication (lowercase, strip accents, remove special chars)
fn normalize_title(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'á' | 'â' | 'ä' | 'ã' => 'a',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ò' | 'ó' | 'ô' | 'ö' | 'õ' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            'æ' => 'a',
            'œ' => 'o',
            _ => c,
        })
        .filter(|c| c.is_alphanumeric() || *c == ' ')
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

async fn get_aggregated_streaming() -> Result<Value, String> {
    let s3_client = create_s3_client().await?;
    let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "kusanagi-news".to_string());
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
    let endpoint =
        std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://192.168.0.170:9010".to_string());
    let region = std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let access_key =
        std::env::var("S3_ACCESS_KEY").map_err(|_| "S3_ACCESS_KEY not set".to_string())?;
    let secret_key =
        std::env::var("S3_SECRET_KEY").map_err(|_| "S3_SECRET_KEY not set".to_string())?;

    let credentials =
        aws_sdk_s3::config::Credentials::new(access_key, secret_key, None, None, "custom");
    let mut s3_config_builder = aws_sdk_s3::config::Builder::new()
        .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
        .region(aws_sdk_s3::config::Region::new(region))
        .endpoint_url(&endpoint)
        .credentials_provider(credentials)
        .force_path_style(true);

    let ignore_ssl = std::env::var("S3_IGNORE_SSL").unwrap_or_default() == "true"
        || endpoint.starts_with("http://192.168.0.170");

    if ignore_ssl {
        s3_config_builder = configure_insecure_s3(s3_config_builder);
    }

    let s3_config = s3_config_builder.build();

    Ok(S3Client::from_conf(s3_config))
}
