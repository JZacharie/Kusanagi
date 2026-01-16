use kube::{Client, Api, api::ListParams};
use k8s_openapi::api::core::v1::Service;
use serde::Serialize;
use chrono::Utc;

#[derive(Serialize)]
pub struct ServiceInfo {
    pub name: String,
    pub namespace: String,
    pub type_: String,
    pub cluster_ip: String,
    pub external_ip: Option<String>,
    pub ports: String,
    pub age: String,
}

pub async fn get_services() -> Result<Vec<ServiceInfo>, String> {
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    let services: Api<Service> = Api::all(client);
    let list = services.list(&ListParams::default()).await.map_err(|e| e.to_string())?;

    let mut service_infos = Vec::new();

    for svc in list {
        let name = svc.metadata.name.clone().unwrap_or_default();
        let namespace = svc.metadata.namespace.clone().unwrap_or_default();
        let spec = svc.spec.unwrap_or_default();
        let type_ = spec.type_.unwrap_or_default();
        let cluster_ip = spec.cluster_ip.unwrap_or_default();
        
        let external_ip = if let Some(status) = svc.status {
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

        let ports = spec.ports.unwrap_or_default()
            .iter()
            .map(|p| {
                let target = p.target_port.clone().map_or("".to_string(), |t| match t {
                    k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(i) => i.to_string(),
                    k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::String(s) => s,
                });
                format!("{}:{}/{}", p.port, target, p.protocol.clone().unwrap_or_default())
            })
            .collect::<Vec<_>>()
            .join(", ");

        let creation_timestamp = svc.metadata.creation_timestamp.map(|t| t.0).unwrap_or(Utc::now());
        let duration = Utc::now().signed_duration_since(creation_timestamp);
        let age = if duration.num_days() > 0 {
            format!("{}d", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{}h", duration.num_hours())
        } else {
            format!("{}m", duration.num_minutes())
        };

        service_infos.push(ServiceInfo {
            name,
            namespace,
            type_,
            cluster_ip,
            external_ip,
            ports,
            age,
        });
    }

    Ok(service_infos)
}
