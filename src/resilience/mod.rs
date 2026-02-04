//! Resilience patterns for external service calls
//!
//! This module provides:
//! - Circuit Breaker: Prevent cascading failures
//! - Retry Policies: Automatic retry with backoff
//! - Timeout handling: Prevent hanging requests
//! - Bulkhead pattern: Isolate failures
//!
//! # Example
//!

use crate::error::KusanagiError;
// ```rust
// use crate::resilience::{CircuitBreaker, RetryPolicy};
//
// let cb = CircuitBreaker::new("prometheus", 5, Duration::from_secs(60));
// let retry = RetryPolicy::exponential_backoff(3, Duration::from_millis(100));
//
// let result = cb.call(|| async {
//     retry.execute(|| fetch_metrics()).await
// }).await;
// ```

pub mod circuit_breaker;
pub mod retry;
pub mod timeout;

pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState, CircuitBreakerRegistry, CircuitBreakerMetrics};
pub use retry::{RetryPolicy, RetryPolicyBuilder, RetryStrategy};
pub use timeout::{Timeout, TimeoutFuture, TimeoutExt, with_timeout, with_timeout_result};

use crate::error::Result;
use std::future::Future;
use std::time::Duration;

/// Resilient client wrapper
///
/// Combines circuit breaker and retry for maximum resilience
pub struct ResilientClient {
    circuit_breaker: CircuitBreaker,
    retry_policy: RetryPolicy,
    timeout: Duration,
}

impl ResilientClient {
    /// Create a new resilient client
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            circuit_breaker: CircuitBreaker::new(name, 5, Duration::from_secs(60)),
            retry_policy: RetryPolicy::exponential_backoff(3, Duration::from_millis(100)),
            timeout: Duration::from_secs(30),
        }
    }

    /// Set custom circuit breaker
    pub fn with_circuit_breaker(mut self, cb: CircuitBreaker) -> Self {
        self.circuit_breaker = cb;
        self
    }

    /// Set custom retry policy
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Execute an operation with full resilience
    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        self.circuit_breaker
            .call(|| async {
                self.retry_policy.execute(&operation).await
            })
            .await
    }
}

/// Metrics for resilience operations
#[derive(Debug, Clone, Default)]
pub struct ResilienceMetrics {
    pub circuit_breaker_opens: u64,
    pub circuit_breaker_closes: u64,
    pub retries_total: u64,
    pub retries_successful: u64,
    pub timeouts: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resilient_client_success() {
        let client = ResilientClient::new("test");
        
        let result = client.execute(|| async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_resilient_client_retry_then_success() {
        let client = ResilientClient::new("test")
            .with_retry_policy(RetryPolicy::fixed_delay(3, Duration::from_millis(10)));
        
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let counter_clone = counter.clone();
        
        let result = client.execute(move || {
            let counter = counter_clone.clone();
            async move {
                let attempt = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if attempt < 2 {
                    Err(KusanagiError::network("transient error"))
                } else {
                    Ok(42)
                }
            }
        }).await;
        
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
    }
}
