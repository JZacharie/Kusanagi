//! Event Use Cases
//!
//! Application layer use cases for Kubernetes events.

use crate::domain::entities::{ClusterEvent, EventType, Paginated};
use crate::domain::ports::KubernetesRepository;
use crate::error::Result;
use std::sync::Arc;

/// Get recent events use case
pub struct GetRecentEventsUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl GetRecentEventsUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        namespace: Option<&str>,
        event_type: Option<&str>,
        page: usize,
        per_page: usize,
    ) -> Result<Paginated<ClusterEvent>> {
        let events = self.repository.list_events(namespace, event_type).await?;
        
        let total = events.len();
        let start = (page - 1) * per_page;
        let end = (start + per_page).min(total);
        
        let items = if start < total {
            events[start..end].to_vec()
        } else {
            vec![]
        };
        
        Ok(Paginated::new(items, page, per_page, total))
    }
}

/// Get warning events use case (prioritized)
pub struct GetWarningEventsUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl GetWarningEventsUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        namespace: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ClusterEvent>> {
        let events = self.repository.list_events(namespace, Some("Warning")).await?;
        
        // Sort by last timestamp (most recent first)
        let mut sorted_events = events;
        sorted_events.sort_by(|a, b| b.last_timestamp.cmp(&a.last_timestamp));
        
        Ok(sorted_events.into_iter().take(limit).collect())
    }
}

/// Event statistics use case
pub struct GetEventStatsUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl GetEventStatsUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, namespace: Option<&str>) -> Result<EventStats> {
        let events = self.repository.list_events(namespace, None).await?;
        
        let total = events.len();
        let warnings = events.iter().filter(|e| e.event_type == EventType::Warning).count();
        let normals = events.iter().filter(|e| e.event_type == EventType::Normal).count();
        
        Ok(EventStats {
            total,
            warnings,
            normals,
        })
    }
}

/// Event statistics DTO
#[derive(Debug, Clone, serde::Serialize)]
pub struct EventStats {
    pub total: usize,
    pub warnings: usize,
    pub normals: usize,
}

/// Event service - aggregates all event use cases
pub struct EventService {
    pub get_recent: GetRecentEventsUseCase,
    pub get_warnings: GetWarningEventsUseCase,
    pub get_stats: GetEventStatsUseCase,
}

impl EventService {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self {
            get_recent: GetRecentEventsUseCase::new(repository.clone()),
            get_warnings: GetWarningEventsUseCase::new(repository.clone()),
            get_stats: GetEventStatsUseCase::new(repository),
        }
    }
}
