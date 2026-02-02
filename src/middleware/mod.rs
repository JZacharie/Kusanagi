//! HTTP Middleware Collection
//!
//! Provides common middleware for the Kusanagi application:
//! - Structured logging with correlation IDs
//! - Rate limiting
//! - Authentication (future)
//! - CORS handling

pub mod logging;
pub mod rate_limit;

pub use logging::{StructuredLogging, CorrelationId, get_correlation_id, CORRELATION_ID_HEADER};
pub use rate_limit::{RateLimiter, RateLimitConfig, KeyExtractor};

use actix_web::{HttpResponse, Error};

/// Common middleware error response
pub fn error_response(status: actix_web::http::StatusCode, message: &str) -> HttpResponse {
    HttpResponse::build(status).json(serde_json::json!({
        "success": false,
        "error": {
            "code": status.as_u16(),
            "message": message,
        },
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
