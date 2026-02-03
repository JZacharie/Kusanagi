//! Rate Limiting Middleware
//!
//! Protects the API from abuse by limiting request rates per client.
//! Supports multiple backends: in-memory (default) and Redis (future).

use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::{Ready, ready, LocalBoxFuture};
use std::collections::HashMap;

use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tracing::{warn, debug};

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed in the window
    pub max_requests: u32,
    /// Time window for rate limiting
    pub window: Duration,
    /// Key extractor function (IP, Header, etc.)
    pub key_extractor: KeyExtractor,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
            key_extractor: KeyExtractor::Ip,
        }
    }
}

/// How to identify clients for rate limiting
#[derive(Debug, Clone)]
pub enum KeyExtractor {
    /// Use client IP address
    Ip,
    /// Use a specific header value (e.g., API key)
    Header(String),
    /// Use both IP and path
    IpAndPath,
}

/// Rate limit state for a single client
#[derive(Debug)]
struct ClientState {
    count: u32,
    window_start: Instant,
}

/// In-memory rate limiter store
#[derive(Debug)]
pub struct RateLimiterStore {
    clients: Mutex<HashMap<String, ClientState>>,
    config: RateLimitConfig,
}

impl RateLimiterStore {
    pub fn new(config: RateLimitConfig) -> Arc<Self> {
        Arc::new(Self {
            clients: Mutex::new(HashMap::new()),
            config,
        })
    }

    /// Clean up old entries periodically
    pub fn cleanup(&self) {
        let mut clients = self.clients.lock().unwrap();
        let now = Instant::now();
        let window = self.config.window;
        clients.retain(|_, state| now.duration_since(state.window_start) < window);
    }
}

/// Rate limiting middleware
#[derive(Debug, Clone)]
pub struct RateLimiter {
    store: Arc<RateLimiterStore>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            store: RateLimiterStore::new(config),
        }
    }

    /// Create with default config (100 req/min)
    pub fn default_per_minute() -> Self {
        Self::new(RateLimitConfig::default())
    }

    /// Create with custom requests per minute
    pub fn per_minute(max_requests: u32) -> Self {
        Self::new(RateLimitConfig {
            max_requests,
            ..Default::default()
        })
    }

    /// Create strict limiter (10 req/min)
    pub fn strict() -> Self {
        Self::per_minute(10)
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimiterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterMiddleware {
            service,
            store: self.store.clone(),
        }))
    }
}

pub struct RateLimiterMiddleware<S> {
    service: S,
    store: Arc<RateLimiterStore>,
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
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
        // Extract client key
        let key = extract_client_key(&req, &self.store.config.key_extractor);
        let now = Instant::now();
        let window = self.store.config.window;
        let max_requests = self.store.config.max_requests;

        // Check rate limit
        let is_limited = {
            let mut clients = self.store.clients.lock().unwrap();
            
            // Cleanup old entries every 100 requests
            if clients.len() % 100 == 0 {
                drop(clients);
                self.store.cleanup();
                clients = self.store.clients.lock().unwrap();
            }

            let client_state = clients.entry(key.clone()).or_insert_with(|| ClientState {
                count: 0,
                window_start: now,
            });

            // Reset window if expired
            if now.duration_since(client_state.window_start) >= window {
                client_state.count = 0;
                client_state.window_start = now;
            }

            // Check limit
            if client_state.count >= max_requests {
                let _retry_after = window.as_secs() - now.duration_since(client_state.window_start).as_secs();
                
                warn!(
                    client_key = %key,
                    count = client_state.count,
                    max_requests = max_requests,
                    "Rate limit exceeded"
                );

                true
            } else {
                // Increment counter
                client_state.count += 1;
                
                debug!(
                    client_key = %key,
                    count = client_state.count,
                    remaining = max_requests - client_state.count,
                    "Request allowed"
                );
                
                false
            }
        };
        
        if is_limited {
            return Box::pin(async move {
                Err(actix_web::error::ErrorTooManyRequests(
                    "Rate limit exceeded. Please try again later."
                ))
            });
        }

        let future = self.service.call(req);
        Box::pin(async move { future.await })
    }
}

fn extract_client_key(req: &ServiceRequest, extractor: &KeyExtractor) -> String {
    match extractor {
        KeyExtractor::Ip => {
            req.connection_info()
                .peer_addr()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        }
        KeyExtractor::Header(header_name) => {
            req.headers()
                .get(header_name)
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    // Fallback to IP if header not present
                    req.connection_info()
                        .peer_addr()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                })
        }
        KeyExtractor::IpAndPath => {
            let ip = req.connection_info()
                .peer_addr()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            format!("{}:{}", ip, req.path())
        }
    }
}

/// Different rate limits for different endpoints
#[derive(Debug, Clone)]
pub struct TieredRateLimiter {
    default: RateLimiter,
    api: RateLimiter,
    strict: RateLimiter,
}

impl TieredRateLimiter {
    pub fn new() -> Self {
        Self {
            default: RateLimiter::per_minute(100),
            api: RateLimiter::per_minute(1000), // API endpoints allow more
            strict: RateLimiter::strict(),      // Sensitive endpoints
        }
    }

    pub fn for_path(path: &str) -> RateLimiter {
        if path.starts_with("/api/") {
            if path.contains("/health") || path.contains("/metrics") {
                // Health checks - very permissive
                RateLimiter::per_minute(10000)
            } else if path.contains("/auth") || path.contains("/login") {
                // Auth endpoints - strict
                RateLimiter::strict()
            } else {
                // Regular API
                RateLimiter::per_minute(1000)
            }
        } else {
            // Static files, etc.
            RateLimiter::per_minute(100)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_requests, 100);
        assert_eq!(config.window, Duration::from_secs(60));
    }

    #[test]
    fn test_client_state_window() {
        let state = ClientState {
            count: 5,
            window_start: Instant::now(),
        };
        assert_eq!(state.count, 5);
    }
}
