use actix_web::{web, HttpResponse, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use crate::translation;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub id: String,
    pub source: String,
    pub title: String,
    pub url: String,
    pub description: Option<String>,
    pub published_at: DateTime<Utc>,
    pub score: Option<i32>,
    pub tags: Vec<String>,
    pub translated_title: Option<String>,
    pub translated_description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NewsFeedResponse {
    pub items: Vec<NewsItem>,
    pub total: usize,
    pub cached_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct NewsCache {
    items: Arc<RwLock<Vec<NewsItem>>>,
    last_update: Arc<RwLock<DateTime<Utc>>>,
}

impl NewsCache {
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(Vec::new())),
            last_update: Arc::new(RwLock::new(Utc::now())),
        }
    }

    pub async fn get_items(&self) -> Vec<NewsItem> {
        self.items.read().await.clone()
    }

    pub async fn update_items(&self, new_items: Vec<NewsItem>) {
        let mut current_items = self.items.write().await;
        
        // Use a HashMap for de-duplication based on ID
        let mut items_map: HashMap<String, NewsItem> = current_items
            .drain(..)
            .map(|item| (item.id.clone(), item))
            .collect();

        // Add new items, overwriting (updating) if same ID exists
        for item in new_items {
            items_map.insert(item.id.clone(), item);
        }

        // Convert back to Vec and sort
        let mut all_items: Vec<NewsItem> = items_map.into_values().collect();
        all_items.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        
        // Cap total items to prevent infinite memory growth (e.g., 500 items)
        if all_items.len() > 500 {
            all_items.truncate(500);
        }

        *current_items = all_items;
        *self.last_update.write().await = Utc::now();
    }


    pub async fn last_update(&self) -> DateTime<Utc> {
        *self.last_update.read().await
    }

    pub async fn should_refresh(&self) -> bool {
        let last = self.last_update().await;
        let now = Utc::now();
        (now - last).num_minutes() >= 30
    }
}

// Fetch Hacker News top stories
async fn fetch_hackernews() -> Result<Vec<NewsItem>, String> {
    let client = reqwest::Client::new();
    
    // Get top story IDs
    let top_stories: Vec<u64> = client
        .get("https://hacker-news.firebaseio.com/v0/topstories.json")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    
    // Fetch first 50 stories for better diversity
    for id in top_stories.iter().take(50) {
        let story: serde_json::Value = client
            .get(format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        if let Some(title) = story.get("title").and_then(|v| v.as_str()) {
            let title_lower = title.to_lowercase();
            
            // Primary tags based on content
            let mut tags = Vec::new();
            if title_lower.contains("docker") || title_lower.contains("container") { tags.push("docker".to_string()); }
            if title_lower.contains("kubernetes") || title_lower.contains("k8s") { tags.push("k8s".to_string()); }
            if title_lower.contains("devops") || title_lower.contains("gitops") { tags.push("devops".to_string()); }
            if title_lower.contains("rust") { tags.push("rust".to_string()); }
            if title_lower.contains("ai") || title_lower.contains("llm") { tags.push("ai".to_string()); }
            
            // If no specific tags found, default to general "tech"
            if tags.is_empty() {
                tags.push("tech".to_string());
            }

            let url = story.get("url")
                .and_then(|v| v.as_str())
                .unwrap_or(&format!("https://news.ycombinator.com/item?id={}", id))
                .to_string();

            let timestamp = story.get("time")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let score = story.get("score")
                .and_then(|v| v.as_i64())
                .map(|s| s as i32);

            items.push(NewsItem {
                id: format!("hn_{}", id),
                source: "hackernews".to_string(),
                title: title.to_string(),
                url,
                description: None,
                published_at: DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now),
                score,
                tags,
                translated_title: None,
                translated_description: None,
            });
        }
    }

    Ok(items)
}

// Fetch Korben RSS feed
async fn fetch_korben_rss() -> Result<Vec<NewsItem>, String> {
    tracing::info!("Starting Korben RSS fetch from https://korben.info/feed");
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get("https://korben.info/feed")
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to send request to Korben: {}", e);
            e.to_string()
        })?;

    tracing::info!("Korben response status: {}", response.status());

    let content = response
        .bytes()
        .await
        .map_err(|e| {
            tracing::error!("Failed to get bytes from Korben response: {}", e);
            e.to_string()
        })?;

    let channel = rss::Channel::read_from(&content[..]).map_err(|e| e.to_string())?;
    let mut items = Vec::new();

    for item in channel.items() {
        let title = item.title().unwrap_or("").to_string();
        let description = item.description().unwrap_or("").to_string();
        let url = item.link().unwrap_or("").to_string();
        
        if url.is_empty() {
            continue;
        }

        let pub_date = item.pub_date()
            .and_then(|d| DateTime::parse_from_rfc2822(d).ok())
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        // Use URL hash or clean URL as stable ID
        use base64::{engine::general_purpose, Engine as _};
        let id = format!("korben_{}", general_purpose::URL_SAFE_NO_PAD.encode(url.as_bytes()));

        items.push(NewsItem {
            id,
            source: "korben".to_string(),
            title,
            url,
            description: Some(description),
            published_at: pub_date,
            score: None,
            tags: vec!["tech".to_string(), "french".to_string()],
            translated_title: None,
            translated_description: None,
        });
    }

    tracing::info!("Successfully fetched {} items from Korben", items.len());
    Ok(items)
}

// Fetch GitHub trending repositories
async fn fetch_github_trending() -> Result<Vec<NewsItem>, String> {
    let client = reqwest::Client::builder()
        .user_agent("Kusanagi-Dashboard/1.0")
        .build()
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    let topics = vec!["docker", "kubernetes", "devops"];

    for topic in topics {
        let url = format!(
            "https://api.github.com/search/repositories?q=topic:{}&sort=stars&order=desc&per_page=10",
            topic
        );

        let response: serde_json::Value = client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        if let Some(repos) = response.get("items").and_then(|v| v.as_array()) {
            for repo in repos {
                let name = repo.get("full_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let description = repo.get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let html_url = repo.get("html_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let stars = repo.get("stargazers_count")
                    .and_then(|v| v.as_i64())
                    .map(|s| s as i32);

                let updated_at = repo.get("updated_at")
                    .and_then(|v| v.as_str())
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now);

                items.push(NewsItem {
                    id: format!("gh_{}", name.replace("/", "_")),
                    source: "github".to_string(),
                    title: name,
                    url: html_url,
                    description,
                    published_at: updated_at,
                    score: stars,
                    tags: vec![topic.to_string()],
                    translated_title: None,
                    translated_description: None,
                });
            }
        }
    }

    Ok(items)
}

// Aggregate all news sources
async fn aggregate_news() -> Result<Vec<NewsItem>, String> {
    let (hn_result, korben_result, gh_result) = tokio::join!(
        fetch_hackernews(),
        fetch_korben_rss(),
        fetch_github_trending()
    );

    let mut all_items = Vec::new();

    if let Ok(items) = hn_result {
        all_items.extend(items);
    }

    if let Ok(items) = korben_result {
        all_items.extend(items);
    }

    if let Ok(items) = gh_result {
        all_items.extend(items);
    }

    // Sort by published date (newest first)
    all_items.sort_by(|a, b| b.published_at.cmp(&a.published_at));

    Ok(all_items)
}

// Background task to refresh cache
pub async fn start_news_refresh_task(cache: NewsCache) {
    // Load initial data immediately
    tokio::spawn(async move {
        // Initial load from S3
        tracing::info!("Loading news from S3...");
        let s3_client = translation::get_s3_client().await;
        match translation::get_news_from_s3(&s3_client).await {
            Ok(items) => {
                if !items.is_empty() {
                    cache.update_items(items.clone()).await;
                    tracing::info!("Loaded {} items from S3", items.len());
                }
            }
            Err(e) => {
                tracing::warn!("Failed to load news from S3: {}", e);
            }
        }

        // Fetch fresh news
        tracing::info!("Fetching fresh news...");
        match aggregate_news().await {
            Ok(items) => {
                cache.update_items(items.clone()).await;
                tracing::info!("News cache updated: {} items", items.len());
                
                // Store fresh items to S3
                for item in &items {
                    if let Err(e) = translation::store_news_item(&s3_client, item).await {
                        tracing::error!("Failed to store news item {} to S3: {}", item.id, e);
                    }
                }

                // Start background translation
                let cache_clone = cache.clone();
                tokio::spawn(async move {
                    process_news_enrichment(cache_clone).await;
                });
            }
            Err(e) => {
                tracing::error!("Failed to fetch fresh news: {}", e);
            }
        }

        // Then refresh every 30 minutes
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1800)); // 30 minutes
        interval.tick().await; // Skip first tick (immediate)
        
        loop {
            interval.tick().await;
            if cache.should_refresh().await {
                tracing::info!("Refreshing news cache...");
                match aggregate_news().await {
                    Ok(items) => {
                        cache.update_items(items.clone()).await;
                        tracing::info!("News cache refreshed successfully");
                        
                        let s3_client = translation::get_s3_client().await;
                        for item in &items {
                            if let Err(e) = translation::store_news_item(&s3_client, item).await {
                                tracing::error!("Failed to store news item {} to S3: {}", item.id, e);
                            }
                        }

                        // Start background translation
                        let cache_clone = cache.clone();
                        tokio::spawn(async move {
                            process_news_enrichment(cache_clone).await;
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to refresh news cache: {}", e);
                    }
                }
            }
        }
    });
}

async fn process_news_enrichment(cache: NewsCache) {
    let s3_client = translation::get_s3_client().await;
    if let Err(e) = translation::ensure_bucket_exists(&s3_client).await {
        tracing::error!("Failed to ensure translation bucket exists: {}", e);
        return;
    }

    let items = cache.get_items().await;
    for item in items {
        let mut needs_update = false;
        let mut updated_item = item.clone();

        // 1. Generate tags if missing key:value tags
        let has_kv_tags = item.tags.iter().any(|t| t.contains(':'));
        if !has_kv_tags {
            tracing::info!("Generating tags for news item: {}", item.title);
            match translation::generate_tags_with_ollama(&item.title).await {
                Ok(new_tags) => {
                    if !new_tags.is_empty() {
                        updated_item.tags = new_tags;
                        needs_update = true;
                    }
                }
                Err(e) => tracing::warn!("Failed to generate tags for {}: {}", item.id, e),
            }
        }

        // 2. Translate if not French source and not yet translated
        if item.source != "korben" && item.translated_title.is_none() {
            // Try to get from S3 cache first
            if let Some(cached) = translation::get_cached_translation(&s3_client, &item.id).await {
                updated_item.translated_title = Some(cached.title);
                updated_item.translated_description = cached.description;
                needs_update = true;
            } else {
                tracing::info!("Translating news item: {}", item.title);
                
                // Translate title
                if let Ok(t_title) = translation::translate_with_ollama(&item.title, "fr").await {
                    updated_item.translated_title = Some(t_title);
                    needs_update = true;

                    // Translate description if present
                    if let Some(desc) = &item.description {
                        if !desc.trim().is_empty() {
                            if let Ok(t_desc) = translation::translate_with_ollama(desc, "fr").await {
                                updated_item.translated_description = Some(t_desc);
                            }
                        }
                    }
                }
            }
        }

        if needs_update {
            // Update in-memory cache
            let mut items_write = cache.items.write().await;
            if let Some(i) = items_write.iter_mut().find(|i| i.id == item.id) {
                *i = updated_item.clone();
            }

            // Persist to S3
            if let Err(e) = translation::store_news_item(&s3_client, &updated_item).await {
                tracing::error!("Failed to persist enriched news item {} to S3: {}", item.id, e);
            }

            // Delay to avoid overwhelming Ollama
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }
}

// API endpoint handler
pub async fn get_news(
    cache: web::Data<NewsCache>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse> {
    // Refresh cache if needed
    if cache.should_refresh().await {
        match aggregate_news().await {
            Ok(items) => {
                cache.update_items(items).await;
            }
            Err(e) => {
                tracing::error!("Failed to fetch news: {}", e);
            }
        }
    }

    let mut items = cache.get_items().await;

    // Filter by source if specified
    if let Some(source) = query.get("source") {
        items.retain(|item| item.source == *source);
    }

    // Filter by tag if specified
    if let Some(tag) = query.get("tag") {
        items.retain(|item| item.tags.contains(tag) || item.tags.iter().any(|t| t.split(':').next_back() == Some(tag)));
    }

    // Limit results if specified
    if let Some(limit_str) = query.get("limit") {
        if let Ok(limit) = limit_str.parse::<usize>() {
            items.truncate(limit);
        }
    }

    let response = NewsFeedResponse {
        total: items.len(),
        items,
        cached_at: cache.last_update().await,
    };

    Ok(HttpResponse::Ok().json(response))
}
