//! Standardized API Response Module
//!
//! Provides consistent response formats across all API endpoints:
//! - Success responses with data
//! - Error responses with detailed information
//! - Paginated responses
//! - Empty responses

use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use crate::error::KusanagiError;

/// Standard API response wrapper
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
}

/// API Error details
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Response metadata (pagination, etc.)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_pages: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Create a success response with data
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: Some(ResponseMeta {
                timestamp: Some(chrono::Utc::now().to_rfc3339()),
                ..Default::default()
            }),
        }
    }

    /// Create a success response with data and pagination
    pub fn success_paginated(data: T, page: usize, per_page: usize, total: usize) -> Self {
        let total_pages = (total as f64 / per_page as f64).ceil() as usize;
        
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: Some(ResponseMeta {
                page: Some(page),
                per_page: Some(per_page),
                total: Some(total),
                total_pages: Some(total_pages),
                timestamp: Some(chrono::Utc::now().to_rfc3339()),
            }),
        }
    }

    /// Create an error response
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code.into(),
                message: message.into(),
                details: None,
            }),
            meta: Some(ResponseMeta {
                timestamp: Some(chrono::Utc::now().to_rfc3339()),
                ..Default::default()
            }),
        }
    }

    /// Create an error response with details
    pub fn error_with_details(
        code: impl Into<String>,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code.into(),
                message: message.into(),
                details: Some(details),
            }),
            meta: Some(ResponseMeta {
                timestamp: Some(chrono::Utc::now().to_rfc3339()),
                ..Default::default()
            }),
        }
    }

    /// Convert to HttpResponse
    pub fn into_http_response(self, status_code: actix_web::http::StatusCode) -> HttpResponse {
        HttpResponse::build(status_code).json(self)
    }
}

impl Default for ResponseMeta {
    fn default() -> Self {
        Self {
            page: None,
            per_page: None,
            total: None,
            total_pages: None,
            timestamp: None,
        }
    }
}

/// Helper trait for converting results to standardized responses
pub trait IntoApiResponse<T> {
    fn into_api_response(self) -> ApiResponse<T>;
}

impl<T: Serialize> IntoApiResponse<T> for Result<T, KusanagiError> {
    fn into_api_response(self) -> ApiResponse<T> {
        match self {
            Ok(data) => ApiResponse::success(data),
            Err(e) => ApiResponse::error(
                format!("{:?}", e),
                e.to_string(),
            ),
        }
    }
}

/// Standard empty response for operations that don't return data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmptyResponse {
    pub message: String,
}

impl EmptyResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Common response helpers
pub mod helpers {
    use super::*;

    /// Create a 200 OK response with data
    pub fn ok<T: Serialize>(data: T) -> HttpResponse {
        ApiResponse::success(data).into_http_response(actix_web::http::StatusCode::OK)
    }

    /// Create a 201 Created response
    pub fn created<T: Serialize>(data: T) -> HttpResponse {
        ApiResponse::success(data).into_http_response(actix_web::http::StatusCode::CREATED)
    }

    /// Create a 204 No Content response
    pub fn no_content() -> HttpResponse {
        HttpResponse::NoContent().finish()
    }

    /// Create a 400 Bad Request response
    pub fn bad_request(message: impl Into<String>) -> HttpResponse {
        ApiResponse::<()>::error("BAD_REQUEST", message)
            .into_http_response(actix_web::http::StatusCode::BAD_REQUEST)
    }

    /// Create a 401 Unauthorized response
    pub fn unauthorized(message: impl Into<String>) -> HttpResponse {
        ApiResponse::<()>::error("UNAUTHORIZED", message)
            .into_http_response(actix_web::http::StatusCode::UNAUTHORIZED)
    }

    /// Create a 403 Forbidden response
    pub fn forbidden(message: impl Into<String>) -> HttpResponse {
        ApiResponse::<()>::error("FORBIDDEN", message)
            .into_http_response(actix_web::http::StatusCode::FORBIDDEN)
    }

    /// Create a 404 Not Found response
    pub fn not_found(resource: impl Into<String>) -> HttpResponse {
        ApiResponse::<()>::error("NOT_FOUND", format!("Resource '{}' not found", resource.into()))
            .into_http_response(actix_web::http::StatusCode::NOT_FOUND)
    }

    /// Create a 409 Conflict response
    pub fn conflict(message: impl Into<String>) -> HttpResponse {
        ApiResponse::<()>::error("CONFLICT", message)
            .into_http_response(actix_web::http::StatusCode::CONFLICT)
    }

    /// Create a 422 Unprocessable Entity response
    pub fn unprocessable(message: impl Into<String>) -> HttpResponse {
        ApiResponse::<()>::error("UNPROCESSABLE_ENTITY", message)
            .into_http_response(actix_web::http::StatusCode::UNPROCESSABLE_ENTITY)
    }

    /// Create a 500 Internal Server Error response
    pub fn internal_error(message: impl Into<String>) -> HttpResponse {
        ApiResponse::<()>::error("INTERNAL_ERROR", message)
            .into_http_response(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Create a 503 Service Unavailable response
    pub fn service_unavailable(message: impl Into<String>) -> HttpResponse {
        ApiResponse::<()>::error("SERVICE_UNAVAILABLE", message)
            .into_http_response(actix_web::http::StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// Pagination parameters extractor
#[derive(Debug, Deserialize, Clone)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_per_page")]
    pub per_page: usize,
}

fn default_page() -> usize {
    1
}

fn default_per_page() -> usize {
    20
}

impl PaginationParams {
    /// Validate and sanitize pagination parameters
    pub fn sanitized(&self) -> Self {
        Self {
            page: self.page.max(1),
            per_page: self.per_page.clamp(1, 100),
        }
    }

    /// Calculate offset for database queries
    pub fn offset(&self) -> usize {
        (self.page.saturating_sub(1)) * self.per_page
    }

    /// Calculate limit for database queries
    pub fn limit(&self) -> usize {
        self.per_page
    }
}

/// Sorting parameters
#[derive(Debug, Deserialize, Clone)]
pub struct SortParams {
    #[serde(default)]
    pub sort_by: Option<String>,
    #[serde(default = "default_sort_order")]
    pub sort_order: SortOrder,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

fn default_sort_order() -> SortOrder {
    SortOrder::Asc
}

impl SortOrder {
    pub fn to_sql(&self) -> &'static str {
        match self {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("test data");
        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response = ApiResponse::<()>::error("TEST_ERROR", "Test error message");
        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.message, "Test error message");
    }

    #[test]
    fn test_pagination_params() {
        let params = PaginationParams { page: 2, per_page: 50 };
        let sanitized = params.sanitized();
        assert_eq!(sanitized.page, 2);
        assert_eq!(sanitized.per_page, 50);
        assert_eq!(sanitized.offset(), 50);
        assert_eq!(sanitized.limit(), 50);
    }

    #[test]
    fn test_pagination_params_bounds() {
        let params = PaginationParams { page: 0, per_page: 200 };
        let sanitized = params.sanitized();
        assert_eq!(sanitized.page, 1);  // Min is 1
        assert_eq!(sanitized.per_page, 100);  // Max is 100
    }
}
