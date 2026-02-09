//! Tests for security service (Trivy integration)

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// Mock types for security scanning
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Vulnerability {
    id: String,
    severity: Severity,
    package_name: String,
    installed_version: String,
    fixed_version: Option<String>,
    description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Unknown,
}

impl Severity {
    fn score(&self) -> u8 {
        match self {
            Severity::Critical => 10,
            Severity::High => 7,
            Severity::Medium => 4,
            Severity::Low => 1,
            Severity::Unknown => 0,
        }
    }

    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "critical" => Severity::Critical,
            "high" => Severity::High,
            "medium" => Severity::Medium,
            "low" => Severity::Low,
            _ => Severity::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
struct SecurityReport {
    image: String,
    vulnerabilities: Vec<Vulnerability>,
    scanned_at: String,
}

#[derive(Debug, Clone)]
struct SecurityRepository {
    reports: Arc<Mutex<Vec<SecurityReport>>>,
}

impl SecurityRepository {
    fn new() -> Self {
        Self {
            reports: Arc::new(Mutex::new(vec![])),
        }
    }

    async fn save_report(&self, report: SecurityReport) {
        self.reports.lock().await.push(report);
    }

    async fn get_report(&self, image: &str) -> Option<SecurityReport> {
        self.reports
            .lock()
            .await
            .iter()
            .find(|r| r.image == image)
            .cloned()
    }

    async fn list_reports(&self) -> Vec<SecurityReport> {
        self.reports.lock().await.clone()
    }
}

// Service layer
struct SecurityService {
    repository: Arc<SecurityRepository>,
}

impl SecurityService {
    fn new(repository: Arc<SecurityRepository>) -> Self {
        Self { repository }
    }

    async fn get_vulnerability_summary(&self) -> VulnerabilitySummary {
        let reports = self.repository.list_reports().await;

        let mut critical = 0;
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;

        for report in &reports {
            for vuln in &report.vulnerabilities {
                match vuln.severity {
                    Severity::Critical => critical += 1,
                    Severity::High => high += 1,
                    Severity::Medium => medium += 1,
                    Severity::Low => low += 1,
                    _ => {}
                }
            }
        }

        let total = critical + high + medium + low;

        VulnerabilitySummary {
            total,
            critical,
            high,
            medium,
            low,
            risk_score: if total > 0 {
                ((critical * 10 + high * 7 + medium * 4 + low) as f64 / total as f64) * 10.0
            } else {
                0.0
            },
        }
    }

    async fn get_critical_vulnerabilities(&self) -> Vec<VulnerabilityWithImage> {
        let reports = self.repository.list_reports().await;
        let mut critical = vec![];

        for report in reports {
            for vuln in report.vulnerabilities {
                if vuln.severity == Severity::Critical {
                    critical.push(VulnerabilityWithImage {
                        image: report.image.clone(),
                        vulnerability: vuln,
                    });
                }
            }
        }

        critical
    }

    async fn get_vulnerabilities_by_severity(
        &self,
        severity: Severity,
    ) -> Vec<VulnerabilityWithImage> {
        let reports = self.repository.list_reports().await;
        let mut result = vec![];

        for report in reports {
            for vuln in report.vulnerabilities {
                if vuln.severity == severity {
                    result.push(VulnerabilityWithImage {
                        image: report.image.clone(),
                        vulnerability: vuln,
                    });
                }
            }
        }

        result
    }

    async fn has_fixable_vulnerabilities(&self, image: &str) -> Option<Vec<Vulnerability>> {
        let report = self.repository.get_report(image).await?;
        let fixable: Vec<Vulnerability> = report
            .vulnerabilities
            .into_iter()
            .filter(|v| v.fixed_version.is_some())
            .collect();

        if fixable.is_empty() {
            None
        } else {
            Some(fixable)
        }
    }

    async fn get_images_with_critical_vulns(&self) -> Vec<String> {
        let reports = self.repository.list_reports().await;
        reports
            .into_iter()
            .filter(|r| {
                r.vulnerabilities
                    .iter()
                    .any(|v| v.severity == Severity::Critical)
            })
            .map(|r| r.image)
            .collect()
    }
}

#[derive(Debug, Clone)]
struct VulnerabilitySummary {
    total: u32,
    critical: u32,
    high: u32,
    medium: u32,
    low: u32,
    risk_score: f64,
}

#[derive(Debug, Clone)]
struct VulnerabilityWithImage {
    image: String,
    vulnerability: Vulnerability,
}

#[test]
fn test_severity_score() {
    assert_eq!(Severity::Critical.score(), 10);
    assert_eq!(Severity::High.score(), 7);
    assert_eq!(Severity::Medium.score(), 4);
    assert_eq!(Severity::Low.score(), 1);
    assert_eq!(Severity::Unknown.score(), 0);
}

#[test]
fn test_severity_from_str() {
    assert_eq!(Severity::from_str("critical"), Severity::Critical);
    assert_eq!(Severity::from_str("CRITICAL"), Severity::Critical);
    assert_eq!(Severity::from_str("high"), Severity::High);
    assert_eq!(Severity::from_str("medium"), Severity::Medium);
    assert_eq!(Severity::from_str("low"), Severity::Low);
    assert_eq!(Severity::from_str("unknown"), Severity::Unknown);
}

#[tokio::test]
async fn test_empty_vulnerability_summary() {
    let repo = Arc::new(SecurityRepository::new());
    let service = SecurityService::new(repo);

    let summary = service.get_vulnerability_summary().await;

    assert_eq!(summary.total, 0);
    assert_eq!(summary.critical, 0);
    assert_eq!(summary.high, 0);
    assert_eq!(summary.risk_score, 0.0);
}

#[tokio::test]
async fn test_vulnerability_summary_with_data() {
    let repo = Arc::new(SecurityRepository::new());

    let report = SecurityReport {
        image: "nginx:latest".to_string(),
        vulnerabilities: vec![
            Vulnerability {
                id: "CVE-2023-0001".to_string(),
                severity: Severity::Critical,
                package_name: "openssl".to_string(),
                installed_version: "1.1.1".to_string(),
                fixed_version: Some("1.1.2".to_string()),
                description: "Critical vulnerability".to_string(),
            },
            Vulnerability {
                id: "CVE-2023-0002".to_string(),
                severity: Severity::High,
                package_name: "curl".to_string(),
                installed_version: "7.80".to_string(),
                fixed_version: Some("7.81".to_string()),
                description: "High severity".to_string(),
            },
            Vulnerability {
                id: "CVE-2023-0003".to_string(),
                severity: Severity::Medium,
                package_name: "bash".to_string(),
                installed_version: "5.1".to_string(),
                fixed_version: None,
                description: "Medium severity".to_string(),
            },
        ],
        scanned_at: "2024-01-01".to_string(),
    };

    repo.save_report(report).await;

    let service = SecurityService::new(repo);
    let summary = service.get_vulnerability_summary().await;

    assert_eq!(summary.total, 3);
    assert_eq!(summary.critical, 1);
    assert_eq!(summary.high, 1);
    assert_eq!(summary.medium, 1);
    assert_eq!(summary.low, 0);
    assert!(summary.risk_score > 0.0);
}

#[tokio::test]
async fn test_get_critical_vulnerabilities() {
    let repo = Arc::new(SecurityRepository::new());

    repo.save_report(SecurityReport {
        image: "app:v1".to_string(),
        vulnerabilities: vec![
            Vulnerability {
                id: "CVE-2023-0001".to_string(),
                severity: Severity::Critical,
                package_name: "openssl".to_string(),
                installed_version: "1.1.1".to_string(),
                fixed_version: Some("1.1.2".to_string()),
                description: "Critical".to_string(),
            },
            Vulnerability {
                id: "CVE-2023-0002".to_string(),
                severity: Severity::High,
                package_name: "curl".to_string(),
                installed_version: "7.80".to_string(),
                fixed_version: None,
                description: "High".to_string(),
            },
        ],
        scanned_at: "2024-01-01".to_string(),
    })
    .await;

    let service = SecurityService::new(repo);
    let critical = service.get_critical_vulnerabilities().await;

    assert_eq!(critical.len(), 1);
    assert_eq!(critical[0].vulnerability.id, "CVE-2023-0001");
    assert_eq!(critical[0].image, "app:v1");
}

#[tokio::test]
async fn test_get_vulnerabilities_by_severity() {
    let repo = Arc::new(SecurityRepository::new());

    repo.save_report(SecurityReport {
        image: "app:v1".to_string(),
        vulnerabilities: vec![
            Vulnerability {
                id: "CVE-2023-0001".to_string(),
                severity: Severity::High,
                package_name: "pkg1".to_string(),
                installed_version: "1.0".to_string(),
                fixed_version: None,
                description: "High".to_string(),
            },
            Vulnerability {
                id: "CVE-2023-0002".to_string(),
                severity: Severity::High,
                package_name: "pkg2".to_string(),
                installed_version: "2.0".to_string(),
                fixed_version: None,
                description: "High".to_string(),
            },
        ],
        scanned_at: "2024-01-01".to_string(),
    })
    .await;

    let service = SecurityService::new(repo);
    let high_vulns = service
        .get_vulnerabilities_by_severity(Severity::High)
        .await;

    assert_eq!(high_vulns.len(), 2);
}

#[tokio::test]
async fn test_has_fixable_vulnerabilities() {
    let repo = Arc::new(SecurityRepository::new());

    repo.save_report(SecurityReport {
        image: "fixable-app:v1".to_string(),
        vulnerabilities: vec![
            Vulnerability {
                id: "CVE-2023-0001".to_string(),
                severity: Severity::High,
                package_name: "openssl".to_string(),
                installed_version: "1.1.1".to_string(),
                fixed_version: Some("1.1.2".to_string()),
                description: "Fixable".to_string(),
            },
            Vulnerability {
                id: "CVE-2023-0002".to_string(),
                severity: Severity::Medium,
                package_name: "bash".to_string(),
                installed_version: "5.1".to_string(),
                fixed_version: None,
                description: "Not fixable".to_string(),
            },
        ],
        scanned_at: "2024-01-01".to_string(),
    })
    .await;

    let service = SecurityService::new(repo);
    let fixable = service.has_fixable_vulnerabilities("fixable-app:v1").await;

    assert!(fixable.is_some());
    assert_eq!(fixable.unwrap().len(), 1);
}

#[tokio::test]
async fn test_has_fixable_vulnerabilities_none() {
    let repo = Arc::new(SecurityRepository::new());

    repo.save_report(SecurityReport {
        image: "no-fix-app:v1".to_string(),
        vulnerabilities: vec![Vulnerability {
            id: "CVE-2023-0001".to_string(),
            severity: Severity::High,
            package_name: "pkg".to_string(),
            installed_version: "1.0".to_string(),
            fixed_version: None,
            description: "No fix available".to_string(),
        }],
        scanned_at: "2024-01-01".to_string(),
    })
    .await;

    let service = SecurityService::new(repo);
    let fixable = service.has_fixable_vulnerabilities("no-fix-app:v1").await;

    assert!(fixable.is_none());
}

#[tokio::test]
async fn test_get_images_with_critical_vulns() {
    let repo = Arc::new(SecurityRepository::new());

    repo.save_report(SecurityReport {
        image: "critical-app:v1".to_string(),
        vulnerabilities: vec![Vulnerability {
            id: "CVE-2023-0001".to_string(),
            severity: Severity::Critical,
            package_name: "openssl".to_string(),
            installed_version: "1.0".to_string(),
            fixed_version: None,
            description: "Critical".to_string(),
        }],
        scanned_at: "2024-01-01".to_string(),
    })
    .await;

    repo.save_report(SecurityReport {
        image: "safe-app:v1".to_string(),
        vulnerabilities: vec![Vulnerability {
            id: "CVE-2023-0002".to_string(),
            severity: Severity::Low,
            package_name: "pkg".to_string(),
            installed_version: "1.0".to_string(),
            fixed_version: None,
            description: "Low".to_string(),
        }],
        scanned_at: "2024-01-01".to_string(),
    })
    .await;

    let service = SecurityService::new(repo);
    let images = service.get_images_with_critical_vulns().await;

    assert_eq!(images.len(), 1);
    assert_eq!(images[0], "critical-app:v1");
}

#[tokio::test]
async fn test_multiple_reports_summary() {
    let repo = Arc::new(SecurityRepository::new());

    // Report 1
    repo.save_report(SecurityReport {
        image: "app1:v1".to_string(),
        vulnerabilities: vec![Vulnerability {
            id: "CVE-2023-0001".to_string(),
            severity: Severity::Critical,
            package_name: "pkg".to_string(),
            installed_version: "1.0".to_string(),
            fixed_version: None,
            description: "Critical".to_string(),
        }],
        scanned_at: "2024-01-01".to_string(),
    })
    .await;

    // Report 2
    repo.save_report(SecurityReport {
        image: "app2:v1".to_string(),
        vulnerabilities: vec![
            Vulnerability {
                id: "CVE-2023-0002".to_string(),
                severity: Severity::High,
                package_name: "pkg".to_string(),
                installed_version: "1.0".to_string(),
                fixed_version: None,
                description: "High".to_string(),
            },
            Vulnerability {
                id: "CVE-2023-0003".to_string(),
                severity: Severity::High,
                package_name: "pkg2".to_string(),
                installed_version: "2.0".to_string(),
                fixed_version: None,
                description: "High".to_string(),
            },
        ],
        scanned_at: "2024-01-01".to_string(),
    })
    .await;

    let service = SecurityService::new(repo);
    let summary = service.get_vulnerability_summary().await;

    assert_eq!(summary.total, 3);
    assert_eq!(summary.critical, 1);
    assert_eq!(summary.high, 2);
}
