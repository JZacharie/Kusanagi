use serde::{Deserialize, Serialize};
use actix_web::{HttpResponse, Responder};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuotaMetrics {
    pub antigravity_percentage: u8,
    pub notebooklm_percentage: u8,
    pub storage_used_gb: f32,
    pub storage_total_gb: f32,
    pub last_updated: String,
}

impl QuotaMetrics {
    pub fn mock() -> Self {
        // In the future, this could read from a config file or env vars
        QuotaMetrics {
            antigravity_percentage: 42, // The answer to everything
            notebooklm_percentage: 15,
            storage_used_gb: 154.5,
            storage_total_gb: 2048.0, // 2TB plan
            last_updated: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

pub async fn get_quotas() -> impl Responder {
    let metrics = QuotaMetrics::mock();
    HttpResponse::Ok().json(&metrics)
}
