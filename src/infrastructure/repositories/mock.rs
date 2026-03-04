//! Mock and No-Op repository implementations for testing and offline modes
use async_trait::async_trait;
use crate::domain::entities::{BackupsResponse, ClusterInfo, NodeInfo};
use crate::domain::ports::{BackupRepository, ClusterRepository};
use crate::error::{KusanagiError, Result};

pub struct NoOpBackupRepository;

#[async_trait]
impl BackupRepository for NoOpBackupRepository {
    async fn get_backups_status(&self) -> Result<BackupsResponse> {
        Ok(BackupsResponse {
            total_cronjobs: 0,
            active_jobs: 0,
            succeeded_jobs: 0,
            failed_jobs: 0,
            cronjobs: vec![],
        })
    }

    async fn trigger_backup(&self, _namespace: &str, _name: &str) -> Result<String> {
        Err(KusanagiError::ExternalService(
            "Backup not available in offline mode".to_string(),
        ))
    }
}

pub struct MockClusterRepository;

#[async_trait]
impl ClusterRepository for MockClusterRepository {
    async fn get_cluster_info(&self) -> Result<ClusterInfo> {
        Ok(ClusterInfo {
            name: "kusanagi-mock-cluster".to_string(),
            version: "v1.28.0-mock".to_string(),
            status: "healthy".to_string(),
            nodes: 2,
        })
    }

    async fn get_nodes(&self) -> Result<Vec<NodeInfo>> {
        Ok(vec![
            NodeInfo {
                name: "mock-node-01".to_string(),
                status: "Ready".to_string(),
                role: "control-plane".to_string(),
            },
            NodeInfo {
                name: "mock-node-02".to_string(),
                status: "Ready".to_string(),
                role: "worker".to_string(),
            },
        ])
    }
}
