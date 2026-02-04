use async_trait::async_trait;
use crate::error::Result;

#[async_trait]
pub trait NewsfeedRepository: Send + Sync {
    async fn get_news(&self) -> Result<Vec<NewsItem>>;
    async fn refresh_news(&self) -> Result<()>;
    async fn get_github_trending(&self) -> Result<Vec<NewsItem>>;
    async fn get_hackernews(&self) -> Result<Vec<NewsItem>>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NewsItem {
    pub id: String,
    pub title: String,
    pub url: String,
    pub source: String,
    pub published_at: chrono::DateTime<chrono::Utc>,
    pub summary: Option<String>,
    pub tags: Vec<String>,
}
