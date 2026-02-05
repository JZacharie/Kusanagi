// Legacy ingress module - minimal
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IngressInfo {
    pub name: String,
    pub namespace: String,
    pub hosts: Vec<String>,
    pub tls: bool,
    pub class: String,
}

pub async fn get_ingresses() -> Result<Vec<IngressInfo>, Box<dyn std::error::Error>> {
    Ok(vec![
        IngressInfo {
            name: "legacy-api-ingress".to_string(),
            namespace: "legacy-system".to_string(),
            hosts: vec!["api.legacy.local".to_string()],
            tls: true,
            class: "nginx".to_string(),
        },
        IngressInfo {
            name: "legacy-web-ingress".to_string(),
            namespace: "legacy-system".to_string(),
            hosts: vec!["web.legacy.local".to_string()],
            tls: false,
            class: "traefik".to_string(),
        },
    ])
}
