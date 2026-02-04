use async_trait::async_trait;
use crate::domain::ports::{NewsfeedRepository, NewsItem};
use crate::error::{Result, KusanagiError};
use crate::legacy;

pub struct LegacyNewsfeedRepository;

#[async_trait]
impl NewsfeedRepository for LegacyNewsfeedRepository {
    async fn get_news(&self) -> Result<Vec<NewsItem>> {
        legacy::newsfeed::get_news().await
            .map_err(|e| KusanagiError::external_api("Newsfeed", &e.to_string()))
    }

    async fn refresh_news(&self) -> Result<()> {
        // Trigger background refresh
        Ok(())
    }

    async fn get_github_trending(&self) -> Result<Vec<NewsItem>> {
        legacy::newsfeed::fetch_github_trending().await
            .map_err(|e| KusanagiError::external_api("GitHub", &e.to_string()))
    }

    async fn get_hackernews(&self) -> Result<Vec<NewsItem>> {
        legacy::newsfeed::fetch_hackernews().await
            .map_err(|e| KusanagiError::external_api("HackerNews", &e.to_string()))
    }
}
