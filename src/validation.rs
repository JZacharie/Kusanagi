//! Request Validation Module
//!
//! Provides input validation for API endpoints using validator crate.
//! Ensures data integrity and prevents malformed inputs.

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Validation error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErrorResponse {
    pub success: bool,
    pub error: String,
    pub details: Vec<FieldError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

/// Scale request validation
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ScaleRequest {
    #[validate(range(min = 0, max = 100, message = "Replicas must be between 0 and 100"))]
    pub replicas: i32,
}

/// Force delete request validation
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ForceDeleteRequest {
    #[validate(length(min = 1, max = 63, message = "Namespace must be between 1 and 63 characters"))]
    pub namespace: String,
    
    #[validate(length(min = 1, max = 253, message = "Pod name must be between 1 and 253 characters"))]
    pub pod_name: String,
}

/// Sync request validation
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SyncRequest {
    #[validate(length(min = 1, message = "Application name is required"))]
    pub app_name: String,
}

/// Chat request validation
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ChatRequest {
    #[validate(length(min = 1, max = 4000, message = "Message must be between 1 and 4000 characters"))]
    pub message: String,
}

/// Pagination params validation
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct PaginationParams {
    #[validate(range(min = 1, message = "Page must be at least 1"))]
    pub page: Option<usize>,
    
    #[validate(range(min = 1, max = 100, message = "Per page must be between 1 and 100"))]
    pub per_page: Option<usize>,
}

/// MQTT publish request validation
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct MqttPublishRequest {
    #[validate(length(min = 1, max = 255, message = "Topic must be between 1 and 255 characters"))]
    pub topic: String,
    
    #[validate(length(min = 1, max = 65536, message = "Payload must be between 1 and 65536 characters"))]
    pub payload: String,
}

/// Validate and convert result
pub fn validate_request<T: Validate>(data: &T) -> Result<(), ValidationErrorResponse> {
    match data.validate() {
        Ok(()) => Ok(()),
        Err(errors) => {
            let details: Vec<FieldError> = errors
                .field_errors()
                .iter()
                .flat_map(|(field, errs)| {
                    errs.iter().map(move |err| FieldError {
                        field: field.to_string(),
                        message: err.message.clone()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| "Validation failed".to_string()),
                    })
                })
                .collect();
            
            Err(ValidationErrorResponse {
                success: false,
                error: "Validation failed".to_string(),
                details,
            })
        }
    }
}

/// Helper trait for validation
pub trait Validated<T: Validate> {
    fn validate_input(&self) -> Result<(), ValidationErrorResponse>;
}

impl<T: Validate> Validated<T> for T {
    fn validate_input(&self) -> Result<(), ValidationErrorResponse> {
        validate_request(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_request_valid() {
        let req = ScaleRequest { replicas: 3 };
        assert!(req.validate_input().is_ok());
    }

    #[test]
    fn test_scale_request_invalid() {
        let req = ScaleRequest { replicas: -1 };
        assert!(req.validate_input().is_err());
        
        let req = ScaleRequest { replicas: 101 };
        assert!(req.validate_input().is_err());
    }

    #[test]
    fn test_force_delete_request_valid() {
        let req = ForceDeleteRequest {
            namespace: "default".to_string(),
            pod_name: "test-pod".to_string(),
        };
        assert!(req.validate_input().is_ok());
    }

    #[test]
    fn test_force_delete_request_empty() {
        let req = ForceDeleteRequest {
            namespace: "".to_string(),
            pod_name: "test".to_string(),
        };
        assert!(req.validate_input().is_err());
    }

    #[test]
    fn test_chat_request_valid() {
        let req = ChatRequest {
            message: "Hello world".to_string(),
        };
        assert!(req.validate_input().is_ok());
    }

    #[test]
    fn test_chat_request_empty() {
        let req = ChatRequest {
            message: "".to_string(),
        };
        assert!(req.validate_input().is_err());
    }
}
