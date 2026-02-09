//! Tests for ArgoCD service

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// Mock ArgoCD types
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArgoCdApplication {
    name: String,
    namespace: String,
    sync_status: String,
    health_status: String,
    target_revision: String,
}

#[derive(Debug, Clone)]
struct ArgoCdRepository {
    applications: Arc<Mutex<Vec<ArgoCdApplication>>>,
}

impl ArgoCdRepository {
    fn new() -> Self {
        Self {
            applications: Arc::new(Mutex::new(vec![])),
        }
    }

    async fn list_applications(&self) -> Vec<ArgoCdApplication> {
        self.applications.lock().await.clone()
    }

    async fn get_application(&self, name: &str) -> Option<ArgoCdApplication> {
        self.applications
            .lock()
            .await
            .iter()
            .find(|app| app.name == name)
            .cloned()
    }

    async fn add_application(&self, app: ArgoCdApplication) {
        self.applications.lock().await.push(app);
    }

    fn sync_application(
        &self,
        name: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        let apps = self.applications.clone();
        let name = name.to_string();
        Box::pin(async move {
            let mut apps = apps.lock().await;
            if let Some(app) = apps.iter_mut().find(|a| a.name == name) {
                app.sync_status = "Synced".to_string();
                Ok(())
            } else {
                Err(format!("Application {} not found", name))
            }
        })
    }
}

// Service layer
struct ArgoCdService {
    repository: Arc<ArgoCdRepository>,
}

impl ArgoCdService {
    fn new(repository: Arc<ArgoCdRepository>) -> Self {
        Self { repository }
    }

    async fn get_application_summary(&self) -> ApplicationSummary {
        let apps = self.repository.list_applications().await;

        let total = apps.len();
        let synced = apps.iter().filter(|a| a.sync_status == "Synced").count();
        let healthy = apps.iter().filter(|a| a.health_status == "Healthy").count();
        let out_of_sync = apps.iter().filter(|a| a.sync_status != "Synced").count();

        ApplicationSummary {
            total,
            synced,
            healthy,
            out_of_sync,
            sync_percentage: if total > 0 {
                (synced as f64 / total as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    async fn get_out_of_sync_apps(&self) -> Vec<ArgoCdApplication> {
        self.repository
            .list_applications()
            .await
            .into_iter()
            .filter(|a| a.sync_status != "Synced")
            .collect()
    }

    async fn get_apps_by_health(&self, health: &str) -> Vec<ArgoCdApplication> {
        self.repository
            .list_applications()
            .await
            .into_iter()
            .filter(|a| a.health_status == health)
            .collect()
    }

    async fn sync_all(&self) -> SyncResult {
        let apps = self.repository.list_applications().await;
        let mut success = 0;
        let mut failed = 0;

        for app in apps {
            if let Err(_) = self.repository.sync_application(&app.name).await {
                failed += 1;
            } else {
                success += 1;
            }
        }

        SyncResult { success, failed }
    }
}

#[derive(Debug, Clone)]
struct ApplicationSummary {
    total: usize,
    synced: usize,
    healthy: usize,
    out_of_sync: usize,
    sync_percentage: f64,
}

#[derive(Debug, Clone)]
struct SyncResult {
    success: usize,
    failed: usize,
}

#[tokio::test]
async fn test_get_application_summary_empty() {
    let repo = Arc::new(ArgoCdRepository::new());
    let service = ArgoCdService::new(repo);

    let summary = service.get_application_summary().await;

    assert_eq!(summary.total, 0);
    assert_eq!(summary.synced, 0);
    assert_eq!(summary.sync_percentage, 0.0);
}

#[tokio::test]
async fn test_get_application_summary_with_apps() {
    let repo = Arc::new(ArgoCdRepository::new());

    repo.add_application(ArgoCdApplication {
        name: "app-1".to_string(),
        namespace: "argocd".to_string(),
        sync_status: "Synced".to_string(),
        health_status: "Healthy".to_string(),
        target_revision: "main".to_string(),
    })
    .await;

    repo.add_application(ArgoCdApplication {
        name: "app-2".to_string(),
        namespace: "argocd".to_string(),
        sync_status: "Synced".to_string(),
        health_status: "Healthy".to_string(),
        target_revision: "v1.0".to_string(),
    })
    .await;

    repo.add_application(ArgoCdApplication {
        name: "app-3".to_string(),
        namespace: "argocd".to_string(),
        sync_status: "OutOfSync".to_string(),
        health_status: "Degraded".to_string(),
        target_revision: "main".to_string(),
    })
    .await;

    let service = ArgoCdService::new(repo);
    let summary = service.get_application_summary().await;

    assert_eq!(summary.total, 3);
    assert_eq!(summary.synced, 2);
    assert_eq!(summary.healthy, 2);
    assert_eq!(summary.out_of_sync, 1);
    assert!((summary.sync_percentage - 66.67).abs() < 0.1);
}

#[tokio::test]
async fn test_get_out_of_sync_apps() {
    let repo = Arc::new(ArgoCdRepository::new());

    repo.add_application(ArgoCdApplication {
        name: "synced-app".to_string(),
        namespace: "argocd".to_string(),
        sync_status: "Synced".to_string(),
        health_status: "Healthy".to_string(),
        target_revision: "main".to_string(),
    })
    .await;

    repo.add_application(ArgoCdApplication {
        name: "out-of-sync-app".to_string(),
        namespace: "argocd".to_string(),
        sync_status: "OutOfSync".to_string(),
        health_status: "Degraded".to_string(),
        target_revision: "main".to_string(),
    })
    .await;

    let service = ArgoCdService::new(repo);
    let out_of_sync = service.get_out_of_sync_apps().await;

    assert_eq!(out_of_sync.len(), 1);
    assert_eq!(out_of_sync[0].name, "out-of-sync-app");
}

#[tokio::test]
async fn test_get_apps_by_health() {
    let repo = Arc::new(ArgoCdRepository::new());

    repo.add_application(ArgoCdApplication {
        name: "healthy-1".to_string(),
        namespace: "argocd".to_string(),
        sync_status: "Synced".to_string(),
        health_status: "Healthy".to_string(),
        target_revision: "main".to_string(),
    })
    .await;

    repo.add_application(ArgoCdApplication {
        name: "healthy-2".to_string(),
        namespace: "argocd".to_string(),
        sync_status: "Synced".to_string(),
        health_status: "Healthy".to_string(),
        target_revision: "main".to_string(),
    })
    .await;

    repo.add_application(ArgoCdApplication {
        name: "degraded-1".to_string(),
        namespace: "argocd".to_string(),
        sync_status: "Synced".to_string(),
        health_status: "Degraded".to_string(),
        target_revision: "main".to_string(),
    })
    .await;

    let service = ArgoCdService::new(repo);
    let healthy_apps = service.get_apps_by_health("Healthy").await;

    assert_eq!(healthy_apps.len(), 2);
    assert!(healthy_apps.iter().all(|a| a.health_status == "Healthy"));
}

#[tokio::test]
async fn test_sync_all() {
    let repo = Arc::new(ArgoCdRepository::new());

    repo.add_application(ArgoCdApplication {
        name: "app-1".to_string(),
        namespace: "argocd".to_string(),
        sync_status: "OutOfSync".to_string(),
        health_status: "Degraded".to_string(),
        target_revision: "main".to_string(),
    })
    .await;

    repo.add_application(ArgoCdApplication {
        name: "app-2".to_string(),
        namespace: "argocd".to_string(),
        sync_status: "OutOfSync".to_string(),
        health_status: "Degraded".to_string(),
        target_revision: "main".to_string(),
    })
    .await;

    let service = ArgoCdService::new(repo.clone());
    let result = service.sync_all().await;

    assert_eq!(result.success, 2);
    assert_eq!(result.failed, 0);

    // Verify apps are now synced
    let apps = repo.list_applications().await;
    assert!(apps.iter().all(|a| a.sync_status == "Synced"));
}

#[tokio::test]
async fn test_sync_single_app() {
    let repo = Arc::new(ArgoCdRepository::new());

    repo.add_application(ArgoCdApplication {
        name: "app-1".to_string(),
        namespace: "argocd".to_string(),
        sync_status: "OutOfSync".to_string(),
        health_status: "Degraded".to_string(),
        target_revision: "main".to_string(),
    })
    .await;

    let result = repo.sync_application("app-1").await;
    assert!(result.is_ok());

    let app = repo.get_application("app-1").await.unwrap();
    assert_eq!(app.sync_status, "Synced");
}

#[tokio::test]
async fn test_sync_nonexistent_app() {
    let repo = Arc::new(ArgoCdRepository::new());

    let result = repo.sync_application("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_application() {
    let repo = Arc::new(ArgoCdRepository::new());

    let app = ArgoCdApplication {
        name: "my-app".to_string(),
        namespace: "argocd".to_string(),
        sync_status: "Synced".to_string(),
        health_status: "Healthy".to_string(),
        target_revision: "main".to_string(),
    };

    repo.add_application(app.clone()).await;

    let found = repo.get_application("my-app").await;
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "my-app");

    let not_found = repo.get_application("unknown").await;
    assert!(not_found.is_none());
}
