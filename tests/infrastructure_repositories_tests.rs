//! Tests for infrastructure layer repositories
//! Testing repository implementations with mocked external dependencies

use std::sync::Arc;
use tokio::sync::Mutex;

// ============================================================================
// Mock External Dependencies
// ============================================================================

/// Mock Kubernetes Client
#[derive(Clone)]
struct MockK8sClient {
    pods: Arc<Mutex<Vec<MockPod>>>,
    nodes: Arc<Mutex<Vec<MockNode>>>,
    services: Arc<Mutex<Vec<MockService>>>,
    should_fail: Arc<Mutex<bool>>,
}

#[derive(Clone, Debug)]
struct MockPod {
    name: String,
    namespace: String,
    status: String,
    phase: String,
}

#[derive(Clone, Debug)]
struct MockNode {
    name: String,
    status: String,
    role: String,
}

#[derive(Clone, Debug)]
struct MockService {
    name: String,
    namespace: String,
    cluster_ip: String,
}

impl MockK8sClient {
    fn new() -> Self {
        Self {
            pods: Arc::new(Mutex::new(vec![])),
            nodes: Arc::new(Mutex::new(vec![])),
            services: Arc::new(Mutex::new(vec![])),
            should_fail: Arc::new(Mutex::new(false)),
        }
    }

    async fn list_pods(&self, namespace: Option<&str>) -> Result<Vec<MockPod>, String> {
        if *self.should_fail.lock().await {
            return Err("Kubernetes API error".to_string());
        }

        let pods = self.pods.lock().await;
        Ok(match namespace {
            Some(ns) => pods.iter().filter(|p| p.namespace == ns).cloned().collect(),
            None => pods.clone(),
        })
    }

    async fn list_nodes(&self) -> Result<Vec<MockNode>, String> {
        if *self.should_fail.lock().await {
            return Err("Kubernetes API error".to_string());
        }
        Ok(self.nodes.lock().await.clone())
    }

    async fn list_services(&self, namespace: Option<&str>) -> Result<Vec<MockService>, String> {
        if *self.should_fail.lock().await {
            return Err("Kubernetes API error".to_string());
        }

        let services = self.services.lock().await;
        Ok(match namespace {
            Some(ns) => services
                .iter()
                .filter(|s| s.namespace == ns)
                .cloned()
                .collect(),
            None => services.clone(),
        })
    }

    async fn set_fail(&self, fail: bool) {
        *self.should_fail.lock().await = fail;
    }
}

// ============================================================================
// Repository Implementation
// ============================================================================

/// Kubernetes Repository
struct KubernetesRepository {
    client: Arc<MockK8sClient>,
}

impl KubernetesRepository {
    fn new(client: Arc<MockK8sClient>) -> Self {
        Self { client }
    }

    async fn get_all_pods(&self) -> Result<Vec<MockPod>, String> {
        self.client.list_pods(None).await
    }

    async fn get_pods_by_namespace(&self, namespace: &str) -> Result<Vec<MockPod>, String> {
        self.client.list_pods(Some(namespace)).await
    }

    async fn get_nodes(&self) -> Result<Vec<MockNode>, String> {
        self.client.list_nodes().await
    }

    async fn count_pods_by_status(&self) -> Result<PodStatusCounts, String> {
        let pods = self.client.list_pods(None).await?;

        let mut counts = PodStatusCounts::default();
        for pod in &pods {
            match pod.status.as_str() {
                "Running" => counts.running += 1,
                "Pending" => counts.pending += 1,
                "Failed" => counts.failed += 1,
                "Succeeded" => counts.succeeded += 1,
                _ => counts.unknown += 1,
            }
        }
        counts.total = pods.len();

        Ok(counts)
    }

    async fn get_cluster_summary(&self) -> Result<ClusterSummary, String> {
        let pods = self.client.list_pods(None).await?;
        let nodes = self.client.list_nodes().await?;
        let services = self.client.list_services(None).await?;

        Ok(ClusterSummary {
            total_pods: pods.len(),
            total_nodes: nodes.len(),
            total_services: services.len(),
        })
    }
}

#[derive(Default, Debug)]
struct PodStatusCounts {
    total: usize,
    running: usize,
    pending: usize,
    failed: usize,
    succeeded: usize,
    unknown: usize,
}

#[derive(Debug)]
struct ClusterSummary {
    total_pods: usize,
    total_nodes: usize,
    total_services: usize,
}

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn test_repository_get_all_pods() {
    let client = Arc::new(MockK8sClient::new());

    // Add test data
    client.pods.lock().await.push(MockPod {
        name: "pod-1".to_string(),
        namespace: "default".to_string(),
        status: "Running".to_string(),
        phase: "Running".to_string(),
    });
    client.pods.lock().await.push(MockPod {
        name: "pod-2".to_string(),
        namespace: "kube-system".to_string(),
        status: "Running".to_string(),
        phase: "Running".to_string(),
    });

    let repo = KubernetesRepository::new(client);
    let pods = repo.get_all_pods().await.unwrap();

    assert_eq!(pods.len(), 2);
}

#[tokio::test]
async fn test_repository_get_pods_by_namespace() {
    let client = Arc::new(MockK8sClient::new());

    client.pods.lock().await.push(MockPod {
        name: "pod-1".to_string(),
        namespace: "default".to_string(),
        status: "Running".to_string(),
        phase: "Running".to_string(),
    });
    client.pods.lock().await.push(MockPod {
        name: "pod-2".to_string(),
        namespace: "default".to_string(),
        status: "Running".to_string(),
        phase: "Running".to_string(),
    });
    client.pods.lock().await.push(MockPod {
        name: "pod-3".to_string(),
        namespace: "kube-system".to_string(),
        status: "Running".to_string(),
        phase: "Running".to_string(),
    });

    let repo = KubernetesRepository::new(client);
    let pods = repo.get_pods_by_namespace("default").await.unwrap();

    assert_eq!(pods.len(), 2);
}

#[tokio::test]
async fn test_repository_get_nodes() {
    let client = Arc::new(MockK8sClient::new());

    client.nodes.lock().await.push(MockNode {
        name: "master-1".to_string(),
        status: "Ready".to_string(),
        role: "master".to_string(),
    });
    client.nodes.lock().await.push(MockNode {
        name: "worker-1".to_string(),
        status: "Ready".to_string(),
        role: "worker".to_string(),
    });

    let repo = KubernetesRepository::new(client);
    let nodes = repo.get_nodes().await.unwrap();

    assert_eq!(nodes.len(), 2);
    assert!(nodes.iter().any(|n| n.role == "master"));
    assert!(nodes.iter().any(|n| n.role == "worker"));
}

#[tokio::test]
async fn test_repository_count_pods_by_status() {
    let client = Arc::new(MockK8sClient::new());

    client.pods.lock().await.extend(vec![
        MockPod {
            name: "p1".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            phase: "Running".to_string(),
        },
        MockPod {
            name: "p2".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            phase: "Running".to_string(),
        },
        MockPod {
            name: "p3".to_string(),
            namespace: "default".to_string(),
            status: "Pending".to_string(),
            phase: "Pending".to_string(),
        },
        MockPod {
            name: "p4".to_string(),
            namespace: "default".to_string(),
            status: "Failed".to_string(),
            phase: "Failed".to_string(),
        },
        MockPod {
            name: "p5".to_string(),
            namespace: "default".to_string(),
            status: "Succeeded".to_string(),
            phase: "Succeeded".to_string(),
        },
    ]);

    let repo = KubernetesRepository::new(client);
    let counts = repo.count_pods_by_status().await.unwrap();

    assert_eq!(counts.total, 5);
    assert_eq!(counts.running, 2);
    assert_eq!(counts.pending, 1);
    assert_eq!(counts.failed, 1);
    assert_eq!(counts.succeeded, 1);
    assert_eq!(counts.unknown, 0);
}

#[tokio::test]
async fn test_repository_get_cluster_summary() {
    let client = Arc::new(MockK8sClient::new());

    client.pods.lock().await.push(MockPod {
        name: "pod-1".to_string(),
        namespace: "default".to_string(),
        status: "Running".to_string(),
        phase: "Running".to_string(),
    });
    client.nodes.lock().await.push(MockNode {
        name: "node-1".to_string(),
        status: "Ready".to_string(),
        role: "master".to_string(),
    });
    client.services.lock().await.push(MockService {
        name: "svc-1".to_string(),
        namespace: "default".to_string(),
        cluster_ip: "10.0.0.1".to_string(),
    });

    let repo = KubernetesRepository::new(client);
    let summary = repo.get_cluster_summary().await.unwrap();

    assert_eq!(summary.total_pods, 1);
    assert_eq!(summary.total_nodes, 1);
    assert_eq!(summary.total_services, 1);
}

#[tokio::test]
async fn test_repository_error_handling() {
    let client = Arc::new(MockK8sClient::new());
    client.set_fail(true).await;

    let repo = KubernetesRepository::new(client);

    let result = repo.get_all_pods().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Kubernetes API error"));

    let result = repo.get_nodes().await;
    assert!(result.is_err());

    let result = repo.count_pods_by_status().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_repository_empty_results() {
    let client = Arc::new(MockK8sClient::new());
    let repo = KubernetesRepository::new(client);

    let pods = repo.get_all_pods().await.unwrap();
    assert!(pods.is_empty());

    let nodes = repo.get_nodes().await.unwrap();
    assert!(nodes.is_empty());

    let counts = repo.count_pods_by_status().await.unwrap();
    assert_eq!(counts.total, 0);
}

// ============================================================================
// Repository Pattern Tests
// ============================================================================

/// Generic Repository Port
trait Repository<T, E>: Send + Sync {
    fn find_all(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<T>, E>> + Send + '_>>;
    fn find_by_id(
        &self,
        id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<T>, E>> + Send + '_>>;
    fn save(
        &self,
        item: T,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), E>> + Send + '_>>;
    fn delete(
        &self,
        id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), E>> + Send + '_>>;
}

/// In-Memory Repository Implementation
struct InMemoryRepository<T: Clone + Send + Sync> {
    data: Arc<Mutex<Vec<T>>>,
    id_extractor: fn(&T) -> String,
}

impl<T: Clone + Send + Sync + 'static> InMemoryRepository<T> {
    fn new(id_extractor: fn(&T) -> String) -> Self {
        Self {
            data: Arc::new(Mutex::new(vec![])),
            id_extractor,
        }
    }
}

impl<T: Clone + Send + Sync + 'static> Repository<T, String> for InMemoryRepository<T> {
    fn find_all(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<T>, String>> + Send + '_>>
    {
        let data = self.data.clone();
        Box::pin(async move { Ok(data.lock().await.clone()) })
    }

    fn find_by_id(
        &self,
        id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<T>, String>> + Send + '_>>
    {
        let data = self.data.clone();
        let id = id.to_string();
        let id_extractor = self.id_extractor;
        Box::pin(async move {
            let data = data.lock().await;
            Ok(data.iter().find(|item| id_extractor(item) == id).cloned())
        })
    }

    fn save(
        &self,
        item: T,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        let data = self.data.clone();
        let id_extractor = self.id_extractor;
        Box::pin(async move {
            let mut data = data.lock().await;
            let id = id_extractor(&item);
            if let Some(index) = data.iter().position(|i| id_extractor(i) == id) {
                data[index] = item;
            } else {
                data.push(item);
            }
            Ok(())
        })
    }

    fn delete(
        &self,
        id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        let data = self.data.clone();
        let id = id.to_string();
        let id_extractor = self.id_extractor;
        Box::pin(async move {
            let mut data = data.lock().await;
            let initial_len = data.len();
            data.retain(|item| id_extractor(item) != id);
            if data.len() == initial_len {
                Err("Item not found".to_string())
            } else {
                Ok(())
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
struct TestEntity {
    id: String,
    name: String,
    value: i32,
}

fn extract_id(entity: &TestEntity) -> String {
    entity.id.clone()
}

#[tokio::test]
async fn test_generic_repository_crud() {
    let repo = InMemoryRepository::new(extract_id);

    // Create
    let entity = TestEntity {
        id: "1".to_string(),
        name: "Test".to_string(),
        value: 42,
    };
    repo.save(entity.clone()).await.unwrap();

    // Read
    let found = repo.find_by_id("1").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Test");

    // Update
    let updated = TestEntity {
        id: "1".to_string(),
        name: "Updated".to_string(),
        value: 100,
    };
    repo.save(updated).await.unwrap();

    let found = repo.find_by_id("1").await.unwrap();
    assert_eq!(found.unwrap().name, "Updated");

    // Delete
    repo.delete("1").await.unwrap();
    let found = repo.find_by_id("1").await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_generic_repository_find_all() {
    let repo = InMemoryRepository::new(extract_id);

    repo.save(TestEntity {
        id: "1".to_string(),
        name: "A".to_string(),
        value: 1,
    })
    .await
    .unwrap();
    repo.save(TestEntity {
        id: "2".to_string(),
        name: "B".to_string(),
        value: 2,
    })
    .await
    .unwrap();
    repo.save(TestEntity {
        id: "3".to_string(),
        name: "C".to_string(),
        value: 3,
    })
    .await
    .unwrap();

    let all = repo.find_all().await.unwrap();
    assert_eq!(all.len(), 3);
}

#[tokio::test]
async fn test_generic_repository_delete_not_found() {
    let repo = InMemoryRepository::new(extract_id);

    let result = repo.delete("nonexistent").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}
