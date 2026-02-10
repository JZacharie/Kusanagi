//! Extended tests for domain services
//! Tests for Proxmox, HomeAssistant, MQTT, News services

#![allow(dead_code)]

use std::sync::Arc;
use tokio::sync::Mutex;

// ============================================================================
// Proxmox Service Tests
// ============================================================================

#[derive(Debug, Clone)]
struct ProxmoxVM {
    vmid: u32,
    name: String,
    status: String,
    cpu_usage: f64,
    memory_usage: u64,
    memory_total: u64,
}

#[derive(Debug, Clone)]
struct ProxmoxContainer {
    vmid: u32,
    name: String,
    status: String,
}

struct ProxmoxRepository {
    vms: Arc<Mutex<Vec<ProxmoxVM>>>,
    containers: Arc<Mutex<Vec<ProxmoxContainer>>>,
}

impl ProxmoxRepository {
    fn new() -> Self {
        Self {
            vms: Arc::new(Mutex::new(vec![])),
            containers: Arc::new(Mutex::new(vec![])),
        }
    }

    async fn list_vms(&self) -> Vec<ProxmoxVM> {
        self.vms.lock().await.clone()
    }

    async fn list_containers(&self) -> Vec<ProxmoxContainer> {
        self.containers.lock().await.clone()
    }
}

struct ProxmoxService {
    repository: Arc<ProxmoxRepository>,
}

impl ProxmoxService {
    fn new(repository: Arc<ProxmoxRepository>) -> Self {
        Self { repository }
    }

    async fn get_running_vms(&self) -> Vec<ProxmoxVM> {
        self.repository
            .list_vms()
            .await
            .into_iter()
            .filter(|vm| vm.status == "running")
            .collect()
    }

    async fn get_stopped_vms(&self) -> Vec<ProxmoxVM> {
        self.repository
            .list_vms()
            .await
            .into_iter()
            .filter(|vm| vm.status == "stopped")
            .collect()
    }

    async fn get_high_cpu_vms(&self, threshold: f64) -> Vec<ProxmoxVM> {
        self.repository
            .list_vms()
            .await
            .into_iter()
            .filter(|vm| vm.cpu_usage > threshold)
            .collect()
    }

    async fn get_memory_usage_percentage(&self, vmid: u32) -> Option<f64> {
        let vms = self.repository.list_vms().await;
        vms.iter()
            .find(|vm| vm.vmid == vmid)
            .map(|vm| (vm.memory_usage as f64 / vm.memory_total as f64) * 100.0)
    }
}

#[tokio::test]
async fn test_proxmox_get_running_vms() {
    let repo = Arc::new(ProxmoxRepository::new());

    repo.vms.lock().await.extend(vec![
        ProxmoxVM {
            vmid: 100,
            name: "vm1".to_string(),
            status: "running".to_string(),
            cpu_usage: 0.5,
            memory_usage: 1024,
            memory_total: 4096,
        },
        ProxmoxVM {
            vmid: 101,
            name: "vm2".to_string(),
            status: "stopped".to_string(),
            cpu_usage: 0.0,
            memory_usage: 0,
            memory_total: 4096,
        },
        ProxmoxVM {
            vmid: 102,
            name: "vm3".to_string(),
            status: "running".to_string(),
            cpu_usage: 0.3,
            memory_usage: 2048,
            memory_total: 4096,
        },
    ]);

    let service = ProxmoxService::new(repo);
    let running = service.get_running_vms().await;

    assert_eq!(running.len(), 2);
    assert!(running.iter().all(|vm| vm.status == "running"));
}

#[tokio::test]
async fn test_proxmox_get_high_cpu_vms() {
    let repo = Arc::new(ProxmoxRepository::new());

    repo.vms.lock().await.extend(vec![
        ProxmoxVM {
            vmid: 100,
            name: "vm1".to_string(),
            status: "running".to_string(),
            cpu_usage: 0.9,
            memory_usage: 1024,
            memory_total: 4096,
        },
        ProxmoxVM {
            vmid: 101,
            name: "vm2".to_string(),
            status: "running".to_string(),
            cpu_usage: 0.3,
            memory_usage: 1024,
            memory_total: 4096,
        },
        ProxmoxVM {
            vmid: 102,
            name: "vm3".to_string(),
            status: "running".to_string(),
            cpu_usage: 0.95,
            memory_usage: 1024,
            memory_total: 4096,
        },
    ]);

    let service = ProxmoxService::new(repo);
    let high_cpu = service.get_high_cpu_vms(0.8).await;

    assert_eq!(high_cpu.len(), 2);
    assert!(high_cpu.iter().all(|vm| vm.cpu_usage > 0.8));
}

#[tokio::test]
async fn test_proxmox_get_memory_usage_percentage() {
    let repo = Arc::new(ProxmoxRepository::new());

    repo.vms.lock().await.push(ProxmoxVM {
        vmid: 100,
        name: "vm1".to_string(),
        status: "running".to_string(),
        cpu_usage: 0.5,
        memory_usage: 2048,
        memory_total: 4096,
    });

    let service = ProxmoxService::new(repo);
    let percentage = service.get_memory_usage_percentage(100).await;

    assert_eq!(percentage, Some(50.0));
}

// ============================================================================
// HomeAssistant Service Tests
// ============================================================================

#[derive(Debug, Clone)]
struct HaEntity {
    entity_id: String,
    state: String,
    attributes: std::collections::HashMap<String, String>,
    last_updated: String,
}

struct HomeAssistantRepository {
    entities: Arc<Mutex<Vec<HaEntity>>>,
}

impl HomeAssistantRepository {
    fn new() -> Self {
        Self {
            entities: Arc::new(Mutex::new(vec![])),
        }
    }

    async fn get_entities(&self) -> Vec<HaEntity> {
        self.entities.lock().await.clone()
    }
}

struct HomeAssistantService {
    repository: Arc<HomeAssistantRepository>,
}

impl HomeAssistantService {
    fn new(repository: Arc<HomeAssistantRepository>) -> Self {
        Self { repository }
    }

    async fn get_entities_by_domain(&self, domain: &str) -> Vec<HaEntity> {
        self.repository
            .get_entities()
            .await
            .into_iter()
            .filter(|e| e.entity_id.starts_with(&format!("{}.", domain)))
            .collect()
    }

    async fn get_on_entities(&self) -> Vec<HaEntity> {
        self.repository
            .get_entities()
            .await
            .into_iter()
            .filter(|e| e.state == "on")
            .collect()
    }

    async fn count_entities_by_state(&self, state: &str) -> usize {
        self.repository
            .get_entities()
            .await
            .iter()
            .filter(|e| e.state == state)
            .count()
    }
}

#[tokio::test]
async fn test_ha_get_entities_by_domain() {
    let repo = Arc::new(HomeAssistantRepository::new());

    let mut attrs = std::collections::HashMap::new();
    attrs.insert("friendly_name".to_string(), "Living Room".to_string());

    repo.entities.lock().await.extend(vec![
        HaEntity {
            entity_id: "light.living_room".to_string(),
            state: "on".to_string(),
            attributes: attrs.clone(),
            last_updated: "2024-01-01".to_string(),
        },
        HaEntity {
            entity_id: "light.kitchen".to_string(),
            state: "off".to_string(),
            attributes: attrs.clone(),
            last_updated: "2024-01-01".to_string(),
        },
        HaEntity {
            entity_id: "sensor.temperature".to_string(),
            state: "22.5".to_string(),
            attributes: attrs.clone(),
            last_updated: "2024-01-01".to_string(),
        },
    ]);

    let service = HomeAssistantService::new(repo);
    let lights = service.get_entities_by_domain("light").await;

    assert_eq!(lights.len(), 2);
    assert!(lights.iter().all(|e| e.entity_id.starts_with("light.")));
}

#[tokio::test]
async fn test_ha_get_on_entities() {
    let repo = Arc::new(HomeAssistantRepository::new());

    let attrs = std::collections::HashMap::new();

    repo.entities.lock().await.extend(vec![
        HaEntity {
            entity_id: "light.living_room".to_string(),
            state: "on".to_string(),
            attributes: attrs.clone(),
            last_updated: "2024-01-01".to_string(),
        },
        HaEntity {
            entity_id: "light.kitchen".to_string(),
            state: "off".to_string(),
            attributes: attrs.clone(),
            last_updated: "2024-01-01".to_string(),
        },
        HaEntity {
            entity_id: "switch.pump".to_string(),
            state: "on".to_string(),
            attributes: attrs.clone(),
            last_updated: "2024-01-01".to_string(),
        },
    ]);

    let service = HomeAssistantService::new(repo);
    let on_entities = service.get_on_entities().await;

    assert_eq!(on_entities.len(), 2);
    assert!(on_entities.iter().all(|e| e.state == "on"));
}

// ============================================================================
// MQTT Service Tests
// ============================================================================

#[derive(Debug, Clone)]
struct MqttMessage {
    topic: String,
    payload: String,
    qos: u8,
    retained: bool,
}

struct MqttRepository {
    messages: Arc<Mutex<Vec<MqttMessage>>>,
}

impl MqttRepository {
    fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(vec![])),
        }
    }

    async fn get_messages(&self) -> Vec<MqttMessage> {
        self.messages.lock().await.clone()
    }
}

struct MqttService {
    repository: Arc<MqttRepository>,
}

impl MqttService {
    fn new(repository: Arc<MqttRepository>) -> Self {
        Self { repository }
    }

    async fn get_messages_by_topic(&self, topic_pattern: &str) -> Vec<MqttMessage> {
        self.repository
            .get_messages()
            .await
            .into_iter()
            .filter(|m| m.topic.starts_with(topic_pattern))
            .collect()
    }

    async fn count_retained_messages(&self) -> usize {
        self.repository
            .get_messages()
            .await
            .iter()
            .filter(|m| m.retained)
            .count()
    }

    async fn get_high_qos_messages(&self) -> Vec<MqttMessage> {
        self.repository
            .get_messages()
            .await
            .into_iter()
            .filter(|m| m.qos > 0)
            .collect()
    }
}

#[tokio::test]
async fn test_mqtt_get_messages_by_topic() {
    let repo = Arc::new(MqttRepository::new());

    repo.messages.lock().await.extend(vec![
        MqttMessage {
            topic: "home/living_room/temperature".to_string(),
            payload: "22.5".to_string(),
            qos: 1,
            retained: true,
        },
        MqttMessage {
            topic: "home/kitchen/temperature".to_string(),
            payload: "23.0".to_string(),
            qos: 0,
            retained: false,
        },
        MqttMessage {
            topic: "sensors/outside/humidity".to_string(),
            payload: "60".to_string(),
            qos: 1,
            retained: true,
        },
    ]);

    let service = MqttService::new(repo);
    let home_messages = service.get_messages_by_topic("home/").await;

    assert_eq!(home_messages.len(), 2);
    assert!(home_messages.iter().all(|m| m.topic.starts_with("home/")));
}

#[tokio::test]
async fn test_mqtt_count_retained_messages() {
    let repo = Arc::new(MqttRepository::new());

    repo.messages.lock().await.extend(vec![
        MqttMessage {
            topic: "t1".to_string(),
            payload: "p1".to_string(),
            qos: 0,
            retained: true,
        },
        MqttMessage {
            topic: "t2".to_string(),
            payload: "p2".to_string(),
            qos: 0,
            retained: false,
        },
        MqttMessage {
            topic: "t3".to_string(),
            payload: "p3".to_string(),
            qos: 0,
            retained: true,
        },
    ]);

    let service = MqttService::new(repo);
    let retained_count = service.count_retained_messages().await;

    assert_eq!(retained_count, 2);
}

// ============================================================================
// News Service Tests
// ============================================================================

#[derive(Debug, Clone)]
struct NewsItem {
    title: String,
    source: String,
    published_at: String,
    category: String,
    url: String,
}

struct NewsRepository {
    items: Arc<Mutex<Vec<NewsItem>>>,
}

impl NewsRepository {
    fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(vec![])),
        }
    }

    async fn get_items(&self) -> Vec<NewsItem> {
        self.items.lock().await.clone()
    }
}

struct NewsService {
    repository: Arc<NewsRepository>,
}

impl NewsService {
    fn new(repository: Arc<NewsRepository>) -> Self {
        Self { repository }
    }

    async fn get_items_by_source(&self, source: &str) -> Vec<NewsItem> {
        self.repository
            .get_items()
            .await
            .into_iter()
            .filter(|item| item.source == source)
            .collect()
    }

    async fn get_items_by_category(&self, category: &str) -> Vec<NewsItem> {
        self.repository
            .get_items()
            .await
            .into_iter()
            .filter(|item| item.category == category)
            .collect()
    }

    async fn count_by_source(&self) -> std::collections::HashMap<String, usize> {
        let items = self.repository.get_items().await;
        let mut counts = std::collections::HashMap::new();

        for item in items {
            *counts.entry(item.source).or_insert(0) += 1;
        }

        counts
    }
}

#[tokio::test]
async fn test_news_get_items_by_source() {
    let repo = Arc::new(NewsRepository::new());

    repo.items.lock().await.extend(vec![
        NewsItem {
            title: "Article 1".to_string(),
            source: "TechCrunch".to_string(),
            published_at: "2024-01-01".to_string(),
            category: "tech".to_string(),
            url: "http://example.com/1".to_string(),
        },
        NewsItem {
            title: "Article 2".to_string(),
            source: "TechCrunch".to_string(),
            published_at: "2024-01-02".to_string(),
            category: "tech".to_string(),
            url: "http://example.com/2".to_string(),
        },
        NewsItem {
            title: "Article 3".to_string(),
            source: "BBC".to_string(),
            published_at: "2024-01-03".to_string(),
            category: "world".to_string(),
            url: "http://example.com/3".to_string(),
        },
    ]);

    let service = NewsService::new(repo);
    let techcrunch_items = service.get_items_by_source("TechCrunch").await;

    assert_eq!(techcrunch_items.len(), 2);
    assert!(techcrunch_items
        .iter()
        .all(|item| item.source == "TechCrunch"));
}

#[tokio::test]
async fn test_news_count_by_source() {
    let repo = Arc::new(NewsRepository::new());

    repo.items.lock().await.extend(vec![
        NewsItem {
            title: "A1".to_string(),
            source: "Source1".to_string(),
            published_at: "2024-01-01".to_string(),
            category: "tech".to_string(),
            url: "".to_string(),
        },
        NewsItem {
            title: "A2".to_string(),
            source: "Source1".to_string(),
            published_at: "2024-01-02".to_string(),
            category: "tech".to_string(),
            url: "".to_string(),
        },
        NewsItem {
            title: "A3".to_string(),
            source: "Source2".to_string(),
            published_at: "2024-01-03".to_string(),
            category: "world".to_string(),
            url: "".to_string(),
        },
    ]);

    let service = NewsService::new(repo);
    let counts = service.count_by_source().await;

    assert_eq!(counts.get("Source1"), Some(&2));
    assert_eq!(counts.get("Source2"), Some(&1));
}
