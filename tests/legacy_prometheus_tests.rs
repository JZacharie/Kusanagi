//! Tests for legacy prometheus module

// ============================================================================
// Prometheus Types and Functions
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
struct Metric {
    name: String,
    value: f64,
    labels: Vec<(String, String)>,
    timestamp: Option<i64>,
}

#[derive(Debug, Clone)]
struct MetricFamily {
    name: String,
    help: String,
    metric_type: MetricType,
    metrics: Vec<Metric>,
}

#[derive(Debug, Clone, PartialEq)]
enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// Parse Prometheus text format
fn parse_prometheus_text(text: &str) -> Vec<MetricFamily> {
    let mut families = Vec::new();
    let lines: Vec<&str> = text.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        // Parse HELP
        if line.starts_with("# HELP ") {
            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() >= 3 {
                let name = parts[2].to_string();
                let help = if i + 1 < lines.len() && lines[i + 1].starts_with("# TYPE ") {
                    let type_parts: Vec<&str> = lines[i + 1].splitn(3, ' ').collect();
                    let metric_type = match type_parts.get(2) {
                        Some(&"counter") => MetricType::Counter,
                        Some(&"gauge") => MetricType::Gauge,
                        Some(&"histogram") => MetricType::Histogram,
                        Some(&"summary") => MetricType::Summary,
                        _ => MetricType::Gauge,
                    };

                    // Skip TYPE line
                    i += 1;

                    // Parse metrics
                    let mut metrics = Vec::new();
                    i += 1;
                    while i < lines.len()
                        && !lines[i].starts_with('#')
                        && !lines[i].trim().is_empty()
                    {
                        if let Some(metric) = parse_metric_line(lines[i]) {
                            metrics.push(metric);
                        }
                        i += 1;
                    }

                    families.push(MetricFamily {
                        name: name.clone(),
                        help: parts[2].to_string(),
                        metric_type,
                        metrics,
                    });
                    continue;
                } else {
                    parts[2].to_string()
                };

                families.push(MetricFamily {
                    name,
                    help,
                    metric_type: MetricType::Gauge,
                    metrics: vec![],
                });
            }
        }

        i += 1;
    }

    families
}

fn parse_metric_line(line: &str) -> Option<Metric> {
    // Simple parser for: metric_name{label1="value1"} 42
    let line = line.trim();

    if let Some(value_start) = line.rfind(' ') {
        let name_and_labels = &line[..value_start];
        let value_str = &line[value_start + 1..];

        if let Ok(value) = value_str.parse::<f64>() {
            let (name, labels) = parse_name_and_labels(name_and_labels);

            return Some(Metric {
                name,
                value,
                labels,
                timestamp: None,
            });
        }
    }

    None
}

fn parse_name_and_labels(input: &str) -> (String, Vec<(String, String)>) {
    if let Some(bracket_start) = input.find('{') {
        let name = input[..bracket_start].to_string();
        let labels_str = &input[bracket_start + 1..input.len() - 1];

        let labels: Vec<(String, String)> = labels_str
            .split(',')
            .filter_map(|pair| {
                let parts: Vec<&str> = pair.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim().to_string();
                    let value = parts[1].trim().trim_matches('"').to_string();
                    Some((key, value))
                } else {
                    None
                }
            })
            .collect();

        (name, labels)
    } else {
        (input.to_string(), vec![])
    }
}

/// Query metrics by name
fn query_metrics_by_name<'a>(families: &'a [MetricFamily], name: &str) -> Vec<&'a Metric> {
    families
        .iter()
        .filter(|f| f.name == name)
        .flat_map(|f| &f.metrics)
        .collect()
}

/// Calculate average value for a metric
fn calculate_average(families: &[MetricFamily], name: &str) -> Option<f64> {
    let metrics = query_metrics_by_name(families, name);
    if metrics.is_empty() {
        return None;
    }

    let sum: f64 = metrics.iter().map(|m| m.value).sum();
    Some(sum / metrics.len() as f64)
}

/// Find metrics above threshold
fn find_metrics_above_threshold<'a>(
    families: &'a [MetricFamily],
    name: &str,
    threshold: f64,
) -> Vec<&'a Metric> {
    query_metrics_by_name(families, name)
        .into_iter()
        .filter(|m| m.value > threshold)
        .collect()
}

/// Format metric for display
fn format_metric(metric: &Metric) -> String {
    let labels_str = if metric.labels.is_empty() {
        String::new()
    } else {
        let pairs: Vec<String> = metric
            .labels
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, v))
            .collect();
        format!("{{{}}}", pairs.join(","))
    };

    format!("{}{} = {}", metric.name, labels_str, metric.value)
}

/// Check if metrics are healthy (no critical thresholds exceeded)
fn check_metrics_health(families: &[MetricFamily], thresholds: &[(String, f64)]) -> HealthStatus {
    let mut violations = Vec::new();

    for (metric_name, threshold) in thresholds {
        let metrics = query_metrics_by_name(families, metric_name);
        for metric in metrics {
            if metric.value > *threshold {
                violations.push(format!(
                    "{} exceeds threshold: {} > {}",
                    metric.name, metric.value, threshold
                ));
            }
        }
    }

    if violations.is_empty() {
        HealthStatus::Healthy
    } else {
        HealthStatus::Unhealthy(violations)
    }
}

#[derive(Debug, PartialEq)]
enum HealthStatus {
    Healthy,
    Unhealthy(Vec<String>),
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_parse_simple_metric() {
    let input = "http_requests_total 1027";
    let metric = parse_metric_line(input).unwrap();

    assert_eq!(metric.name, "http_requests_total");
    assert_eq!(metric.value, 1027.0);
    assert!(metric.labels.is_empty());
}

#[test]
fn test_parse_metric_with_labels() {
    let input = r#"http_requests_total{method="GET",status="200"} 1027"#;
    let metric = parse_metric_line(input).unwrap();

    assert_eq!(metric.name, "http_requests_total");
    assert_eq!(metric.value, 1027.0);
    assert_eq!(metric.labels.len(), 2);
    assert!(metric
        .labels
        .contains(&("method".to_string(), "GET".to_string())));
    assert!(metric
        .labels
        .contains(&("status".to_string(), "200".to_string())));
}

#[test]
fn test_parse_metric_float_value() {
    let input = "cpu_usage 45.67";
    let metric = parse_metric_line(input).unwrap();

    assert_eq!(metric.value, 45.67);
}

#[test]
fn test_parse_metric_invalid() {
    let input = "invalid_metric";
    assert!(parse_metric_line(input).is_none());
}

#[test]
fn test_parse_prometheus_text() {
    let text = r#"# HELP http_requests_total Total HTTP requests
# TYPE http_requests_total counter
http_requests_total{method="GET",status="200"} 1027
http_requests_total{method="POST",status="201"} 37

# HELP cpu_usage CPU usage percentage
# TYPE cpu_usage gauge
cpu_usage 45.67
"#;

    let families = parse_prometheus_text(text);

    assert_eq!(families.len(), 2);

    // Check first family
    assert_eq!(families[0].name, "http_requests_total");
    assert_eq!(families[0].metric_type, MetricType::Counter);
    assert_eq!(families[0].metrics.len(), 2);

    // Check second family
    assert_eq!(families[1].name, "cpu_usage");
    assert_eq!(families[1].metric_type, MetricType::Gauge);
    assert_eq!(families[1].metrics.len(), 1);
}

#[test]
fn test_query_metrics_by_name() {
    let families = vec![
        MetricFamily {
            name: "http_requests".to_string(),
            help: "HTTP requests".to_string(),
            metric_type: MetricType::Counter,
            metrics: vec![Metric {
                name: "http_requests".to_string(),
                value: 100.0,
                labels: vec![],
                timestamp: None,
            }],
        },
        MetricFamily {
            name: "cpu_usage".to_string(),
            help: "CPU usage".to_string(),
            metric_type: MetricType::Gauge,
            metrics: vec![Metric {
                name: "cpu_usage".to_string(),
                value: 50.0,
                labels: vec![],
                timestamp: None,
            }],
        },
    ];

    let metrics = query_metrics_by_name(&families, "http_requests");
    assert_eq!(metrics.len(), 1);
    assert_eq!(metrics[0].value, 100.0);
}

#[test]
fn test_query_metrics_by_name_not_found() {
    let families: Vec<MetricFamily> = vec![];
    let metrics = query_metrics_by_name(&families, "nonexistent");
    assert!(metrics.is_empty());
}

#[test]
fn test_calculate_average() {
    let families = vec![MetricFamily {
        name: "cpu".to_string(),
        help: "CPU".to_string(),
        metric_type: MetricType::Gauge,
        metrics: vec![
            Metric {
                name: "cpu".to_string(),
                value: 10.0,
                labels: vec![],
                timestamp: None,
            },
            Metric {
                name: "cpu".to_string(),
                value: 20.0,
                labels: vec![],
                timestamp: None,
            },
            Metric {
                name: "cpu".to_string(),
                value: 30.0,
                labels: vec![],
                timestamp: None,
            },
        ],
    }];

    let avg = calculate_average(&families, "cpu").unwrap();
    assert_eq!(avg, 20.0);
}

#[test]
fn test_calculate_average_empty() {
    let families: Vec<MetricFamily> = vec![];
    assert!(calculate_average(&families, "nonexistent").is_none());
}

#[test]
fn test_find_metrics_above_threshold() {
    let families = vec![MetricFamily {
        name: "memory".to_string(),
        help: "Memory".to_string(),
        metric_type: MetricType::Gauge,
        metrics: vec![
            Metric {
                name: "memory".to_string(),
                value: 50.0,
                labels: vec![],
                timestamp: None,
            },
            Metric {
                name: "memory".to_string(),
                value: 80.0,
                labels: vec![],
                timestamp: None,
            },
            Metric {
                name: "memory".to_string(),
                value: 95.0,
                labels: vec![],
                timestamp: None,
            },
        ],
    }];

    let high_memory = find_metrics_above_threshold(&families, "memory", 70.0);
    assert_eq!(high_memory.len(), 2);
    assert!(high_memory.iter().all(|m| m.value > 70.0));
}

#[test]
fn test_format_metric_simple() {
    let metric = Metric {
        name: "requests".to_string(),
        value: 100.0,
        labels: vec![],
        timestamp: None,
    };

    assert_eq!(format_metric(&metric), "requests = 100");
}

#[test]
fn test_format_metric_with_labels() {
    let metric = Metric {
        name: "requests".to_string(),
        value: 100.0,
        labels: vec![
            ("method".to_string(), "GET".to_string()),
            ("status".to_string(), "200".to_string()),
        ],
        timestamp: None,
    };

    let formatted = format_metric(&metric);
    assert!(formatted.contains("requests{"));
    assert!(formatted.contains("method=\"GET\""));
    assert!(formatted.contains("status=\"200\""));
    assert!(formatted.contains(" = 100"));
}

#[test]
fn test_check_metrics_health_healthy() {
    let families = vec![MetricFamily {
        name: "cpu".to_string(),
        help: "CPU".to_string(),
        metric_type: MetricType::Gauge,
        metrics: vec![Metric {
            name: "cpu".to_string(),
            value: 50.0,
            labels: vec![],
            timestamp: None,
        }],
    }];

    let thresholds = vec![("cpu".to_string(), 80.0)];
    let status = check_metrics_health(&families, &thresholds);

    assert_eq!(status, HealthStatus::Healthy);
}

#[test]
fn test_check_metrics_health_unhealthy() {
    let families = vec![
        MetricFamily {
            name: "cpu".to_string(),
            help: "CPU".to_string(),
            metric_type: MetricType::Gauge,
            metrics: vec![Metric {
                name: "cpu".to_string(),
                value: 95.0,
                labels: vec![],
                timestamp: None,
            }],
        },
        MetricFamily {
            name: "memory".to_string(),
            help: "Memory".to_string(),
            metric_type: MetricType::Gauge,
            metrics: vec![Metric {
                name: "memory".to_string(),
                value: 90.0,
                labels: vec![],
                timestamp: None,
            }],
        },
    ];

    let thresholds = vec![("cpu".to_string(), 80.0), ("memory".to_string(), 85.0)];
    let status = check_metrics_health(&families, &thresholds);

    match status {
        HealthStatus::Unhealthy(violations) => {
            assert_eq!(violations.len(), 2);
        }
        _ => panic!("Expected unhealthy status"),
    }
}
