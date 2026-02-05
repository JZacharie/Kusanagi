use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Clone)]
pub struct PreloadCache {
    argocd_data: Arc<RwLock<Option<serde_json::Value>>>,
    proxmox_data: Arc<RwLock<Option<serde_json::Value>>>,
    weather_data: Arc<RwLock<Option<serde_json::Value>>>,
    last_update: Arc<RwLock<HashMap<String, chrono::DateTime<chrono::Utc>>>>,
}

impl PreloadCache {
    pub fn new() -> Self {
        Self {
            argocd_data: Arc::new(RwLock::new(None)),
            proxmox_data: Arc::new(RwLock::new(None)),
            weather_data: Arc::new(RwLock::new(None)),
            last_update: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn preload_argocd(&self) -> serde_json::Value {
        let mut data = self.argocd_data.write().await;
        let mut updates = self.last_update.write().await;
        
        let mock_data = json!({
            "applications": [
                {
                    "name": "nginx-app",
                    "namespace": "argocd",
                    "status": {
                        "sync": "Synced",
                        "health": "Healthy"
                    },
                    "source": {
                        "repoURL": "https://github.com/company/k8s-manifests",
                        "path": "apps/nginx",
                        "targetRevision": "HEAD"
                    },
                    "destination": {
                        "server": "https://kubernetes.default.svc",
                        "namespace": "production"
                    }
                },
                {
                    "name": "redis-cluster",
                    "namespace": "argocd", 
                    "status": {
                        "sync": "OutOfSync",
                        "health": "Progressing"
                    },
                    "source": {
                        "repoURL": "https://github.com/company/helm-charts",
                        "path": "redis",
                        "targetRevision": "v6.2.0"
                    },
                    "destination": {
                        "server": "https://kubernetes.default.svc",
                        "namespace": "database"
                    }
                }
            ],
            "summary": {
                "total": 2,
                "synced": 1,
                "out_of_sync": 1,
                "healthy": 1,
                "progressing": 1
            }
        });
        
        *data = Some(mock_data.clone());
        updates.insert("argocd".to_string(), chrono::Utc::now());
        mock_data
    }

    pub async fn preload_proxmox(&self) -> serde_json::Value {
        let mut data = self.proxmox_data.write().await;
        let mut updates = self.last_update.write().await;
        
        let mock_data = json!({
            "cluster": {
                "name": "pve-cluster",
                "status": "online",
                "nodes": 3,
                "version": "8.1.3"
            },
            "vms": [
                {
                    "vmid": 100,
                    "name": "k8s-master-01",
                    "status": "running",
                    "cpu": 4,
                    "memory": 8192,
                    "disk": 100,
                    "uptime": 2592000,
                    "node": "pve-01"
                },
                {
                    "vmid": 101,
                    "name": "k8s-worker-01", 
                    "status": "running",
                    "cpu": 8,
                    "memory": 16384,
                    "disk": 200,
                    "uptime": 2592000,
                    "node": "pve-02"
                }
            ],
            "containers": [
                {
                    "vmid": 200,
                    "name": "monitoring-ct",
                    "status": "running",
                    "cpu": 2,
                    "memory": 4096,
                    "disk": 50,
                    "uptime": 1296000,
                    "node": "pve-03"
                }
            ],
            "resources": {
                "cpu_usage": 45.2,
                "memory_usage": 62.8,
                "storage_usage": 38.5
            }
        });
        
        *data = Some(mock_data.clone());
        updates.insert("proxmox".to_string(), chrono::Utc::now());
        mock_data
    }

    pub async fn preload_weather(&self) -> serde_json::Value {
        let mut data = self.weather_data.write().await;
        let mut updates = self.last_update.write().await;
        
        let mock_data = json!({
            "current": {
                "location": "Paris, France",
                "temperature": 12.5,
                "humidity": 68,
                "pressure": 1013.2,
                "wind_speed": 15.3,
                "wind_direction": "SW",
                "condition": "Partly Cloudy",
                "visibility": 10.0,
                "uv_index": 3
            },
            "forecast": [
                {
                    "date": "2026-02-05",
                    "temp_min": 8,
                    "temp_max": 15,
                    "condition": "Cloudy",
                    "precipitation": 20
                },
                {
                    "date": "2026-02-06", 
                    "temp_min": 10,
                    "temp_max": 17,
                    "condition": "Sunny",
                    "precipitation": 0
                },
                {
                    "date": "2026-02-07",
                    "temp_min": 9,
                    "temp_max": 16,
                    "condition": "Rain",
                    "precipitation": 80
                }
            ],
            "alerts": [
                {
                    "type": "wind",
                    "severity": "moderate",
                    "message": "Strong winds expected this afternoon"
                }
            ]
        });
        
        *data = Some(mock_data.clone());
        updates.insert("weather".to_string(), chrono::Utc::now());
        mock_data
    }

    pub async fn get_argocd(&self) -> Option<serde_json::Value> {
        self.argocd_data.read().await.clone()
    }

    pub async fn get_proxmox(&self) -> Option<serde_json::Value> {
        self.proxmox_data.read().await.clone()
    }

    pub async fn get_weather(&self) -> Option<serde_json::Value> {
        self.weather_data.read().await.clone()
    }

    pub async fn refresh_all(&self) {
        tokio::join!(
            self.preload_argocd(),
            self.preload_proxmox(), 
            self.preload_weather()
        );
    }

    pub async fn get_cache_status(&self) -> serde_json::Value {
        let updates = self.last_update.read().await;
        json!({
            "argocd": {
                "cached": self.argocd_data.read().await.is_some(),
                "last_update": updates.get("argocd")
            },
            "proxmox": {
                "cached": self.proxmox_data.read().await.is_some(),
                "last_update": updates.get("proxmox")
            },
            "weather": {
                "cached": self.weather_data.read().await.is_some(),
                "last_update": updates.get("weather")
            }
        })
    }
}
