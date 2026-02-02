//! Storage Use Cases
//!
//! Application layer use cases for storage operations.

use crate::domain::entities::StorageInfo;
use crate::domain::ports::KubernetesRepository;
use crate::error::{KusanagiError, Result};
use std::sync::Arc;

/// Get storage information use case
pub struct GetStorageInfoUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl GetStorageInfoUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<StorageInfo> {
        self.repository.get_storage_info().await
    }
}

/// Storage statistics DTO
#[derive(Debug, Clone, serde::Serialize)]
pub struct StorageStats {
    pub total_pvs: usize,
    pub available_pvs: usize,
    pub bound_pvs: usize,
    pub released_pvs: usize,
    pub utilization_percent: f64,
}

/// Get storage statistics use case
pub struct GetStorageStatsUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl GetStorageStatsUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<StorageStats> {
        let info = self.repository.get_storage_info().await?;
        
        let utilization = if info.total_pvs > 0 {
            100.0 * info.bound_pvs as f64 / info.total_pvs as f64
        } else {
            0.0
        };
        
        Ok(StorageStats {
            total_pvs: info.total_pvs,
            available_pvs: info.available_pvs,
            bound_pvs: info.bound_pvs,
            released_pvs: info.released_pvs,
            utilization_percent: utilization,
        })
    }
}

/// Storage service - aggregates all storage use cases
pub struct StorageService {
    pub get_info: GetStorageInfoUseCase,
    pub get_stats: GetStorageStatsUseCase,
}

impl StorageService {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self {
            get_info: GetStorageInfoUseCase::new(repository.clone()),
            get_stats: GetStorageStatsUseCase::new(repository),
        }
    }
}
