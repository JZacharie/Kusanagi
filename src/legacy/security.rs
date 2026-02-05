use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use std::time::{Duration};
#[cfg(feature = "aws")]
use aws_sdk_s3::Client as S3Client;
#[cfg(feature = "aws")]
use aws_config::meta::region::RegionProviderChain;
#[cfg(feature = "aws")]
use aws_sdk_s3::primitives::ByteStream;
#[cfg(feature = "aws")]
use aws_config::BehaviorVersion;
use actix_web::{get, web, HttpResponse, Responder};

const TRIVY_JSON_SERVER: &str = "http://trivy-json-server.trivy-system.svc:8080";
const OLLAMA_URL: &str = "http://ollama.ollama.svc.cluster.local:11434/api/generate";
const S3_ENDPOINT: &str = "http://192.168.0.170:9010";
const BUCKET_NAME: &str = "kusanagi-security-reports";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SecurityReport {
    pub name: String,
    pub report_type: String,
    pub original_data: serde_json::Value,
    pub enrichment: Option<EnrichmentData>,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnrichmentData {
    pub summary: String,
    pub remediation_advice: String,
    pub criticality_score: f64,
}

#[derive(Deserialize)]
struct IndexEntry {
    name: String,
    #[serde(rename = "type")]
    entry_type: String,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

pub async fn start_security_worker() {
    // Check if running in local mode
    if std::env::var("KUSANAGI_MODE").unwrap_or_default() == "local" {
        info!("🔒 Security worker running in local mode - services mocked");
        return;
    }
    
    info!("🛡️ Starting Security Report enrichment worker");
    
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let creds = aws_sdk_s3::config::Credentials::new("minioadmin", "minioadmin", None, None, "static");
    
    let config = aws_sdk_s3::config::Builder::new()
        .behavior_version(BehaviorVersion::latest())
        .region(region_provider.region().await)
        .credentials_provider(creds)
        .endpoint_url(S3_ENDPOINT)
        .force_path_style(true)
        .build();
    
    let s3_client = S3Client::from_conf(config);
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
        .expect("Failed to create HTTP client");

    let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Every hour

    loop {
        interval.tick().await;
        info!("🔍 Crawling Trivy JSON server...");
        
        if let Err(e) = run_discovery_and_enrichment(&http_client, &s3_client).await {
            error!("❌ Security worker error: {}", e);
        }
    }
}

async fn run_discovery_and_enrichment(http: &reqwest::Client, s3: &S3Client) -> Result<(), String> {
    // 1. Fetch root index
    let categories: Vec<IndexEntry> = http.get(TRIVY_JSON_SERVER)
        .send().await
        .map_err(|e| e.to_string())?
        .json().await
        .map_err(|e| e.to_string())?;

    for cat in categories.iter().filter(|c| c.entry_type == "directory") {
        debug!("Processing category: {}", cat.name);
        
        let files: Vec<IndexEntry> = http.get(format!("{}/{}", TRIVY_JSON_SERVER, cat.name))
            .send().await
            .map_err(|e| e.to_string())?
            .json().await
            .map_err(|e| e.to_string())?;

        for file in files.iter().filter(|f| f.name.ends_with(".json")) {
            if let Err(e) = process_single_report(http, s3, &cat.name, &file.name).await {
                warn!("Failed to process report {}/{}: {}", cat.name, file.name, e);
            }
        }
    }
    
    Ok(())
}

async fn process_single_report(http: &reqwest::Client, s3: &S3Client, cat: &str, filename: &str) -> Result<(), String> {
    let url = format!("{}/{}/{}", TRIVY_JSON_SERVER, cat, filename);
    info!("📄 Processing security report: {}", filename);

    // Fetch report
    let raw_report: serde_json::Value = http.get(&url)
        .send().await
        .map_err(|e| e.to_string())?
        .json().await
        .map_err(|e| e.to_string())?;

    // Check if already enriched in S3 (optional optimization)
    // For now, we overwrite
    
    // 2. Enrich with Ollama
    let enrichment = match enrich_with_ollama(http, &raw_report, "fr").await {
        Ok(data) => Some(data),
        Err(e) => {
            warn!("Ollama enrichment failed for {}: {}", filename, e);
            None
        }
    };

    let report = SecurityReport {
        name: filename.to_string(),
        report_type: cat.to_string(),
        original_data: raw_report,
        enrichment,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // 3. Store in S3
    let json_data = serde_json::to_vec(&report).map_err(|e| e.to_string())?;
    let key = format!("{}/{}", cat, filename);
    
    s3.put_object()
        .bucket(BUCKET_NAME)
        .key(&key)
        .body(ByteStream::from(json_data))
        .content_type("application/json")
        .send().await
        .map_err(|e| e.to_string())?;

    info!("✅ Enriched and stored report: {}", key);
    Ok(())
}

// ============================================================================
// API Handlers
// ============================================================================

async fn get_s3_client() -> S3Client {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let creds = aws_sdk_s3::config::Credentials::new("minioadmin", "minioadmin", None, None, "static");
    
    let config = aws_sdk_s3::config::Builder::new()
        .behavior_version(BehaviorVersion::latest())
        .region(region_provider.region().await)
        .credentials_provider(creds)
        .endpoint_url(S3_ENDPOINT)
        .force_path_style(true)
        .build();
    
    S3Client::from_conf(config)
}

#[get("/reports")]
pub async fn list_enriched_reports() -> impl Responder {
    let s3 = get_s3_client().await;
    
    match s3.list_objects_v2().bucket(BUCKET_NAME).send().await {
        Ok(output) => {
            let reports: Vec<String> = output.contents
                .unwrap_or_default()
                .into_iter()
                .filter_map(|o| o.key)
                .collect();
            HttpResponse::Ok().json(reports)
        }
        Err(e) => {
            error!("Failed to list reports from S3: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() }))
        }
    }
}

#[get("/report/{cat}/{name}")]
pub async fn get_enriched_report(path: web::Path<(String, String)>) -> impl Responder {
    let (cat, name) = path.into_inner();
    let s3 = get_s3_client().await;
    let key = format!("{}/{}", cat, name);
    
    match s3.get_object().bucket(BUCKET_NAME).key(&key).send().await {
        Ok(output) => {
            let data = output.body.collect().await.unwrap().to_vec();
            HttpResponse::Ok()
                .content_type("application/json")
                .body(data)
        }
        Err(e) => {
            error!("Failed to get report {} from S3: {}", key, e);
            HttpResponse::NotFound().json(serde_json::json!({ "error": "Report not found" }))
        }
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/security/enriched")
            .service(list_enriched_reports)
            .service(get_enriched_report)
    );
}

async fn enrich_with_ollama(http: &reqwest::Client, report: &serde_json::Value, lang: &str) -> Result<EnrichmentData, String> {
    // Extract summary for the prompt
    let vuln_count = report["Report"]["Vulnerabilities"].as_array().map(|a| a.len()).unwrap_or(0);
    
    if vuln_count == 0 {
        return Ok(EnrichmentData {
            summary: if lang == "en" { "No vulnerabilities found." } else { "Aucune vulnérabilité trouvée." }.to_string(),
            remediation_advice: if lang == "en" { "System clean. No action required." } else { "Système propre. Aucune action requise." }.to_string(),
            criticality_score: 0.0,
        });
    }

    let sample_vulns = report["Report"]["Vulnerabilities"].as_array().unwrap()
        .iter().take(5)
        .map(|v| format!("- {}: {}", v["VulnerabilityID"], v["Title"].as_str().unwrap_or("No title")))
        .collect::<Vec<_>>().join("\n");

    let lang_instruction = if lang == "en" { "in English" } else { "en Français" };
    let prompt = format!(
        "Analyze this Trivy security report summary:\nTotal Vulnerabilities: {}\nSample:\n{}\n\nProvide a JSON object with: summary (brief, {}), remediation_advice (short action steps, {}), and criticality_score (0-10). Response must be ONLY valid JSON.",
        vuln_count, sample_vulns, lang_instruction, lang_instruction
    );

    let request = OllamaRequest {
        model: "llama3".to_string(), // Adjust based on available models
        prompt,
        stream: false,
    };

    let response = http.post(OLLAMA_URL)
        .json(&request)
        .send().await
        .map_err(|e| e.to_string())?
        .json::<OllamaResponse>().await
        .map_err(|e| e.to_string())?;

    // Attempt to parse Ollama's response as enrichment data
    // LLMs can be unpredictable, so we use a fallback if parsing fails
    match serde_json::from_str::<EnrichmentData>(&response.response) {
        Ok(data) => Ok(data),
        Err(_) => {
            // Fallback: search for JSON block or just use text
            Ok(EnrichmentData {
                summary: "AI analysis completed.".to_string(),
                remediation_advice: response.response,
                criticality_score: if vuln_count > 0 { 7.5 } else { 0.0 },
            })
        }
    }
}
