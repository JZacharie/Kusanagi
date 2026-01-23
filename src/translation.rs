use aws_sdk_s3::{Client as S3Client, config::Region};
use aws_config::BehaviorVersion;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use reqwest::Client as HttpClient;

const MINIO_ENDPOINT: &str = "http://192.168.0.170";
const TRANSLATION_BUCKET: &str = "kusanagi-news-translations";
const OLLAMA_URL: &str = "http://192.168.0.52:11434/api/generate";
const OLLAMA_MODEL: &str = "ministral-3:14b";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Translation {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
}

pub async fn get_s3_client() -> S3Client {
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new("us-east-1"))
        .endpoint_url(MINIO_ENDPOINT)
        .load()
        .await;
    S3Client::new(&config)
}

pub async fn ensure_bucket_exists(client: &S3Client) -> Result<(), String> {
    let buckets = client.list_buckets().send().await
        .map_err(|e| format!("Failed to list buckets: {}", e))?;
    
    let exists = buckets.buckets().iter()
        .any(|b| b.name() == Some(TRANSLATION_BUCKET));
    
    if !exists {
        info!("Creating bucket: {}", TRANSLATION_BUCKET);
        client.create_bucket()
            .bucket(TRANSLATION_BUCKET)
            .send()
            .await
            .map_err(|e| format!("Failed to create bucket: {}", e))?;
    }
    
    Ok(())
}

pub async fn get_cached_translation(s3_client: &S3Client, id: &str) -> Option<Translation> {
    let key = format!("{}.json", id);
    let result = s3_client
        .get_object()
        .bucket(TRANSLATION_BUCKET)
        .key(&key)
        .send()
        .await;

    match result {
        Ok(output) => {
            let data = output.body.collect().await.ok()?;
            serde_json::from_slice(&data.into_bytes()).ok()
        }
        Err(_) => None,
    }
}

pub async fn store_translation(s3_client: &S3Client, translation: &Translation) -> Result<(), String> {
    let key = format!("{}.json", translation.id);
    let body = serde_json::to_string(translation)
        .map_err(|e| format!("Failed to serialize translation: {}", e))?;

    s3_client
        .put_object()
        .bucket(TRANSLATION_BUCKET)
        .key(&key)
        .body(body.into_bytes().into())
        .send()
        .await
        .map_err(|e| format!("Failed to upload translation to S3: {}", e))?;

    Ok(())
}

pub async fn translate_with_ollama(text: &str) -> Result<String, String> {
    let client = HttpClient::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?;

    let prompt = format!(
        "Translate the following technical news text to French. ONLY return the translation, no extra text, no apologies, no conversational filler:\n\n{}",
        text
    );

    let request = serde_json::json!({
        "model": OLLAMA_MODEL,
        "prompt": prompt,
        "stream": false
    });

    let response = client
        .post(OLLAMA_URL)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Ollama request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Ollama returned status: {}", response.status()));
    }

    let result: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;
    
    let translated = result["response"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_string();
    
    if translated.is_empty() {
        return Err("Ollama returned empty translation".to_string());
    }
    
    Ok(translated)
}
