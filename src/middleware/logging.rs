//! Structured Logging Middleware with Correlation IDs
//!
//! Provides request tracing with unique correlation IDs for distributed tracing.

use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{Ready, ready, LocalBoxFuture};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;
use tracing::{info, warn, error, Instrument, Span};
use uuid::Uuid;

pub const CORRELATION_ID_HEADER: &str = "x-correlation-id";
pub const REQUEST_ID_HEADER: &str = "x-request-id";

#[derive(Debug, Clone)]
pub struct CorrelationId(pub String);

/// Structured logging middleware
#[derive(Debug, Clone)]
pub struct StructuredLogging;

impl StructuredLogging {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StructuredLogging {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for StructuredLogging
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = StructuredLoggingMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(StructuredLoggingMiddleware { service }))
    }
}

pub struct StructuredLoggingMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for StructuredLoggingMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let correlation_id = req
            .headers()
            .get(CORRELATION_ID_HEADER)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let request_id = Uuid::new_v4().to_string();
        req.extensions_mut().insert(CorrelationId(correlation_id.clone()));

        let method = req.method().to_string();
        let path = req.path().to_string();
        let remote_addr = req.connection_info().peer_addr().map(|s| s.to_string());
        let user_agent = req
            .headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        let span = tracing::info_span!(
            "http_request",
            correlation_id = %correlation_id,
            request_id = %request_id,
            method = %method,
            path = %path,
        );

        let start = Instant::now();

        info!(
            target: "http_request_start",
            correlation_id = %correlation_id,
            request_id = %request_id,
            method = %method,
            path = %path,
            remote_addr = ?remote_addr,
            user_agent = %user_agent,
            "→ Request started"
        );

        let future = self.service.call(req);

        Box::pin(async move {
            let result = future.await;
            let duration = start.elapsed();

            match &result {
                Ok(response) => {
                    let status = response.status();
                    let status_code = status.as_u16();

                    if status.is_success() {
                        info!(
                            target: "http_request_success",
                            correlation_id = %correlation_id,
                            request_id = %request_id,
                            status = status_code,
                            duration_ms = duration.as_millis() as u64,
                            "← Request completed"
                        );
                    } else if status.is_client_error() {
                        warn!(
                            target: "http_request_client_error",
                            correlation_id = %correlation_id,
                            request_id = %request_id,
                            status = status_code,
                            duration_ms = duration.as_millis() as u64,
                            "← Client error"
                        );
                    } else if status.is_server_error() {
                        error!(
                            target: "http_request_server_error",
                            correlation_id = %correlation_id,
                            request_id = %request_id,
                            status = status_code,
                            duration_ms = duration.as_millis() as u64,
                            "← Server error"
                        );
                    }
                }
                Err(e) => {
                    error!(
                        target: "http_request_error",
                        correlation_id = %correlation_id,
                        request_id = %request_id,
                        error = %e,
                        duration_ms = duration.as_millis() as u64,
                        "← Request error"
                    );
                }
            }

            let mut response = result?;
            if let Ok(header_value) = correlation_id.parse() {
                response.headers_mut().insert(
                    CORRELATION_ID_HEADER.parse().unwrap(),
                    header_value,
                );
            }

            Ok(response)
        }.instrument(span))
    }
}

pub fn get_correlation_id(req: &actix_web::HttpRequest) -> Option<String> {
    req.extensions()
        .get::<CorrelationId>()
        .map(|cid| cid.0.clone())
}
