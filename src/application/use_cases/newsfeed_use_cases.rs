use crate::domain::ports::{NewsfeedRepository, NewsItem};
use crate::error::Result;
use std::sync::Arc;

pub struct GetNewsUseCase {
    newsfeed_repo: Arc<dyn NewsfeedRepository>,
}

impl GetNewsUseCase {
    pub fn new(newsfeed_repo: Arc<dyn NewsfeedRepository>) -> Self {
        Self { newsfeed_repo }
    }

    pub async fn execute(&self) -> Result<Vec<NewsItem>> {
        self.newsfeed_repo.get_news().await
    }
}

pub struct RefreshNewsUseCase {
    newsfeed_repo: Arc<dyn NewsfeedRepository>,
}

impl RefreshNewsUseCase {
    pub fn new(newsfeed_repo: Arc<dyn NewsfeedRepository>) -> Self {
        Self { newsfeed_repo }
    }

    pub async fn execute(&self) -> Result<()> {
        self.newsfeed_repo.refresh_news().await
    }
}
