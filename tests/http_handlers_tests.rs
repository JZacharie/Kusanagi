//! Tests for HTTP Handlers (Alert, Backup, Security, HomeAssistant)

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json as AxumJson, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

// ============================================================================
// Alert Handlers Tests
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AlertQuery {
    refresh: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Alert {
    id: String,
    severity: String,
    message: String,
    source: String,
    timestamp: String,
}

#[derive(Clone)]
struct AlertState {
    alerts: Arc<Mutex<Vec<Alert>>>,
    local_mode: bool,
}

async fn get_alerts_handler(
    Query(query): Query<AlertQuery>,
    State(state): State<AlertState>,
) -> impl IntoResponse {
    let force_refresh = query.refresh.unwrap_or(false);

    if state.local_mode {
        return AxumJson(json!({
            "alerts": [
                {"id": "1", "severity": "warning", "message": "Test alert", "source": "test"}
            ],
            "total": 1,
            "local_mode": true
        }));
    }

    let alerts = state.alerts.lock().unwrap();
    let alert_list: Vec<_> = if force_refresh {
        // Simulate refresh
        alerts.iter().cloned().collect()
    } else {
        alerts.iter().cloned().collect()
    };

    AxumJson(json!({
        "alerts": alert_list,
        "total": alert_list.len(),
        "refreshed": force_refresh,
    }))
}

#[tokio::test]
async fn test_get_alerts_basic() {
    let state = AlertState {
        alerts: Arc::new(Mutex::new(vec![Alert {
            id: "1".to_string(),
            severity: "critical".to_string(),
            message: "High CPU".to_string(),
            source: "monitoring".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }])),
        local_mode: false,
    };

    let app = Router::new()
        .route("/api/alerts", get(get_alerts_handler))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/alerts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["total"], 1);
    assert!(!json["refreshed"].as_bool().unwrap());
}

#[tokio::test]
async fn test_get_alerts_with_refresh() {
    let state = AlertState {
        alerts: Arc::new(Mutex::new(vec![])),
        local_mode: false,
    };

    let app = Router::new()
        .route("/api/alerts", get(get_alerts_handler))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/alerts?refresh=true")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["refreshed"].as_bool().unwrap());
}

#[tokio::test]
async fn test_get_alerts_local_mode() {
    let state = AlertState {
        alerts: Arc::new(Mutex::new(vec![])),
        local_mode: true,
    };

    let app = Router::new()
        .route("/api/alerts", get(get_alerts_handler))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/alerts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["local_mode"].as_bool().unwrap());
    assert_eq!(json["total"], 1);
}

// ============================================================================
// Backup Handlers Tests
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Backup {
    name: String,
    namespace: String,
    status: String,
    created_at: String,
    size_bytes: u64,
}

#[derive(Clone)]
struct BackupState {
    backups: Arc<Mutex<Vec<Backup>>>,
}

async fn get_backups_handler(State(state): State<BackupState>) -> impl IntoResponse {
    let backups = state.backups.lock().unwrap();
    let backup_list: Vec<_> = backups.iter().cloned().collect();

    AxumJson(json!({
        "backups": backup_list,
        "count": backup_list.len(),
    }))
}

async fn get_backups_by_namespace(
    Path(namespace): Path<String>,
    State(state): State<BackupState>,
) -> impl IntoResponse {
    let backups = state.backups.lock().unwrap();
    let filtered: Vec<_> = backups
        .iter()
        .filter(|b| b.namespace == namespace)
        .cloned()
        .collect();

    AxumJson(json!({
        "backups": filtered,
        "count": filtered.len(),
        "namespace": namespace,
    }))
}

async fn trigger_backup_handler(
    Path((namespace, name)): Path<(String, String)>,
    State(state): State<BackupState>,
) -> impl IntoResponse {
    let mut backups = state.backups.lock().unwrap();

    // Check if backup already exists
    if let Some(backup) = backups
        .iter_mut()
        .find(|b| b.namespace == namespace && b.name == name)
    {
        backup.status = "InProgress".to_string();
        AxumJson(json!({
            "triggered": true,
            "backup": backup,
            "message": "Backup already exists, restarting",
        }))
    } else {
        let new_backup = Backup {
            name: name.clone(),
            namespace: namespace.clone(),
            status: "Pending".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            size_bytes: 0,
        };
        backups.push(new_backup.clone());
        AxumJson(json!({
            "triggered": true,
            "backup": new_backup,
            "message": "New backup triggered",
        }))
    }
}

#[tokio::test]
async fn test_get_backups() {
    let state = BackupState {
        backups: Arc::new(Mutex::new(vec![
            Backup {
                name: "backup-1".to_string(),
                namespace: "default".to_string(),
                status: "Completed".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                size_bytes: 1024 * 1024 * 100,
            },
            Backup {
                name: "backup-2".to_string(),
                namespace: "production".to_string(),
                status: "InProgress".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                size_bytes: 0,
            },
        ])),
    };

    let app = Router::new()
        .route("/api/backups", get(get_backups_handler))
        .route("/api/backups/{namespace}", get(get_backups_by_namespace))
        .route(
            "/api/backups/{namespace}/{name}/trigger",
            post(trigger_backup_handler),
        )
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/backups")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["count"], 2);
}

#[tokio::test]
async fn test_get_backups_by_namespace() {
    let state = BackupState {
        backups: Arc::new(Mutex::new(vec![
            Backup {
                name: "backup-1".to_string(),
                namespace: "default".to_string(),
                status: "Completed".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                size_bytes: 1024 * 1024 * 100,
            },
            Backup {
                name: "backup-2".to_string(),
                namespace: "default".to_string(),
                status: "Completed".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                size_bytes: 1024 * 1024 * 200,
            },
        ])),
    };

    let app = Router::new()
        .route("/api/backups", get(get_backups_handler))
        .route("/api/backups/{namespace}", get(get_backups_by_namespace))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/backups/default")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["count"], 2);
    assert_eq!(json["namespace"], "default");
}

#[tokio::test]
async fn test_trigger_new_backup() {
    let state = BackupState {
        backups: Arc::new(Mutex::new(vec![])),
    };

    let app = Router::new()
        .route(
            "/api/backups/{namespace}/{name}/trigger",
            post(trigger_backup_handler),
        )
        .with_state(state.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/backups/production/app-db/trigger")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["triggered"].as_bool().unwrap());
    assert_eq!(json["message"], "New backup triggered");

    // Verify backup was created
    let backups = state.backups.lock().unwrap();
    assert_eq!(backups.len(), 1);
    assert_eq!(backups[0].status, "Pending");
}

#[tokio::test]
async fn test_trigger_existing_backup() {
    let state = BackupState {
        backups: Arc::new(Mutex::new(vec![Backup {
            name: "app-db".to_string(),
            namespace: "production".to_string(),
            status: "Completed".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            size_bytes: 1024 * 1024 * 100,
        }])),
    };

    let app = Router::new()
        .route(
            "/api/backups/{namespace}/{name}/trigger",
            post(trigger_backup_handler),
        )
        .with_state(state.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/backups/production/app-db/trigger")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["triggered"].as_bool().unwrap());
    assert_eq!(json["message"], "Backup already exists, restarting");

    // Verify backup status was updated
    let backups = state.backups.lock().unwrap();
    assert_eq!(backups[0].status, "InProgress");
}

// ============================================================================
// Security Handlers Tests
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Vulnerability {
    id: String,
    severity: String,
    package: String,
    version: String,
    fixed_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SecurityReport {
    category: String,
    name: String,
    critical_count: usize,
    high_count: usize,
    medium_count: usize,
    low_count: usize,
}

#[derive(Clone)]
struct SecurityState {
    vulnerabilities: Arc<Mutex<Vec<Vulnerability>>>,
    reports: Arc<Mutex<Vec<SecurityReport>>>,
}

async fn get_security_handler(State(state): State<SecurityState>) -> impl IntoResponse {
    let vulns = state.vulnerabilities.lock().unwrap();

    let critical = vulns.iter().filter(|v| v.severity == "critical").count();
    let high = vulns.iter().filter(|v| v.severity == "high").count();
    let medium = vulns.iter().filter(|v| v.severity == "medium").count();
    let low = vulns.iter().filter(|v| v.severity == "low").count();
    let fixable = vulns.iter().filter(|v| v.fixed_version.is_some()).count();

    AxumJson(json!({
        "summary": {
            "critical": critical,
            "high": high,
            "medium": medium,
            "low": low,
            "total": vulns.len(),
            "fixable": fixable,
        },
        "status": if critical > 0 { "critical" } else if high > 0 { "warning" } else { "ok" },
    }))
}

async fn get_security_reports_handler(State(state): State<SecurityState>) -> impl IntoResponse {
    let reports = state.reports.lock().unwrap();
    let report_list: Vec<_> = reports.iter().cloned().collect();

    AxumJson(json!({
        "reports": report_list,
        "count": report_list.len(),
    }))
}

async fn get_security_report_handler(
    Path((category, name)): Path<(String, String)>,
    State(state): State<SecurityState>,
) -> impl IntoResponse {
    let reports = state.reports.lock().unwrap();

    if let Some(report) = reports
        .iter()
        .find(|r| r.category == category && r.name == name)
    {
        AxumJson(json!({
            "found": true,
            "report": report,
        }))
        .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            AxumJson(json!({
                "found": false,
                "error": format!("Report {}/{} not found", category, name),
            })),
        )
            .into_response()
    }
}

async fn get_vulnerabilities_handler(State(state): State<SecurityState>) -> impl IntoResponse {
    let vulns = state.vulnerabilities.lock().unwrap();
    let vuln_list: Vec<_> = vulns.iter().cloned().collect();

    AxumJson(json!({
        "vulnerabilities": vuln_list,
        "count": vuln_list.len(),
    }))
}

#[tokio::test]
async fn test_get_security_summary() {
    let state = SecurityState {
        vulnerabilities: Arc::new(Mutex::new(vec![
            Vulnerability {
                id: "CVE-2024-0001".to_string(),
                severity: "critical".to_string(),
                package: "openssl".to_string(),
                version: "1.1.1".to_string(),
                fixed_version: Some("1.1.2".to_string()),
            },
            Vulnerability {
                id: "CVE-2024-0002".to_string(),
                severity: "high".to_string(),
                package: "curl".to_string(),
                version: "7.88.0".to_string(),
                fixed_version: Some("7.88.1".to_string()),
            },
            Vulnerability {
                id: "CVE-2024-0003".to_string(),
                severity: "medium".to_string(),
                package: "nginx".to_string(),
                version: "1.20.0".to_string(),
                fixed_version: None,
            },
        ])),
        reports: Arc::new(Mutex::new(vec![])),
    };

    let app = Router::new()
        .route("/api/security/summary", get(get_security_handler))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/security/summary")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["summary"]["critical"], 1);
    assert_eq!(json["summary"]["high"], 1);
    assert_eq!(json["summary"]["medium"], 1);
    assert_eq!(json["summary"]["total"], 3);
    assert_eq!(json["summary"]["fixable"], 2);
    assert_eq!(json["status"], "critical");
}

#[tokio::test]
async fn test_get_security_reports() {
    let state = SecurityState {
        vulnerabilities: Arc::new(Mutex::new(vec![])),
        reports: Arc::new(Mutex::new(vec![SecurityReport {
            category: "images".to_string(),
            name: "app".to_string(),
            critical_count: 0,
            high_count: 2,
            medium_count: 5,
            low_count: 10,
        }])),
    };

    let app = Router::new()
        .route("/api/security/reports", get(get_security_reports_handler))
        .route(
            "/api/security/reports/{category}/{name}",
            get(get_security_report_handler),
        )
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/security/reports")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["count"], 1);
}

#[tokio::test]
async fn test_get_security_report_found() {
    let state = SecurityState {
        vulnerabilities: Arc::new(Mutex::new(vec![])),
        reports: Arc::new(Mutex::new(vec![SecurityReport {
            category: "images".to_string(),
            name: "app".to_string(),
            critical_count: 0,
            high_count: 2,
            medium_count: 5,
            low_count: 10,
        }])),
    };

    let app = Router::new()
        .route(
            "/api/security/reports/{category}/{name}",
            get(get_security_report_handler),
        )
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/security/reports/images/app")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["found"].as_bool().unwrap());
    assert_eq!(json["report"]["category"], "images");
}

#[tokio::test]
async fn test_get_security_report_not_found() {
    let state = SecurityState {
        vulnerabilities: Arc::new(Mutex::new(vec![])),
        reports: Arc::new(Mutex::new(vec![])),
    };

    let app = Router::new()
        .route(
            "/api/security/reports/{category}/{name}",
            get(get_security_report_handler),
        )
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/security/reports/unknown/test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_vulnerabilities() {
    let state = SecurityState {
        vulnerabilities: Arc::new(Mutex::new(vec![Vulnerability {
            id: "CVE-2024-0001".to_string(),
            severity: "critical".to_string(),
            package: "openssl".to_string(),
            version: "1.1.1".to_string(),
            fixed_version: Some("1.1.2".to_string()),
        }])),
        reports: Arc::new(Mutex::new(vec![])),
    };

    let app = Router::new()
        .route(
            "/api/security/vulnerabilities",
            get(get_vulnerabilities_handler),
        )
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/security/vulnerabilities")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["count"], 1);
    assert_eq!(json["vulnerabilities"][0]["id"], "CVE-2024-0001");
}

// ============================================================================
// HomeAssistant Handlers Tests
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Device {
    id: String,
    name: String,
    entity_id: String,
    state: String,
    attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Sensor {
    entity_id: String,
    state: String,
    unit_of_measurement: Option<String>,
    friendly_name: String,
    last_updated: String,
}

#[derive(Clone)]
struct HomeAssistantState {
    devices: Arc<Mutex<Vec<Device>>>,
    sensors: Arc<Mutex<Vec<Sensor>>>,
}

async fn get_devices_handler(State(state): State<HomeAssistantState>) -> impl IntoResponse {
    let devices = state.devices.lock().unwrap();
    let device_list: Vec<_> = devices.iter().cloned().collect();

    AxumJson(json!({
        "devices": device_list,
        "count": device_list.len(),
    }))
}

async fn get_sensors_handler(State(state): State<HomeAssistantState>) -> impl IntoResponse {
    let sensors = state.sensors.lock().unwrap();
    let sensor_list: Vec<_> = sensors.iter().cloned().collect();

    AxumJson(json!({
        "sensors": sensor_list,
        "count": sensor_list.len(),
    }))
}

#[tokio::test]
async fn test_get_devices() {
    let mut attributes = HashMap::new();
    attributes.insert("brightness".to_string(), json!(255));
    attributes.insert("color_temp".to_string(), json!(300));

    let state = HomeAssistantState {
        devices: Arc::new(Mutex::new(vec![
            Device {
                id: "light.living_room".to_string(),
                name: "Living Room Light".to_string(),
                entity_id: "light.living_room".to_string(),
                state: "on".to_string(),
                attributes: attributes.clone(),
            },
            Device {
                id: "switch.kitchen".to_string(),
                name: "Kitchen Switch".to_string(),
                entity_id: "switch.kitchen".to_string(),
                state: "off".to_string(),
                attributes: HashMap::new(),
            },
        ])),
        sensors: Arc::new(Mutex::new(vec![])),
    };

    let app = Router::new()
        .route("/api/ha/devices", get(get_devices_handler))
        .route("/api/ha/sensors", get(get_sensors_handler))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/ha/devices")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["count"], 2);
    assert_eq!(json["devices"][0]["name"], "Living Room Light");
    assert_eq!(json["devices"][0]["attributes"]["brightness"], 255);
}

#[tokio::test]
async fn test_get_sensors() {
    let state = HomeAssistantState {
        devices: Arc::new(Mutex::new(vec![])),
        sensors: Arc::new(Mutex::new(vec![
            Sensor {
                entity_id: "sensor.temperature".to_string(),
                state: "22.5".to_string(),
                unit_of_measurement: Some("°C".to_string()),
                friendly_name: "Temperature".to_string(),
                last_updated: chrono::Utc::now().to_rfc3339(),
            },
            Sensor {
                entity_id: "sensor.humidity".to_string(),
                state: "60".to_string(),
                unit_of_measurement: Some("%".to_string()),
                friendly_name: "Humidity".to_string(),
                last_updated: chrono::Utc::now().to_rfc3339(),
            },
        ])),
    };

    let app = Router::new()
        .route("/api/ha/devices", get(get_devices_handler))
        .route("/api/ha/sensors", get(get_sensors_handler))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/ha/sensors")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["count"], 2);
    assert_eq!(json["sensors"][0]["unit_of_measurement"], "°C");
}
