use crate::domain::entities::{DataModel, Surface};
use crate::domain::ports::A2UIRepository;
use crate::AdvancedCache;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

pub struct A2UIRepositoryImpl {
    cache: Arc<AdvancedCache<String>>,
}

impl A2UIRepositoryImpl {
    pub fn new(cache: Arc<AdvancedCache<String>>) -> Self {
        Self { cache }
    }
}

#[async_trait]
impl A2UIRepository for A2UIRepositoryImpl {
    async fn get_surface(&self, surface_id: &str) -> Result<Option<Surface>> {
        let key = format!("a2ui:surface:{}", surface_id);
        if let Some(json) = self.cache.get(&key).await {
            return Ok(Some(serde_json::from_str(&json)?));
        }
        Ok(None)
    }

    async fn save_surface(&self, surface: Surface) -> Result<()> {
        let key = format!("a2ui:surface:{}", surface.id);
        let json = serde_json::to_string(&surface)?;
        self.cache.set(key, json, None).await;
        Ok(())
    }

    async fn get_data_model(&self, surface_id: &str) -> Result<Option<DataModel>> {
        let key = format!("a2ui:data:{}", surface_id);
        if let Some(json) = self.cache.get(&key).await {
            return Ok(Some(serde_json::from_str(&json)?));
        }
        Ok(None)
    }

    async fn save_data_model(&self, surface_id: &str, data_model: DataModel) -> Result<()> {
        let key = format!("a2ui:data:{}", surface_id);
        let json = serde_json::to_string(&data_model)?;
        self.cache.set(key, json, None).await;
        Ok(())
    }

    async fn delete_surface(&self, surface_id: &str) -> Result<()> {
        let s_key = format!("a2ui:surface:{}", surface_id);
        let d_key = format!("a2ui:data:{}", surface_id);
        self.cache.delete(&s_key).await;
        self.cache.delete(&d_key).await;
        Ok(())
    }
}
