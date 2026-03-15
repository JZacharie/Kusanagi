//! S3 Transcription Repository Implementation
//!
//! Infrastructure adapter for storing transcriptions in S3.

use crate::domain::ports::TranscriptionRepository;
use crate::error::{KusanagiError, Result};
use async_trait::async_trait;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use std::sync::Arc;
use tracing::{info, error};

pub struct S3TranscriptionRepository {
    client: Client,
    bucket: String,
}

impl S3TranscriptionRepository {
    pub fn new(client: Client, bucket: String) -> Self {
        Self { client, bucket }
    }
}

#[async_trait]
impl TranscriptionRepository for S3TranscriptionRepository {
    async fn store_transcription(&self, filename: &str, text: &str) -> Result<String> {
        let key = format!("transcriptions/{}.txt", filename);
        
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(text.as_bytes().to_vec()))
            .content_type("text/plain")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to upload transcription to S3: {}", e);
                KusanagiError::external_service(format!("S3 error: {}", e))
            })?;

        info!("Successfully stored transcription in S3: {}", key);
        Ok(key)
    }
}
