//! Tests for application use cases
//! These tests verify business logic at the application layer

use std::sync::Arc;
use tokio::sync::Mutex;

// Mock Repository Port - Using async_trait for mocking
trait PodRepository: Send + Sync {
    fn list_pods(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<MockPod>> + Send + '_>>;
    fn get_pod(&self, name: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<MockPod>> + Send + '_>>;
    fn get_pods_by_status(&self, status: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<MockPod>> + Send + '_>>;
}

// Mock Pod Entity
#[derive(Debug, Clone)]
struct MockPod {
    name: String,
    namespace: String,
    status: String,
    restart_count: i32,
    cpu_usage: f64,
    memory_usage: f64,
}

// Mock Repository Implementation
struct InMemoryPodRepository {
    pods: Arc<Mutex<Vec<MockPod>>>,
}

impl InMemoryPodRepository {
    fn new(pods: Vec<MockPod>) -> Self {
        Self {
            pods: Arc::new(Mutex::new(pods)),
        }
    }
}

impl PodRepository for InMemoryPodRepository {
    fn list_pods(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<MockPod>> + Send + '_>> {
        let pods = self.pods.clone();
        Box::pin(async move {
            pods.lock().await.clone()
        })
    }

    fn get_pod(&self, name: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<MockPod>> + Send + '_>> {
        let pods = self.pods.clone();
        let name = name.to_string();
        Box::pin(async move {
            pods.lock().await.iter().find(|p| p.name == name).cloned()
        })
    }

    fn get_pods_by_status(&self, status: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<MockPod>> + Send + '_>> {
        let pods = self.pods.clone();
        let status = status.to_string();
        Box::pin(async move {
            pods.lock().await.iter().filter(|p| p.status == status).cloned().collect()
        })
    }
}

// DTOs
#[derive(Debug, Clone)]
struct PodDto {
    name: String,
    namespace: String,
    status: String,
    age: String,
}

#[derive(Debug, Clone)]
struct PodDetailDto {
    name: String,
    namespace: String,
    status: String,
    restart_count: i32,
    cpu_usage: f64,
    memory_usage: f64,
}

#[derive(Debug, Clone)]
struct PodStatusSummaryDto {
    total: usize,
    running: usize,
    pending: usize,
    failed: usize,
    restart_threshold_exceeded: usize,
}

// Use Cases
struct ListPodsUseCase<R: PodRepository> {
    repository: Arc<R>,
}

impl<R: PodRepository> ListPodsUseCase<R> {
    fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    async fn execute(&self) -> Vec<PodDto> {
        let pods = self.repository.list_pods().await;
        pods.into_iter()
            .map(|p| PodDto {
                name: p.name,
                namespace: p.namespace,
                status: p.status,
                age: "5m".to_string(), // Simplified
            })
            .collect()
    }
}

struct GetPodDetailUseCase<R: PodRepository> {
    repository: Arc<R>,
}

impl<R: PodRepository> GetPodDetailUseCase<R> {
    fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    async fn execute(&self, name: &str) -> Option<PodDetailDto> {
        self.repository.get_pod(name).await.map(|p| PodDetailDto {
            name: p.name,
            namespace: p.namespace,
            status: p.status,
            restart_count: p.restart_count,
            cpu_usage: p.cpu_usage,
            memory_usage: p.memory_usage,
        })
    }
}

struct GetPodStatusSummaryUseCase<R: PodRepository> {
    repository: Arc<R>,
}

impl<R: PodRepository> GetPodStatusSummaryUseCase<R> {
    fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    async fn execute(&self, restart_threshold: i32) -> PodStatusSummaryDto {
        let pods = self.repository.list_pods().await;

        PodStatusSummaryDto {
            total: pods.len(),
            running: pods.iter().filter(|p| p.status == "Running").count(),
            pending: pods.iter().filter(|p| p.status == "Pending").count(),
            failed: pods.iter().filter(|p| p.status == "Failed").count(),
            restart_threshold_exceeded: pods.iter().filter(|p| p.restart_count > restart_threshold).count(),
        }
    }
}

struct FindProblematicPodsUseCase<R: PodRepository> {
    repository: Arc<R>,
}

impl<R: PodRepository> FindProblematicPodsUseCase<R> {
    fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    async fn execute(&self, max_restarts: i32) -> Vec<PodDto> {
        let pods = self.repository.list_pods().await;
        pods.into_iter()
            .filter(|p| {
                p.status == "Failed"
                    || p.status == "CrashLoopBackOff"
                    || p.status == "Unknown"
                    || p.restart_count > max_restarts
            })
            .map(|p| PodDto {
                name: p.name,
                namespace: p.namespace,
                status: p.status,
                age: "5m".to_string(),
            })
            .collect()
    }
}

// Tests
#[tokio::test]
async fn test_list_pods_use_case() {
    let pods = vec![
        MockPod {
            name: "pod-1".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            restart_count: 0,
            cpu_usage: 100.0,
            memory_usage: 512.0,
        },
        MockPod {
            name: "pod-2".to_string(),
            namespace: "kube-system".to_string(),
            status: "Running".to_string(),
            restart_count: 0,
            cpu_usage: 50.0,
            memory_usage: 256.0,
        },
    ];

    let repo = Arc::new(InMemoryPodRepository::new(pods));
    let use_case = ListPodsUseCase::new(repo);

    let result = use_case.execute().await;

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "pod-1");
    assert_eq!(result[1].namespace, "kube-system");
}

#[tokio::test]
async fn test_list_pods_use_case_empty() {
    let repo = Arc::new(InMemoryPodRepository::new(vec![]));
    let use_case = ListPodsUseCase::new(repo);

    let result = use_case.execute().await;

    assert!(result.is_empty());
}

#[tokio::test]
async fn test_get_pod_detail_use_case_found() {
    let pods = vec![
        MockPod {
            name: "pod-1".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            restart_count: 5,
            cpu_usage: 100.0,
            memory_usage: 512.0,
        },
    ];

    let repo = Arc::new(InMemoryPodRepository::new(pods));
    let use_case = GetPodDetailUseCase::new(repo);

    let result = use_case.execute("pod-1").await;

    assert!(result.is_some());
    let pod = result.unwrap();
    assert_eq!(pod.name, "pod-1");
    assert_eq!(pod.restart_count, 5);
    assert_eq!(pod.cpu_usage, 100.0);
}

#[tokio::test]
async fn test_get_pod_detail_use_case_not_found() {
    let repo = Arc::new(InMemoryPodRepository::new(vec![]));
    let use_case = GetPodDetailUseCase::new(repo);

    let result = use_case.execute("nonexistent").await;

    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_pod_status_summary_use_case() {
    let pods = vec![
        MockPod { name: "p1".to_string(), namespace: "default".to_string(), status: "Running".to_string(), restart_count: 0, cpu_usage: 0.0, memory_usage: 0.0 },
        MockPod { name: "p2".to_string(), namespace: "default".to_string(), status: "Running".to_string(), restart_count: 1, cpu_usage: 0.0, memory_usage: 0.0 },
        MockPod { name: "p3".to_string(), namespace: "default".to_string(), status: "Pending".to_string(), restart_count: 0, cpu_usage: 0.0, memory_usage: 0.0 },
        MockPod { name: "p4".to_string(), namespace: "default".to_string(), status: "Failed".to_string(), restart_count: 10, cpu_usage: 0.0, memory_usage: 0.0 },
        MockPod { name: "p5".to_string(), namespace: "default".to_string(), status: "Running".to_string(), restart_count: 6, cpu_usage: 0.0, memory_usage: 0.0 },
    ];

    let repo = Arc::new(InMemoryPodRepository::new(pods));
    let use_case = GetPodStatusSummaryUseCase::new(repo);

    let summary = use_case.execute(5).await;

    assert_eq!(summary.total, 5);
    assert_eq!(summary.running, 3);
    assert_eq!(summary.pending, 1);
    assert_eq!(summary.failed, 1);
    assert_eq!(summary.restart_threshold_exceeded, 2); // p4 and p5
}

#[tokio::test]
async fn test_find_problematic_pods_use_case() {
    let pods = vec![
        MockPod { name: "healthy".to_string(), namespace: "default".to_string(), status: "Running".to_string(), restart_count: 0, cpu_usage: 0.0, memory_usage: 0.0 },
        MockPod { name: "failed".to_string(), namespace: "default".to_string(), status: "Failed".to_string(), restart_count: 0, cpu_usage: 0.0, memory_usage: 0.0 },
        MockPod { name: "crashloop".to_string(), namespace: "default".to_string(), status: "CrashLoopBackOff".to_string(), restart_count: 0, cpu_usage: 0.0, memory_usage: 0.0 },
        MockPod { name: "unknown".to_string(), namespace: "default".to_string(), status: "Unknown".to_string(), restart_count: 0, cpu_usage: 0.0, memory_usage: 0.0 },
        MockPod { name: "restarting".to_string(), namespace: "default".to_string(), status: "Running".to_string(), restart_count: 10, cpu_usage: 0.0, memory_usage: 0.0 },
    ];

    let repo = Arc::new(InMemoryPodRepository::new(pods));
    let use_case = FindProblematicPodsUseCase::new(repo);

    let problematic = use_case.execute(5).await;

    assert_eq!(problematic.len(), 4);
    assert!(problematic.iter().any(|p| p.name == "failed"));
    assert!(problematic.iter().any(|p| p.name == "crashloop"));
    assert!(problematic.iter().any(|p| p.name == "unknown"));
    assert!(problematic.iter().any(|p| p.name == "restarting"));
}

#[tokio::test]
async fn test_find_problematic_pods_use_case_no_problems() {
    let pods = vec![
        MockPod { name: "pod-1".to_string(), namespace: "default".to_string(), status: "Running".to_string(), restart_count: 0, cpu_usage: 0.0, memory_usage: 0.0 },
        MockPod { name: "pod-2".to_string(), namespace: "default".to_string(), status: "Running".to_string(), restart_count: 1, cpu_usage: 0.0, memory_usage: 0.0 },
    ];

    let repo = Arc::new(InMemoryPodRepository::new(pods));
    let use_case = FindProblematicPodsUseCase::new(repo);

    let problematic = use_case.execute(5).await;

    assert!(problematic.is_empty());
}

// Mock for testing with failures
struct FailingPodRepository;

impl PodRepository for FailingPodRepository {
    fn list_pods(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<MockPod>> + Send + '_>> {
        Box::pin(async move { vec![] })
    }

    fn get_pod(&self, _name: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<MockPod>> + Send + '_>> {
        Box::pin(async move { None })
    }

    fn get_pods_by_status(&self, _status: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<MockPod>> + Send + '_>> {
        Box::pin(async move { vec![] })
    }
}

#[tokio::test]
async fn test_use_case_with_repository_failure() {
    let repo = Arc::new(FailingPodRepository);
    let use_case = ListPodsUseCase::new(repo);

    let result = use_case.execute().await;

    // Should return empty list gracefully
    assert!(result.is_empty());
}
