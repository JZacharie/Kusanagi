//! Process Audio Use Case
//!
//! Handles the orchestration of audio transcription (ASR) and storage.

use crate::domain::ports::{TranscriptionRepository, NotificationRepository};
use crate::domain::services::llm_service::LlmService;
use crate::error::Result;
use std::sync::Arc;
use tracing::{info, error};

pub struct ProcessAudioUseCase {
    llm_service: Arc<LlmService>,
    transcription_repo: Arc<dyn TranscriptionRepository>,
    notification_repo: Arc<dyn NotificationRepository>,
}

impl ProcessAudioUseCase {
    pub fn new(
        llm_service: Arc<LlmService>,
        transcription_repo: Arc<dyn TranscriptionRepository>,
        notification_repo: Arc<dyn NotificationRepository>,
    ) -> Self {
        Self {
            llm_service,
            transcription_repo,
            notification_repo,
        }
    }

    pub async fn execute(&self, audio_data: Vec<u8>, filename: &str) -> Result<String> {
        info!("Processing audio for ASR: {}", filename);
        
        // 1. Perform ASR
        let asr_result = self.llm_service
            .asr(audio_data, filename)
            .await
            .map_err(|e| {
                error!("ASR failed for {}: {}", filename, e);
                crate::error::KusanagiError::external_service(format!("LiteLLM ASR error: {}", e))
            })?;
        
        info!("ASR successful for {}. Text length: {}", filename, asr_result.text.len());

        // 2. Store transcription in S3
        let storage_key = self.transcription_repo
            .store_transcription(filename, &asr_result.text)
            .await?;
        
        info!("Transcription stored successfully: {}", storage_key);

        // 3. Send notification to external MQTT
        if let Err(e) = self.notification_repo
            .send_message("omnia2/messages", &asr_result.text)
            .await {
                error!("Failed to send MQTT notification: {}", e);
            }
        
        Ok(asr_result.text)
    }
}
