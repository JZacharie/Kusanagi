//! Circuit Breaker pattern implementation
//!
//! The Circuit Breaker prevents cascading failures by stopping requests
//! to a failing service and allowing it time to recover.
//!
//! # States
//!
//! - **Closed**: Normal operation, requests pass through
//! - **Open**: Failure threshold reached, requests fail fast
//! - **Half-Open**: Testing if service has recovered
//!
//! # Example
//!
//! ```rust
//! let cb = CircuitBreaker::new("prometheus", 5, Duration::from_secs(60));
//!
//! match cb.call(|| async { fetch_data().await }).await {
//!     Ok(data) => println!("Success: {:?}", data),
//!     Err(e) => println!("Failed: {}", e),
//! }
//! ```

use crate::error::{KusanagiError, Result};
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Normal operation - requests pass through
    Closed,
    /// Failure threshold reached - requests fail fast
    Open,
    /// Testing if service recovered
    HalfOpen,
}

impl Default for CircuitState {
    fn default() -> Self {
        CircuitState::Closed
    }
}

impl std::fmt::Display for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "CLOSED"),
            CircuitState::Open => write!(f, "OPEN"),
            CircuitState::HalfOpen => write!(f, "HALF_OPEN"),
        }
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Time before attempting to close circuit (half-open)
    pub reset_timeout: Duration,
    /// Number of successes required to close circuit from half-open
    pub success_threshold: u32,
    /// Percentage of requests to allow through in half-open state (0-100)
    pub half_open_max_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            reset_timeout: Duration::from_secs(60),
            success_threshold: 3,
            half_open_max_requests: 1,
        }
    }
}

/// Internal state of the circuit breaker
#[derive(Debug, Default)]
struct CircuitBreakerState {
    state: CircuitState,
    failures: u32,
    successes: u32,
    last_failure_time: Option<Instant>,
    half_open_requests: u32,
}

/// Circuit breaker for resilient operations
///
/// Wraps operations and tracks failures/successes to determine
/// when to stop calling a failing service.
pub struct CircuitBreaker {
    name: String,
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitBreakerState>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default config
    pub fn new(name: impl Into<String>, failure_threshold: u32, reset_timeout: Duration) -> Self {
        let config = CircuitBreakerConfig {
            failure_threshold,
            reset_timeout,
            ..Default::default()
        };
        Self::with_config(name, config)
    }

    /// Create a new circuit breaker with custom config
    pub fn with_config(name: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        Self {
            name: name.into(),
            config,
            state: Arc::new(RwLock::new(CircuitBreakerState::default())),
        }
    }

    /// Get the current state
    pub async fn state(&self) -> CircuitState {
        let state = self.state.read().await;
        state.state
    }

    /// Check if circuit allows requests
    pub async fn allow_request(&self) -> bool {
        let mut state = self.state.write().await;
        
        match state.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if we should transition to half-open
                if let Some(last_failure) = state.last_failure_time {
                    if last_failure.elapsed() >= self.config.reset_timeout {
                        info!(
                            circuit_breaker = %self.name,
                            "Circuit transitioning from OPEN to HALF_OPEN"
                        );
                        state.state = CircuitState::HalfOpen;
                        state.half_open_requests = 0;
                        state.failures = 0;
                        state.successes = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Limit concurrent requests in half-open state
                if state.half_open_requests < self.config.half_open_max_requests {
                    state.half_open_requests += 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Record a successful operation
    async fn record_success(&self) {
        let mut state = self.state.write().await;
        
        match state.state {
            CircuitState::HalfOpen => {
                state.successes += 1;
                state.half_open_requests = state.half_open_requests.saturating_sub(1);
                
                if state.successes >= self.config.success_threshold {
                    info!(
                        circuit_breaker = %self.name,
                        "Circuit transitioning from HALF_OPEN to CLOSED"
                    );
                    state.state = CircuitState::Closed;
                    state.failures = 0;
                    state.successes = 0;
                }
            }
            CircuitState::Closed => {
                state.failures = 0;
            }
            CircuitState::Open => {
                // Shouldn't happen, but reset
                state.half_open_requests = state.half_open_requests.saturating_sub(1);
            }
        }
        
        debug!(
            circuit_breaker = %self.name,
            state = %state.state,
            failures = state.failures,
            successes = state.successes,
            "Recorded success"
        );
    }

    /// Record a failed operation
    async fn record_failure(&self) {
        let mut state = self.state.write().await;
        
        match state.state {
            CircuitState::HalfOpen => {
                warn!(
                    circuit_breaker = %self.name,
                    "Failure in HALF_OPEN state, transitioning to OPEN"
                );
                state.state = CircuitState::Open;
                state.last_failure_time = Some(Instant::now());
                state.half_open_requests = state.half_open_requests.saturating_sub(1);
            }
            CircuitState::Closed => {
                state.failures += 1;
                state.last_failure_time = Some(Instant::now());
                
                if state.failures >= self.config.failure_threshold {
                    warn!(
                        circuit_breaker = %self.name,
                        failures = state.failures,
                        threshold = self.config.failure_threshold,
                        "Circuit transitioning from CLOSED to OPEN"
                    );
                    state.state = CircuitState::Open;
                }
            }
            CircuitState::Open => {
                // Update last failure time
                state.last_failure_time = Some(Instant::now());
            }
        }
        
        debug!(
            circuit_breaker = %self.name,
            state = %state.state,
            failures = state.failures,
            "Recorded failure"
        );
    }

    /// Execute an operation with circuit breaker protection
    pub async fn call<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        // Check if we should allow the request
        if !self.allow_request().await {
            return Err(KusanagiError::internal(format!(
                "Circuit breaker '{}' is OPEN",
                self.name
            )));
        }

        // Execute the operation
        match operation().await {
            Ok(result) => {
                self.record_success().await;
                Ok(result)
            }
            Err(e) => {
                // Only record failure for transient errors
                if e.is_transient() {
                    self.record_failure().await;
                }
                Err(e)
            }
        }
    }

    /// Get metrics for this circuit breaker
    pub async fn metrics(&self) -> CircuitBreakerMetrics {
        let state = self.state.read().await;
        CircuitBreakerMetrics {
            name: self.name.clone(),
            state: state.state,
            failures: state.failures,
            successes: state.successes,
        }
    }
}

/// Metrics for a circuit breaker
#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    pub name: String,
    pub state: CircuitState,
    pub failures: u32,
    pub successes: u32,
}

/// Registry for multiple circuit breakers
pub struct CircuitBreakerRegistry {
    breakers: Arc<RwLock<std::collections::HashMap<String, CircuitBreaker>>>,
}

impl CircuitBreakerRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            breakers: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Register a circuit breaker
    pub async fn register(&self, breaker: CircuitBreaker) {
        let mut breakers = self.breakers.write().await;
        breakers.insert(breaker.name.clone(), breaker);
    }

    /// Get a circuit breaker by name
    pub async fn get(&self, name: &str) -> Option<CircuitBreaker> {
        let breakers = self.breakers.read().await;
        breakers.get(name).cloned()
    }

    /// Get metrics for all circuit breakers
    pub async fn metrics(&self) -> Vec<CircuitBreakerMetrics> {
        let breakers = self.breakers.read().await;
        let mut metrics = Vec::new();
        for breaker in breakers.values() {
            metrics.push(breaker.metrics().await);
        }
        metrics
    }
}

impl Default for CircuitBreakerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CircuitBreaker {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            config: self.config.clone(),
            state: Arc::clone(&self.state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::new("test", 3, Duration::from_secs(60));
        assert_eq!(cb.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let cb = CircuitBreaker::new("test", 3, Duration::from_secs(60));
        
        // Record 3 failures
        for _ in 0..3 {
            let _ = cb.call(|| async {
                Err::<i32, _>(KusanagiError::network("test error"))
            }).await;
        }
        
        // Circuit should be open
        assert_eq!(cb.state().await, CircuitState::Open);
        
        // Next call should fail fast
        let result = cb.call(|| async { Ok(42) }).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("OPEN"));
    }

    #[tokio::test]
    async fn test_circuit_breaker_success_resets_failures() {
        let cb = CircuitBreaker::new("test", 3, Duration::from_secs(60));
        
        // Record 2 failures
        for _ in 0..2 {
            let _ = cb.call(|| async {
                Err::<i32, _>(KusanagiError::network("test error"))
            }).await;
        }
        
        // Then a success
        let _ = cb.call(|| async { Ok(42) }).await;
        
        // Circuit should still be closed
        assert_eq!(cb.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open() {
        let cb = CircuitBreaker::new("test", 2, Duration::from_millis(50));
        
        // Open the circuit
        for _ in 0..2 {
            let _ = cb.call(|| async {
                Err::<i32, _>(KusanagiError::network("test error"))
            }).await;
        }
        
        assert_eq!(cb.state().await, CircuitState::Open);
        
        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Circuit should transition to half-open
        assert!(cb.allow_request().await);
        assert_eq!(cb.state().await, CircuitState::HalfOpen);
        
        // Success should close the circuit
        let _ = cb.call(|| async { Ok(42) }).await;
        
        // Need more successes to close (success_threshold = 3 by default)
        let _ = cb.call(|| async { Ok(42) }).await;
        let _ = cb.call(|| async { Ok(42) }).await;
        
        assert_eq!(cb.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_failure_reopens() {
        let cb = CircuitBreaker::new("test", 2, Duration::from_millis(50));
        
        // Open the circuit
        for _ in 0..2 {
            let _ = cb.call(|| async {
                Err::<i32, _>(KusanagiError::network("test error"))
            }).await;
        }
        
        assert_eq!(cb.state().await, CircuitState::Open);
        
        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Trigger transition to half-open by calling allow_request
        let _ = cb.allow_request().await;
        
        // Circuit should transition to half-open
        assert_eq!(cb.state().await, CircuitState::HalfOpen);
        
        // Failure should reopen
        let _ = cb.call(|| async {
            Err::<i32, _>(KusanagiError::network("test error"))
        }).await;
        
        assert_eq!(cb.state().await, CircuitState::Open);
    }

    #[test]
    fn test_circuit_state_display() {
        assert_eq!(format!("{}", CircuitState::Closed), "CLOSED");
        assert_eq!(format!("{}", CircuitState::Open), "OPEN");
        assert_eq!(format!("{}", CircuitState::HalfOpen), "HALF_OPEN");
    }
}
