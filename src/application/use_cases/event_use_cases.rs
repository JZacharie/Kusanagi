//! Event-specific use cases

use crate::application::dtos::*;
use crate::application::mappers::*;
use crate::domain::entities::*;
use crate::domain::ports::*;
use crate::error::Result;
use std::sync::Arc;

/// Use case: Get recent events with pagination
pub struct GetRecentEventsUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl GetRecentEventsUseCase {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    pub async fn execute(
        &self,
        event_type: Option<&str>,
        page: usize,
        per_page: usize,
    ) -> Result<PaginatedResponse<EventDto>> {
        let events = self.k8s_repo.list_events(None, event_type).await?;
        
        let total = events.len();
        let total_pages = if total == 0 { 1 } else { (total + per_page - 1) / per_page };
        
        // Paginate
        let start = (page.saturating_sub(1)) * per_page;
        let paginated_events: Vec<ClusterEvent> = events
            .into_iter()
            .skip(start)
            .take(per_page)
            .collect();
        
        let dtos = EventMapper::to_dto_list(paginated_events);
        
        Ok(PaginatedResponse {
            items: dtos,
            page,
            per_page,
            total,
            total_pages,
        })
    }
}

/// Use case: Get warning events summary
pub struct GetWarningSummaryUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl GetWarningSummaryUseCase {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    pub async fn execute(&self) -> Result<WarningSummaryDto> {
        let events = self.k8s_repo.list_events(None, Some("Warning")).await?;
        
        // Group by reason
        let mut reason_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for event in &events {
            *reason_counts.entry(event.reason.clone()).or_insert(0) += 1;
        }
        
        // Get top reasons
        let mut top_reasons: Vec<(String, usize)> = reason_counts.into_iter().collect();
        top_reasons.sort_by(|a, b| b.1.cmp(&a.1));
        top_reasons.truncate(5);
        
        let top_issues: Vec<String> = top_reasons
            .into_iter()
            .map(|(reason, count)| format!("{} ({} occurrences)", reason, count))
            .collect();
        
        Ok(WarningSummaryDto {
            total_warnings: events.len(),
            top_issues,
            severity: if events.len() > 50 {
                "High".to_string()
            } else if events.len() > 10 {
                "Medium".to_string()
            } else {
                "Low".to_string()
            },
        })
    }
}

/// DTO for warning summary
#[derive(Debug, Clone, serde::Serialize)]
pub struct WarningSummaryDto {
    pub total_warnings: usize,
    pub top_issues: Vec<String>,
    pub severity: String,
}

/// Use case: Get events for a specific resource
pub struct GetResourceEventsUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl GetResourceEventsUseCase {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    pub async fn execute(
        &self,
        kind: &str,
        name: &str,
        namespace: Option<&str>,
    ) -> Result<Vec<EventDto>> {
        let events = self.k8s_repo.list_events(namespace, None).await?;
        
        let filtered: Vec<ClusterEvent> = events
            .into_iter()
            .filter(|e| {
                e.involved_object.kind == kind && e.involved_object.name == name
            })
            .collect();
        
        Ok(EventMapper::to_dto_list(filtered))
    }
}

/// Use case: Export events for remediation
pub struct ExportEventsUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl ExportEventsUseCase {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    pub async fn execute(&self, language: &str) -> Result<String> {
        let events = self.k8s_repo.list_events(None, Some("Warning")).await?;
        
        // Generate markdown report
        let mut report = format!(
            "# {}\n\n",
            if language == "fr" { "Rapport d'Événements" } else { "Events Report" }
        );
        
        report.push_str(&format!("Generated: {}\n\n", chrono::Utc::now().to_rfc3339()));
        report.push_str(&format!("Total Warnings: {}\n\n", events.len()));
        
        report.push_str("## Recent Events\n\n");
        for event in events.iter().take(20) {
            report.push_str(&format!(
                "- **{}** ({}) - {}\n  - {}\n\n",
                event.reason,
                event.namespace,
                event.involved_object.name,
                event.message
            ));
        }
        
        Ok(report)
    }
}
