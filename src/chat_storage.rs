use aws_sdk_s3::{Client, config::Region};
use aws_config::BehaviorVersion;
use serde::Serialize;
use tracing::{info, error};

const MINIO_ENDPOINT: &str = "http://192.168.0.170";
const BUCKET_NAME: &str = "kusanagi-chat-history";

#[derive(Serialize)]
pub struct ChatMessage {
    pub timestamp: String,
    pub user_message: String,
    pub ai_response: String,
    pub response_type: String,
}

pub async fn store_chat_message(user_msg: &str, ai_response: &str, response_type: &str) -> Result<(), String> {
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new("us-east-1")) // MinIO defaults
        .endpoint_url(MINIO_ENDPOINT)
        .load()
        .await;

    let client = Client::new(&config);

    // Ensure bucket exists (simplified, assuming bucket might exist or we just try to upload)
    // For robust prod code we might check/create, but for now we assume it exists or we fail.
    
    let timestamp = chrono::Utc::now().to_rfc3339();
    let message = ChatMessage {
        timestamp: timestamp.clone(),
        user_message: user_msg.to_string(),
        ai_response: ai_response.to_string(),
        response_type: response_type.to_string(),
    };

    let body = serde_json::to_string(&message)
        .map_err(|e| format!("Failed to serialize message: {}", e))?;

    let key = format!("chat-{}.json", timestamp);

    client
        .put_object()
        .bucket(BUCKET_NAME)
        .key(&key)
        .body(body.into_bytes().into())
        .send()
        .await
        .map_err(|e| format!("Failed to upload to S3: {}", e))?;

    info!("Stored chat message to S3: {}", key);
    Ok(())
}
