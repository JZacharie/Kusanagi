use crate::domain::entities::{A2UIMessage, DataModel, Surface};
use crate::domain::ports::A2UIRepository;
use anyhow::{anyhow, Result};
use std::sync::Arc;

pub struct A2UIUseCase {
    pub repository: Arc<dyn A2UIRepository>,
}

impl A2UIUseCase {
    pub fn new(repository: Arc<dyn A2UIRepository>) -> Self {
        Self { repository }
    }

    pub async fn process_message(&self, message: A2UIMessage) -> Result<()> {
        match message {
            A2UIMessage::SurfaceUpdate {
                surface_id,
                components,
            } => {
                let mut surface = self
                    .repository
                    .get_surface(&surface_id)
                    .await?
                    .unwrap_or_else(|| Surface::new(surface_id.clone()));

                surface.update_components(components);
                self.repository.save_surface(surface).await?;
            }
            A2UIMessage::DataModelUpdate { surface_id, data } => {
                let mut data_model = self
                    .repository
                    .get_data_model(&surface_id)
                    .await?
                    .unwrap_or_default();

                if let Some(obj) = data.as_object() {
                    for (k, v) in obj {
                        data_model.values.insert(k.clone(), v.clone());
                    }
                }

                self.repository
                    .save_data_model(&surface_id, data_model)
                    .await?;
            }
            A2UIMessage::UserAction {
                surface_id,
                action_id,
                component_id,
                payload: _,
            } => {
                tracing::info!(
                    "User Action received: {} on component {} for surface {}",
                    action_id,
                    component_id,
                    surface_id
                );
                // Here we would typically notify the agent or handle the action logic
            }
        }
        Ok(())
    }

    pub async fn get_surface(&self, surface_id: &str) -> Result<Surface> {
        self.repository
            .get_surface(surface_id)
            .await?
            .ok_or_else(|| anyhow!("Surface not found: {}", surface_id))
    }

    pub async fn get_data_model(&self, surface_id: &str) -> Result<DataModel> {
        self.repository
            .get_data_model(surface_id)
            .await?
            .ok_or_else(|| anyhow!("Data model not found for surface: {}", surface_id))
    }
}
