use actix_web::{web, HttpResponse, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NewsSource {
    HackerNews,
    Korben,
    GitHubTrending,
}

impl NewsSource {
    fn as_str(&self) -> &str {
        match self {
            NewsSource::HackerNews => "hackernews",
            NewsSource::Korben => "korben",
            NewsSource::GitHubTrending => "github",
        }
    }
}

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

    pub async fn update_items(&self, items: Vec<NewsItem>) {
        *self.items.write().await = items;
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
    
    // Fetch first 30 stories
    for id in top_stories.iter().take(30) {
        let story: serde_json::Value = client
            .get(&format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        if let Some(title) = story.get("title").and_then(|v| v.as_str()) {
            // Filter for Docker/Kubernetes/DevOps related content
            let title_lower = title.to_lowercase();
            let is_relevant = title_lower.contains("docker") 
                || title_lower.contains("kubernetes") 
                || title_lower.contains("k8s")
                || title_lower.contains("devops")
                || title_lower.contains("container")
                || title_lower.contains("cloud native")
                || title_lower.contains("gitops")
                || title_lower.contains("helm")
                || title_lower.contains("argocd");

            if is_relevant {
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
                    tags: vec!["tech".to_string()],
                });
            }
        }
    }

    Ok(items)
}

// Fetch Korben RSS feed
async fn fetch_korben_rss() -> Result<Vec<NewsItem>, String> {
    let client = reqwest::Client::new();
    let content = client
        .get("https://korben.info/feed")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;

    let channel = rss::Channel::read_from(&content[..]).map_err(|e| e.to_string())?;
    let mut items = Vec::new();

    for item in channel.items().iter().take(20) {
        let title = item.title().unwrap_or("").to_string();
        let title_lower = title.to_lowercase();
        let description = item.description().unwrap_or("").to_string();
        let desc_lower = description.to_lowercase();

        // Filter for tech/Docker/DevOps content
        let is_relevant = title_lower.contains("docker") 
            || title_lower.contains("kubernetes")
            || title_lower.contains("linux")
            || title_lower.contains("serveur")
            || title_lower.contains("cloud")
            || desc_lower.contains("docker")
            || desc_lower.contains("kubernetes")
            || desc_lower.contains("devops");

        if is_relevant {
            let url = item.link().unwrap_or("").to_string();
            let pub_date = item.pub_date()
                .and_then(|d| DateTime::parse_from_rfc2822(d).ok())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(Utc::now);

            items.push(NewsItem {
                id: format!("korben_{}", items.len()),
                source: "korben".to_string(),
                title,
                url,
                description: Some(description),
                published_at: pub_date,
                score: None,
                tags: vec!["tech".to_string(), "french".to_string()],
            });
        }
    }

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
        // Initial load
        tracing::info!("Loading initial news cache...");
        match aggregate_news().await {
            Ok(items) => {
                cache.update_items(items.clone()).await;
                tracing::info!("Initial news cache loaded: {} items", items.len());
            }
            Err(e) => {
                tracing::error!("Failed to load initial news cache: {}", e);
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
                        cache.update_items(items).await;
                        tracing::info!("News cache refreshed successfully");
                    }
                    Err(e) => {
                        tracing::error!("Failed to refresh news cache: {}", e);
                    }
                }
            }
        }
    });
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
