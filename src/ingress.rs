use kube::{Client, Api, api::ListParams};
use k8s_openapi::api::networking::v1::Ingress;
use serde::Serialize;
use chrono::Utc;

#[derive(Serialize)]
pub struct IngressInfo {
    pub name: String,
    pub namespace: String,
    pub load_balancer: Option<String>,
    pub rules: Vec<String>,
    pub age: String,
}

pub async fn get_ingresses() -> Result<Vec<IngressInfo>, String> {
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    let ingresses: Api<Ingress> = Api::all(client);
    let list = ingresses.list(&ListParams::default()).await.map_err(|e| e.to_string())?;

    let mut ingress_infos = Vec::new();

    for ing in list {
        let name = ing.metadata.name.clone().unwrap_or_default();
        let namespace = ing.metadata.namespace.clone().unwrap_or_default();
        
        let load_balancer = if let Some(status) = ing.status {
             if let Some(lb) = status.load_balancer {
                if let Some(ingress) = lb.ingress {
                    ingress.first().and_then(|i| i.ip.clone().or(i.hostname.clone()))
                } else {
                    None
                }
             } else {
                 None
             }
        } else {
            None
        };

        let rules = if let Some(spec) = ing.spec {
            spec.rules.unwrap_or_default().iter().flat_map(|rule| {
                let host = rule.host.clone().unwrap_or("*".to_string());
                if let Some(http) = &rule.http {
                    http.paths.iter().map(|path| {
                         format!("{}{}", host, path.path.clone().unwrap_or("".to_string()))
                    }).collect::<Vec<_>>()
                } else {
                    vec![host]
                }
            }).collect()
        } else {
            Vec::new()
        };

        let creation_timestamp = ing.metadata.creation_timestamp.map(|t| t.0).unwrap_or(Utc::now());
        let duration = Utc::now().signed_duration_since(creation_timestamp);
        let age = if duration.num_days() > 0 {
            format!("{}d", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{}h", duration.num_hours())
        } else {
            format!("{}m", duration.num_minutes())
        };

        ingress_infos.push(IngressInfo {
            name,
            namespace,
            load_balancer,
            rules,
            age,
        });
    }

    Ok(ingress_infos)
}
