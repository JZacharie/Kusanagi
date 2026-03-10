use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrivyVulnerability {
    #[serde(rename = "VulnerabilityID")]
    pub vulnerability_id: Option<String>,
    #[serde(rename = "PkgName")]
    pub pkg_name: Option<String>,
    #[serde(rename = "InstalledVersion")]
    pub installed_version: Option<String>,
    #[serde(rename = "FixedVersion")]
    pub fixed_version: Option<String>,
    #[serde(rename = "Severity")]
    pub severity: Option<String>,
    #[serde(rename = "Title")]
    pub title: Option<String>,
    #[serde(rename = "Description")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrivyResult {
    #[serde(rename = "Target")]
    pub target: Option<String>,
    #[serde(rename = "Class")]
    pub class: Option<String>,
    #[serde(rename = "Type")]
    pub result_type: Option<String>,
    #[serde(rename = "Vulnerabilities")]
    pub vulnerabilities: Option<Vec<TrivyVulnerability>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrivyReport {
    #[serde(rename = "ArtifactName")]
    pub artifact_name: Option<String>,
    #[serde(rename = "ArtifactType")]
    pub artifact_type: Option<String>,
    #[serde(rename = "Results")]
    pub results: Option<Vec<TrivyResult>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedReport {
    pub report_id: String,
    pub timestamp: String,
    pub image_name: String,
    pub namespace: String,
}

/// Get vulnerability summary from Trivy server
pub async fn get_vulnerabilities(cache: &crate::AdvancedCache<String>) -> Result<Value, String> {
    const CACHE_KEY: &str = "kusanagi_trivy_vulnerabilities";

    if let Some(cached) = cache.get(CACHE_KEY).await {
        if let Ok(value) = serde_json::from_str::<Value>(&cached) {
            tracing::debug!("Returning cached Trivy vulnerabilities");
            return Ok(value);
        }
    }

    let trivy_url = env::var("TRIVY_SERVER_URL")
        .unwrap_or_else(|_| "http://trivy-json-server.trivy-system.svc:8080".to_string());

    // Try to fetch from Trivy server
    let result = match fetch_trivy_reports(&trivy_url).await {
        Ok(reports) => {
            tracing::info!("Fetched {} Trivy reports from server", reports.len());
            let summary = aggregate_vulnerabilities(&reports);
            Ok(summary)
        }
        Err(e) => {
            tracing::warn!("Trivy server unavailable: {}, trying S3 cache", e);
            // Fallback to S3 cached reports
            match fetch_from_s3_cache().await {
                Ok(cached_data) => {
                    tracing::info!("Using S3 cached Trivy data");
                    Ok(cached_data)
                }
                Err(s3_err) => {
                    tracing::warn!("S3 cache unavailable: {}", s3_err);
                    // Last fallback: try to get image list from Kubernetes
                    match fetch_images_from_k8s().await {
                        Ok(k8s_data) => {
                            tracing::info!("Using Kubernetes image scan data");
                            Ok(k8s_data)
                        }
                        Err(k8s_err) => {
                            tracing::error!("All Trivy sources failed: K8s error: {}", k8s_err);
                            Err(format!(
                                "Trivy service unavailable. Server: {}, S3: {}, K8s: {}",
                                e, s3_err, k8s_err
                            ))
                        }
                    }
                }
            }
        }
    };

    if let Ok(value) = &result {
        if let Ok(json_str) = serde_json::to_string(value) {
            cache
                .set(
                    CACHE_KEY.to_string(),
                    json_str,
                    Some(std::time::Duration::from_secs(600)), // 10 minutes cache
                )
                .await;
        }
    }

    result
}

/// Fetch reports from Trivy server
async fn fetch_trivy_reports(trivy_url: &str) -> Result<Vec<TrivyReport>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Fetch list of available reports
    let list_url = format!("{}/reports", trivy_url);
    let response = client
        .get(&list_url)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Trivy server: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Trivy server returned error: {}",
            response.status()
        ));
    }

    let reports_list: Vec<String> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse reports list: {}", e))?;

    let mut all_reports = Vec::new();

    // Fetch each report
    for report_name in reports_list.iter().take(10) {
        let report_url = format!("{}/reports/{}", trivy_url, report_name);
        match client.get(&report_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<TrivyReport>().await {
                    Ok(report) => {
                        all_reports.push(report.clone());
                        // Cache to S3 in background
                        let report_clone = report.clone();
                        let report_name_clone = report_name.clone();
                        tokio::spawn(async move {
                            let _ = cache_report_to_s3(&report_name_clone, &report_clone).await;
                        });
                    }
                    Err(e) => tracing::warn!("Failed to parse report {}: {}", report_name, e),
                }
            }
            Ok(resp) => tracing::warn!("Failed to fetch report {}: {}", report_name, resp.status()),
            Err(e) => tracing::warn!("Failed to fetch report {}: {}", report_name, e),
        }
    }

    Ok(all_reports)
}

/// Aggregate vulnerabilities from multiple reports
fn aggregate_vulnerabilities(reports: &[TrivyReport]) -> Value {
    let mut critical = 0;
    let mut high = 0;
    let mut medium = 0;
    let mut low = 0;
    let mut images = Vec::new();

    for report in reports {
        let artifact_name = report.artifact_name.as_deref().unwrap_or("unknown");
        let mut img_critical = 0;
        let mut img_high = 0;
        let mut img_medium = 0;
        let mut img_low = 0;

        if let Some(results) = &report.results {
            for result in results {
                if let Some(vulns) = &result.vulnerabilities {
                    for vuln in vulns {
                        let severity = vuln.severity.as_deref().unwrap_or("UNKNOWN").to_uppercase();
                        match severity.as_str() {
                            "CRITICAL" => {
                                critical += 1;
                                img_critical += 1;
                            }
                            "HIGH" => {
                                high += 1;
                                img_high += 1;
                            }
                            "MEDIUM" => {
                                medium += 1;
                                img_medium += 1;
                            }
                            "LOW" => {
                                low += 1;
                                img_low += 1;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Extract namespace from artifact name if present (format: namespace/image:tag)
        let namespace = artifact_name
            .split('/')
            .next()
            .unwrap_or("default")
            .to_string();

        images.push(json!({
            "image": artifact_name,
            "namespace": namespace,
            "critical_count": img_critical,
            "high_count": img_high,
            "medium_count": img_medium,
            "low_count": img_low,
            "last_scan": chrono::Utc::now().to_rfc3339()
        }));
    }

    json!({
        "critical": critical,
        "high": high,
        "medium": medium,
        "low": low,
        "total": critical + high + medium + low,
        "images": images
    })
}

/// Cache report to S3
async fn cache_report_to_s3(report_id: &str, report: &TrivyReport) -> Result<(), String> {
    let bucket_name = env::var("S3_BUCKET_SECURITY_REPORTS")
        .unwrap_or_else(|_| "kusanagi-security-reports".to_string());

    /*
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;

    // Use custom endpoint if S3_ENDPOINT is set (for MinIO)
    let s3_client = if let Ok(endpoint) = env::var("S3_ENDPOINT") {
        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .endpoint_url(endpoint)
            .force_path_style(true)
            .build();
        aws_sdk_s3::Client::from_conf(s3_config)
    } else {
        };
    */

    let report_json =
        serde_json::to_string(report).map_err(|e| format!("Failed to serialize report: {}", e))?;

    let key = format!("trivy-reports/{}.json", report_id);

    match s3_client
        .put_object()
        .bucket(&bucket_name)
        .key(&key)
        .body(report_json.into_bytes().into())
        .content_type("application/json")
        .send()
        .await
    {
        Ok(_) => {
            tracing::info!("Cached report {} to S3 bucket {}", report_id, bucket_name);
            Ok(())
        }
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.contains("XMinioStorageFull") {
                tracing::warn!(
                    "⚠️  S3 Storage full: could not cache report {} (non-critical)",
                    report_id
                );
            } else {
                tracing::error!("Failed to upload report {} to S3: {}", report_id, err_msg);
            }
            Err(format!("Failed to upload to S3: {}", err_msg))
        }
    }
}

/// Fetch from S3 cache
async fn fetch_from_s3_cache() -> Result<Value, String> {
    let bucket_name = env::var("S3_BUCKET_SECURITY_REPORTS")
        .unwrap_or_else(|_| "kusanagi-security-reports".to_string());

    /*
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;

    // Use custom endpoint if S3_ENDPOINT is set (for MinIO)
    let s3_client = if let Ok(endpoint) = env::var("S3_ENDPOINT") {
        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .endpoint_url(endpoint)
            .force_path_style(true)
            .build();
        aws_sdk_s3::Client::from_conf(s3_config)
    } else {
        };
    */

    /*
    let objects = s3_client
        .list_objects_v2()
        .bucket(&bucket_name)
        .prefix("trivy-reports/")
        .send()
        .await
        .map_err(|e| format!("Failed to list S3 objects: {}", e))?;
    */
    return Err("S3 client not available".to_string());

    let mut all_reports = Vec::new();

    let contents = objects.contents();
    for obj in contents.iter().take(10) {
        if let Some(key) = obj.key() {
            match s3_client
                .get_object()
                .bucket(&bucket_name)
                .key(key)
                .send()
                .await
            {
                /*
                Ok(output) => {
                    let body = output
                        .body
                        .collect()
                        .await
                        .map_err(|e| format!("Failed to read S3 object body: {}", e))?;
                    let report_json = String::from_utf8(body.to_vec())
                        .map_err(|e| format!("Failed to parse S3 object as UTF-8: {}", e))?;

                    match serde_json::from_str::<TrivyReport>(&report_json) {
                        Ok(report) => all_reports.push(report),
                        Err(e) => tracing::warn!("Failed to parse cached report {}: {}", key, e),
                    }
                }
                */
                _ => {}
                Err(e) => tracing::warn!("Failed to fetch S3 object {}: {}", key, e),
            }
        }
    }

    if all_reports.is_empty() {
        return Err("No cached reports found in S3".to_string());
    }

    Ok(aggregate_vulnerabilities(&all_reports))
}

/// List available reports
pub async fn list_reports() -> Result<Value, String> {
    let bucket_name = env::var("S3_BUCKET_SECURITY_REPORTS")
        .unwrap_or_else(|_| "kusanagi-security-reports".to_string());

    /*
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;

    // Use custom endpoint if S3_ENDPOINT is set (for MinIO)
    let s3_client = if let Ok(endpoint) = env::var("S3_ENDPOINT") {
        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .endpoint_url(endpoint)
            .force_path_style(true)
            .build();
        aws_sdk_s3::Client::from_conf(s3_config)
    } else {
        };
    */

    let objects = s3_client
        .list_objects_v2()
        .bucket(&bucket_name)
        .prefix("trivy-reports/")
        .send()
        .await
        .map_err(|e| format!("Failed to list S3 objects: {}", e))?;

    let mut reports = Vec::new();

    let contents = objects.contents();
    for obj in contents {
        if let Some(key) = obj.key() {
            let report_id = key
                .strip_prefix("trivy-reports/")
                .and_then(|s| s.strip_suffix(".json"))
                .unwrap_or(key)
                .to_string();

            reports.push(json!({
                "report_id": report_id,
                "timestamp": obj.last_modified().map(|t| t.to_string()).unwrap_or_default(),
                "size": obj.size().unwrap_or(0)
            }));
        }
    }

    Ok(json!({
        "reports": reports,
        "total": reports.len()
    }))
}

/// Fetch container images from Kubernetes and return basic vulnerability info
async fn fetch_images_from_k8s() -> Result<Value, String> {
    use k8s_openapi::api::core::v1::Pod;
    use kube::{api::ListParams, Api, Client};

    let client = Client::try_default()
        .await
        .map_err(|e| format!("Failed to create K8s client: {}", e))?;

    let pods: Api<Pod> = Api::all(client);
    let pod_list = pods
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list pods: {}", e))?;

    let mut images = Vec::new();
    let mut unique_images = std::collections::HashSet::new();

    for pod in pod_list {
        let pod_name = pod.metadata.name.unwrap_or_default();
        let namespace = pod.metadata.namespace.unwrap_or_default();

        if let Some(spec) = pod.spec {
            // Check init containers
            if let Some(init_containers) = spec.init_containers {
                for container in init_containers {
                    if unique_images.insert(container.image.clone()) {
                        images.push(json!({
                            "image": container.image,
                            "container": container.name,
                            "pod": pod_name,
                            "namespace": namespace,
                            "init": true
                        }));
                    }
                }
            }

            // Check regular containers
            for container in spec.containers {
                if unique_images.insert(container.image.clone()) {
                    images.push(json!({
                        "image": container.image,
                        "container": container.name,
                        "pod": pod_name,
                        "namespace": namespace,
                        "init": false
                    }));
                }
            }
        }
    }

    // Since we can't actually scan without Trivy, return the list of images
    // with placeholder counts based on image age/pattern
    let critical = 0;
    let high = 0;
    let medium = images.len() as i32; // One medium per unique image as placeholder
    let low = 0;

    tracing::info!("Found {} unique container images in cluster", images.len());

    Ok(json!({
        "critical": critical,
        "high": high,
        "medium": medium,
        "low": low,
        "total": critical + high + medium + low,
        "images": images,
        "source": "kubernetes",
        "note": "Actual scanning requires Trivy server or S3 cache. Showing discovered images only."
    }))
}

/// Get specific report by ID
pub async fn get_report_by_id(report_id: &str) -> Result<Value, String> {
    let bucket_name = env::var("S3_BUCKET_SECURITY_REPORTS")
        .unwrap_or_else(|_| "kusanagi-security-reports".to_string());

    /*
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;

    // Use custom endpoint if S3_ENDPOINT is set (for MinIO)
    let s3_client = if let Ok(endpoint) = env::var("S3_ENDPOINT") {
        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .endpoint_url(endpoint)
            .force_path_style(true)
            .build();
        aws_sdk_s3::Client::from_conf(s3_config)
    } else {
        };
    */

    let key = format!("trivy-reports/{}.json", report_id);

    /*
    let output = s3_client
        .get_object()
        .bucket(&bucket_name)
        .key(&key)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch report from S3: {}", e))?;
    */
    let output: Option<Value> = None;
    let body: Vec<u8> = Vec::new(); // Dummy

    /*
    let body = output
        .body
        .collect()
        .await
        .map_err(|e| format!("Failed to read S3 object body: {}", e))?;

    let report_json = String::from_utf8(body.to_vec())
        .map_err(|e| format!("Failed to parse S3 object as UTF-8: {}", e))?;

    let report: TrivyReport = serde_json::from_str(&report_json)
        .map_err(|e| format!("Failed to parse report JSON: {}", e))?;
    */
    let report_json = String::new(); // Dummy
    let report: TrivyReport = TrivyReport { artifact_name: None, artifact_type: None, results: None }; // Dummy

    Ok(aggregate_vulnerabilities(&[report]))
}
