//! Tests for domain services
//! These tests use mocks to avoid external dependencies

use std::sync::Arc;
use tokio::sync::Mutex;

// Mock structures for testing
#[derive(Debug, Clone)]
struct MockClusterData {
    pods: Vec<MockPod>,
    nodes: Vec<MockNode>,
}

#[derive(Debug, Clone)]
struct MockPod {
    name: String,
    namespace: String,
    status: String,
    restart_count: i32,
}

#[derive(Debug, Clone)]
struct MockNode {
    name: String,
    status: String,
    role: String,
}

trait MockKubernetesRepository: Send + Sync {
    fn get_pods(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<MockPod>, String>> + Send + '_>>;
    fn get_nodes(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<MockNode>, String>> + Send + '_>>;
}

struct MockK8sRepo {
    data: Arc<Mutex<MockClusterData>>,
}

impl MockKubernetesRepository for MockK8sRepo {
    fn get_pods(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<MockPod>, String>> + Send + '_>> {
        let data = self.data.clone();
        Box::pin(async move {
            let data = data.lock().await;
            Ok(data.pods.clone())
        })
    }

    fn get_nodes(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<MockNode>, String>> + Send + '_>> {
        let data = self.data.clone();
        Box::pin(async move {
            let data = data.lock().await;
            Ok(data.nodes.clone())
        })
    }
}

// Service under test
struct PodService<R: MockKubernetesRepository> {
    repository: Arc<R>,
}

impl<R: MockKubernetesRepository> PodService<R> {
    fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    async fn count_pods_by_status(&self) -> Result<(usize, usize, usize), String> {
        let pods = self.repository.get_pods().await?;
        
        let running = pods.iter().filter(|p| p.status == "Running").count();
        let pending = pods.iter().filter(|p| p.status == "Pending").count();
        let failed = pods.iter().filter(|p| p.status == "Failed").count();
        
        Ok((running, pending, failed))
    }

    async fn get_problematic_pods(&self) -> Result<Vec<MockPod>, String> {
        let pods = self.repository.get_pods().await?;
        
        let problematic: Vec<MockPod> = pods
            .into_iter()
            .filter(|p| {
                p.status == "Failed" 
                    || p.status == "CrashLoopBackOff"
                    || p.restart_count > 5
            })
            .collect();
        
        Ok(problematic)
    }

    async fn get_pods_in_namespace(&self, namespace: &str) -> Result<Vec<MockPod>, String> {
        let pods = self.repository.get_pods().await?;
        
        let filtered: Vec<MockPod> = pods
            .into_iter()
            .filter(|p| p.namespace == namespace)
            .collect();
        
        Ok(filtered)
    }
}

#[tokio::test]
async fn test_count_pods_by_status() {
    let data = MockClusterData {
        pods: vec![
            MockPod { name: "pod-1".to_string(), namespace: "default".to_string(), status: "Running".to_string(), restart_count: 0 },
            MockPod { name: "pod-2".to_string(), namespace: "default".to_string(), status: "Running".to_string(), restart_count: 0 },
            MockPod { name: "pod-3".to_string(), namespace: "default".to_string(), status: "Pending".to_string(), restart_count: 0 },
            MockPod { name: "pod-4".to_string(), namespace: "default".to_string(), status: "Failed".to_string(), restart_count: 3 },
        ],
        nodes: vec![],
    };

    let repo = MockK8sRepo { data: Arc::new(Mutex::new(data)) };
    let service = PodService::new(Arc::new(repo));

    let (running, pending, failed) = service.count_pods_by_status().await.unwrap();

    assert_eq!(running, 2);
    assert_eq!(pending, 1);
    assert_eq!(failed, 1);
}

#[tokio::test]
async fn test_get_problematic_pods() {
    let data = MockClusterData {
        pods: vec![
            MockPod { name: "healthy-1".to_string(), namespace: "default".to_string(), status: "Running".to_string(), restart_count: 0 },
            MockPod { name: "failed-1".to_string(), namespace: "default".to_string(), status: "Failed".to_string(), restart_count: 0 },
            MockPod { name: "crashloop-1".to_string(), namespace: "default".to_string(), status: "CrashLoopBackOff".to_string(), restart_count: 0 },
            MockPod { name: "restarting-1".to_string(), namespace: "default".to_string(), status: "Running".to_string(), restart_count: 10 },
        ],
        nodes: vec![],
    };

    let repo = MockK8sRepo { data: Arc::new(Mutex::new(data)) };
    let service = PodService::new(Arc::new(repo));

    let problematic = service.get_problematic_pods().await.unwrap();

    assert_eq!(problematic.len(), 3);
    assert!(problematic.iter().any(|p| p.name == "failed-1"));
    assert!(problematic.iter().any(|p| p.name == "crashloop-1"));
    assert!(problematic.iter().any(|p| p.name == "restarting-1"));
}

#[tokio::test]
async fn test_get_pods_in_namespace() {
    let data = MockClusterData {
        pods: vec![
            MockPod { name: "pod-1".to_string(), namespace: "default".to_string(), status: "Running".to_string(), restart_count: 0 },
            MockPod { name: "pod-2".to_string(), namespace: "kube-system".to_string(), status: "Running".to_string(), restart_count: 0 },
            MockPod { name: "pod-3".to_string(), namespace: "default".to_string(), status: "Running".to_string(), restart_count: 0 },
        ],
        nodes: vec![],
    };

    let repo = MockK8sRepo { data: Arc::new(Mutex::new(data)) };
    let service = PodService::new(Arc::new(repo));

    let default_pods = service.get_pods_in_namespace("default").await.unwrap();
    assert_eq!(default_pods.len(), 2);

    let kube_system_pods = service.get_pods_in_namespace("kube-system").await.unwrap();
    assert_eq!(kube_system_pods.len(), 1);

    let empty_pods = service.get_pods_in_namespace("nonexistent").await.unwrap();
    assert!(empty_pods.is_empty());
}

// Node service tests
struct NodeService<R: MockKubernetesRepository> {
    repository: Arc<R>,
}

impl<R: MockKubernetesRepository> NodeService<R> {
    fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    async fn count_nodes_by_status(&self) -> Result<(usize, usize), String> {
        let nodes = self.repository.get_nodes().await?;
        
        let ready = nodes.iter().filter(|n| n.status == "Ready").count();
        let not_ready = nodes.len() - ready;
        
        Ok((ready, not_ready))
    }

    async fn get_master_nodes(&self) -> Result<Vec<MockNode>, String> {
        let nodes = self.repository.get_nodes().await?;
        
        let masters: Vec<MockNode> = nodes
            .into_iter()
            .filter(|n| n.role == "master" || n.role == "control-plane")
            .collect();
        
        Ok(masters)
    }
}

#[tokio::test]
async fn test_count_nodes_by_status() {
    let data = MockClusterData {
        pods: vec![],
        nodes: vec![
            MockNode { name: "master-1".to_string(), status: "Ready".to_string(), role: "master".to_string() },
            MockNode { name: "worker-1".to_string(), status: "Ready".to_string(), role: "worker".to_string() },
            MockNode { name: "worker-2".to_string(), status: "NotReady".to_string(), role: "worker".to_string() },
        ],
    };

    let repo = MockK8sRepo { data: Arc::new(Mutex::new(data)) };
    let service = NodeService::new(Arc::new(repo));

    let (ready, not_ready) = service.count_nodes_by_status().await.unwrap();

    assert_eq!(ready, 2);
    assert_eq!(not_ready, 1);
}

#[tokio::test]
async fn test_get_master_nodes() {
    let data = MockClusterData {
        pods: vec![],
        nodes: vec![
            MockNode { name: "master-1".to_string(), status: "Ready".to_string(), role: "master".to_string() },
            MockNode { name: "master-2".to_string(), status: "Ready".to_string(), role: "master".to_string() },
            MockNode { name: "worker-1".to_string(), status: "Ready".to_string(), role: "worker".to_string() },
            MockNode { name: "worker-2".to_string(), status: "Ready".to_string(), role: "worker".to_string() },
        ],
    };

    let repo = MockK8sRepo { data: Arc::new(Mutex::new(data)) };
    let service = NodeService::new(Arc::new(repo));

    let masters = service.get_master_nodes().await.unwrap();

    assert_eq!(masters.len(), 2);
    assert!(masters.iter().all(|n| n.role == "master"));
}
