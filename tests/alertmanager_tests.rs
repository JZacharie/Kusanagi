#[cfg(test)]
mod tests {
    use kusanagi::legacy::alertmanager::{Alert, AlertsResponse};
    use chrono::Utc;

    #[test]
    fn test_alerts_response_new() {
        let response = AlertsResponse {
            critical: vec![],
            warning: vec![],
            info: vec![],
            total: 0,
            firing: 0,
            pending: 0,
        };

        assert_eq!(response.total, 0);
        assert_eq!(response.firing, 0);
        assert_eq!(response.pending, 0);
    }

    #[test]
    fn test_alert_creation() {
        let alert = Alert {
            name: "TestAlert".to_string(),
            severity: "critical".to_string(),
            state: "firing".to_string(),
            summary: "Test summary".to_string(),
            description: Some("Test description".to_string()),
            namespace: Some("default".to_string()),
            pod: Some("test-pod".to_string()),
            started_at: Utc::now(),
            fingerprint: "abc123".to_string(),
        };

        assert_eq!(alert.name, "TestAlert");
        assert_eq!(alert.severity, "critical");
        assert_eq!(alert.state, "firing");
    }

    #[test]
    fn test_alerts_response_with_alerts() {
        let critical_alert = Alert {
            name: "CriticalAlert".to_string(),
            severity: "critical".to_string(),
            state: "firing".to_string(),
            summary: "Critical issue".to_string(),
            description: None,
            namespace: None,
            pod: None,
            started_at: Utc::now(),
            fingerprint: "crit123".to_string(),
        };

        let warning_alert = Alert {
            name: "WarningAlert".to_string(),
            severity: "warning".to_string(),
            state: "firing".to_string(),
            summary: "Warning issue".to_string(),
            description: None,
            namespace: None,
            pod: None,
            started_at: Utc::now(),
            fingerprint: "warn123".to_string(),
        };

        let response = AlertsResponse {
            critical: vec![critical_alert],
            warning: vec![warning_alert],
            info: vec![],
            total: 2,
            firing: 2,
            pending: 0,
        };

        assert_eq!(response.critical.len(), 1);
        assert_eq!(response.warning.len(), 1);
        assert_eq!(response.info.len(), 0);
        assert_eq!(response.total, 2);
    }
}
