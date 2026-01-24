use aws_sdk_s3::{Client as S3Client, config::Region};
use aws_config::BehaviorVersion;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use reqwest::Client as HttpClient;

fn get_s3_endpoint() -> String {
    std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://192.168.0.170:9010".to_string())
}

fn get_s3_bucket() -> String {
    std::env::var("S3_BUCKET").unwrap_or_else(|_| "kusanagi-news".to_string())
}

fn get_ollama_url() -> String {
    std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://192.168.0.52:11434/api/generate".to_string())
}

fn get_ollama_model() -> String {
    std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "ministral-3:14b".to_string())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Translation {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
}

pub async fn get_s3_client() -> S3Client {
    let access_key = std::env::var("S3_ACCESS_KEY").unwrap_or_default();
    let secret_key = std::env::var("S3_SECRET_KEY").unwrap_or_default();
    
    let credentials = aws_sdk_s3::config::Credentials::new(
        access_key,
        secret_key,
        None,
        None,
        "kusanagi"
    );

    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new("us-east-1"))
        .endpoint_url(get_s3_endpoint())
        .credentials_provider(credentials)
        .load()
        .await;
    S3Client::new(&config)
}

pub async fn ensure_bucket_exists(client: &S3Client) -> Result<(), String> {
    let bucket_name = get_s3_bucket();
    let buckets = client.list_buckets().send().await
        .map_err(|e| format!("Failed to list buckets: {}", e))?;
    
    let exists = buckets.buckets().iter()
        .any(|b| b.name() == Some(&bucket_name));
    
    if !exists {
        info!("Creating bucket: {}", bucket_name);
        client.create_bucket()
            .bucket(bucket_name)
            .send()
            .await
            .map_err(|e| format!("Failed to create bucket: {}", e))?;
    }
    
    Ok(())
}

pub async fn get_cached_translation(s3_client: &S3Client, id: &str) -> Option<Translation> {
    let key = format!("translations/{}.json", id);
    let result = s3_client
        .get_object()
        .bucket(get_s3_bucket())
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
    let key = format!("translations/{}.json", translation.id);
    let body = serde_json::to_string(translation)
        .map_err(|e| format!("Failed to serialize translation: {}", e))?;

    s3_client
        .put_object()
        .bucket(get_s3_bucket())
        .key(&key)
        .body(body.into_bytes().into())
        .send()
        .await
        .map_err(|e| format!("Failed to upload translation to S3: {}", e))?;

    Ok(())
}

pub async fn store_news_item(s3_client: &S3Client, item: &crate::newsfeed::NewsItem) -> Result<(), String> {
    let key = format!("news/{}.json", item.id);
    let body = serde_json::to_string(item)
        .map_err(|e| format!("Failed to serialize news item: {}", e))?;

    s3_client
        .put_object()
        .bucket(get_s3_bucket())
        .key(&key)
        .body(body.into_bytes().into())
        .send()
        .await
        .map_err(|e| format!("Failed to upload news item to S3: {}", e))?;

    Ok(())
}

pub async fn get_news_from_s3(s3_client: &S3Client) -> Result<Vec<crate::newsfeed::NewsItem>, String> {
    let bucket = get_s3_bucket();
    let response = s3_client
        .list_objects_v2()
        .bucket(&bucket)
        .prefix("news/")
        .send()
        .await
        .map_err(|e| format!("Failed to list news in S3: {}", e))?;

    let mut items = Vec::new();
    let objects = response.contents();
    for object in objects {
        if let Some(key) = object.key() {
            if key.ends_with(".json") {
                let result = s3_client
                    .get_object()
                    .bucket(&bucket)
                    .key(key)
                    .send()
                    .await;

                if let Ok(output) = result {
                    if let Ok(data) = output.body.collect().await {
                        if let Ok(item) = serde_json::from_slice::<crate::newsfeed::NewsItem>(&data.into_bytes()) {
                            items.push(item);
                        }
                    }
                }
            }
        }
    }
    
    Ok(items)
}

pub async fn translate_with_ollama(text: &str) -> Result<String, String> {
    let client = HttpClient::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?;

    let prompt = format!(
        "Translate the following technical news text to French. Output ONLY the translated text. Do not include introductory phrases like 'Here is the translation', explanations, or any other additional content.\n\nText: {}",
        text
    );

    let request = serde_json::json!({
        "model": get_ollama_model(),
        "prompt": prompt,
        "stream": false
    });

    let response = client
        .post(get_ollama_url())
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

pub async fn generate_tags_with_ollama(text: &str) -> Result<Vec<String>, String> {
    let client = HttpClient::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?;

    let prompt = format!(
        "Analyze the following technical news and generate descriptive tags in 'key:value' format (e.g., 'category:devops', 'language:rust', 'tool:kubernetes'). Output ONLY the tags, separated by commas. Do not include any other text.\n\nNews: {}",
        text
    );

    let request = serde_json::json!({
        "model": get_ollama_model(),
        "prompt": prompt,
        "stream": false
    });

    let response = client
        .post(get_ollama_url())
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Ollama request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Ollama returned status: {}", response.status()));
    }

    let result: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;
    
    let tags_str = result["response"]
        .as_str()
        .unwrap_or("")
        .trim();
    
    if tags_str.is_empty() {
        return Ok(Vec::new());
    }
    
    let tags = tags_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s.contains(':'))
        .collect();
    
    Ok(tags)
}
