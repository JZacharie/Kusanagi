//! Application State for Axum
//!
//! Centralized state management for all handlers.

use std::sync::Arc;

use crate::{
    application::use_cases::{
        GetAlertsUseCase, GetHomeAssistantUseCase, GetSecurityUseCase, GetWeatherUseCase,
    },
    AdvancedCache, Config,
};

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    /// Weather use case
    pub weather_use_case: Arc<GetWeatherUseCase>,
    /// Alerts use case
    pub alerts_use_case: Arc<GetAlertsUseCase>,
    /// Security use case
    pub security_use_case: Arc<GetSecurityUseCase>,
    /// HomeAssistant use case
    pub ha_use_case: Arc<GetHomeAssistantUseCase>,
    /// Kubernetes cache (for services, ingress)
    pub k8s_cache: Arc<AdvancedCache<String>>,
    /// ArgoCD cache
    pub argocd_cache: Arc<AdvancedCache<String>>,
    /// General cache
    pub general_cache: Arc<AdvancedCache<String>>,
    /// Application configuration
    pub config: Config,
    /// HTTP client for external APIs
    pub http_client: reqwest::Client,
}

impl AppState {
    /// Create a new application state with all dependencies
    pub async fn new(
        weather_use_case: Arc<GetWeatherUseCase>,
        alerts_use_case: Arc<GetAlertsUseCase>,
        security_use_case: Arc<GetSecurityUseCase>,
        ha_use_case: Arc<GetHomeAssistantUseCase>,
        k8s_cache: Arc<AdvancedCache<String>>,
        argocd_cache: Arc<AdvancedCache<String>>,
        general_cache: Arc<AdvancedCache<String>>,
        config: Config,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            weather_use_case,
            alerts_use_case,
            security_use_case,
            ha_use_case,
            k8s_cache,
            argocd_cache,
            general_cache,
            config,
            http_client,
        }
    }
}
