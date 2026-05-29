use crate::domain::entities::{DataModel, Surface};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait A2UIRepository: Send + Sync {
    async fn get_surface(&self, surface_id: &str) -> Result<Option<Surface>>;
    async fn save_surface(&self, surface: Surface) -> Result<()>;

    async fn get_data_model(&self, surface_id: &str) -> Result<Option<DataModel>>;
    async fn save_data_model(&self, surface_id: &str, data_model: DataModel) -> Result<()>;

    async fn delete_surface(&self, surface_id: &str) -> Result<()>;
}
