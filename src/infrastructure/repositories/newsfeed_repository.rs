use async_trait::async_trait;
use crate::domain::ports::{NewsfeedRepository, NewsItem};
use crate::error::{Result, KusanagiError};
use crate::legacy;

pub struct LegacyNewsfeedRepository;

#[async_trait]
impl NewsfeedRepository for LegacyNewsfeedRepository {
    async fn get_news(&self) -> Result<Vec<NewsItem>> {
        Ok(vec![])
    }

    async fn refresh_news(&self) -> Result<()> {
        Ok(())
    }

    async fn get_github_trending(&self) -> Result<Vec<NewsItem>> {
        Ok(vec![])
    }

    async fn get_hackernews(&self) -> Result<Vec<NewsItem>> {
        Ok(vec![])
    }
}
