//! Retry policies for resilient operations
//!
//! Provides different retry strategies:
//! - Fixed delay: Wait a constant time between retries
//! - Exponential backoff: Double the wait time after each failure
//! - Custom: Define your own delay calculation
//!
//! # Example
//!
//! ```rust
//! use crate::resilience::RetryPolicy;
//! use std::time::Duration;
//!
//! let policy = RetryPolicy::exponential_backoff(3, Duration::from_millis(100));
//!
//! let result = policy.execute(|| async {
//!     fetch_data().await
//! }).await;
//! ```

use crate::error::{KusanagiError, Result};
use rand::Rng;
use std::future::Future;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    max_attempts: u32,
    /// Strategy for calculating delay
    strategy: RetryStrategy,
    /// Which errors are retryable
    retryable_predicate: fn(&KusanagiError) -> bool,
}

/// Retry delay strategies
#[derive(Debug, Clone)]
pub enum RetryStrategy {
    /// Fixed delay between retries
    Fixed {
        delay: Duration,
    },
    /// Exponential backoff with optional jitter
    Exponential {
        /// Initial delay
        initial_delay: Duration,
        /// Maximum delay
        max_delay: Duration,
        /// Multiplier for each attempt
        multiplier: f64,
        /// Add random jitter to prevent thundering herd
        jitter: bool,
    },
    /// Custom delay function
    Custom {
        /// Function that returns delay for attempt number
        delay_fn: fn(u32) -> Duration,
    },
}

impl RetryPolicy {
    /// Create a retry policy with fixed delay
    pub fn fixed_delay(max_attempts: u32, delay: Duration) -> Self {
        Self {
            max_attempts,
            strategy: RetryStrategy::Fixed { delay },
            retryable_predicate: |e| e.is_transient(),
        }
    }

    /// Create a retry policy with exponential backoff
    pub fn exponential_backoff(max_attempts: u32, initial_delay: Duration) -> Self {
        Self {
            max_attempts,
            strategy: RetryStrategy::Exponential {
                initial_delay,
                max_delay: Duration::from_secs(60),
                multiplier: 2.0,
                jitter: true,
            },
            retryable_predicate: |e| e.is_transient(),
        }
    }

    /// Create a retry policy with exponential backoff and custom settings
    pub fn exponential_backoff_with_settings(
        max_attempts: u32,
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
        jitter: bool,
    ) -> Self {
        Self {
            max_attempts,
            strategy: RetryStrategy::Exponential {
                initial_delay,
                max_delay,
                multiplier,
                jitter,
            },
            retryable_predicate: |e| e.is_transient(),
        }
    }

    /// Create a retry policy with custom delay function
    pub fn custom(max_attempts: u32, delay_fn: fn(u32) -> Duration) -> Self {
        Self {
            max_attempts,
            strategy: RetryStrategy::Custom { delay_fn },
            retryable_predicate: |e| e.is_transient(),
        }
    }

    /// Set a custom predicate for determining retryable errors
    pub fn with_retryable_predicate(mut self, predicate: fn(&KusanagiError) -> bool) -> Self {
        self.retryable_predicate = predicate;
        self
    }

    /// Calculate delay for a specific attempt
    fn calculate_delay(&self, attempt: u32) -> Duration {
        match &self.strategy {
            RetryStrategy::Fixed { delay } => *delay,
            RetryStrategy::Exponential {
                initial_delay,
                max_delay,
                multiplier,
                jitter,
            } => {
                let base_delay = initial_delay.as_millis() as f64
                    * multiplier.powi(attempt as i32 - 1);
                let delay_ms = base_delay.min(max_delay.as_millis() as f64) as u64;

                if *jitter {
                    // Add random jitter (0-25%)
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let jitter_factor = rng.gen_range(0.0..0.25);
                    let jitter_ms = (delay_ms as f64 * jitter_factor) as u64;
                    Duration::from_millis(delay_ms + jitter_ms)
                } else {
                    Duration::from_millis(delay_ms)
                }
            }
            RetryStrategy::Custom { delay_fn } => delay_fn(attempt),
        }
    }

    /// Execute an operation with retry
    pub async fn execute<F, Fut, T>(&self, operation: &F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut last_error = None;

        for attempt in 1..=self.max_attempts {
            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        info!(
                            attempt = attempt,
                            "Operation succeeded after retries"
                        );
                    }
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e);

                    // Check if error is retryable
                    if !(self.retryable_predicate)(&last_error.as_ref().unwrap()) {
                        debug!("Error is not retryable, failing fast");
                        return Err(last_error.unwrap());
                    }

                    if attempt < self.max_attempts {
                        let delay = self.calculate_delay(attempt);
                        warn!(
                            attempt = attempt,
                            max_attempts = self.max_attempts,
                            ?delay,
                            error = %last_error.as_ref().unwrap(),
                            "Operation failed, retrying"
                        );
                        tokio::time::sleep(delay).await;
                    } else {
                        warn!(
                            attempt = attempt,
                            max_attempts = self.max_attempts,
                            error = %last_error.as_ref().unwrap(),
                            "Operation failed, no more retries"
                        );
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            KusanagiError::internal("Max retries exceeded")
        }))
    }

    /// Execute with a context name for better logging
    pub async fn execute_with_context<F, Fut, T>(
        &self,
        context: &str,
        operation: &F,
    ) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut last_error = None;

        for attempt in 1..=self.max_attempts {
            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        info!(
                            context = context,
                            attempt = attempt,
                            "Operation succeeded after retries"
                        );
                    }
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e);

                    if !(self.retryable_predicate)(&last_error.as_ref().unwrap()) {
                        return Err(last_error.unwrap());
                    }

                    if attempt < self.max_attempts {
                        let delay = self.calculate_delay(attempt);
                        warn!(
                            context = context,
                            attempt = attempt,
                            max_attempts = self.max_attempts,
                            ?delay,
                            error = %last_error.as_ref().unwrap(),
                            "Operation failed, retrying"
                        );
                        tokio::time::sleep(delay).await;
                    } else {
                        warn!(
                            context = context,
                            attempt = attempt,
                            max_attempts = self.max_attempts,
                            error = %last_error.as_ref().unwrap(),
                            "Operation failed, no more retries"
                        );
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| KusanagiError::internal(format!(
            "{}: Max retries exceeded",
            context
        ))))
    }

    /// Get the maximum attempts
    pub fn max_attempts(&self) -> u32 {
        self.max_attempts
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::exponential_backoff(3, Duration::from_millis(100))
    }
}

/// Builder for retry policy
pub struct RetryPolicyBuilder {
    max_attempts: u32,
    strategy: RetryStrategy,
    retryable_predicate: fn(&KusanagiError) -> bool,
}

impl RetryPolicyBuilder {
    /// Create a new builder
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            strategy: RetryStrategy::Fixed {
                delay: Duration::from_millis(100),
            },
            retryable_predicate: |e| e.is_transient(),
        }
    }

    /// Use fixed delay strategy
    pub fn fixed_delay(mut self, delay: Duration) -> Self {
        self.strategy = RetryStrategy::Fixed { delay };
        self
    }

    /// Use exponential backoff strategy
    pub fn exponential_backoff(
        mut self,
        initial_delay: Duration,
        max_delay: Duration,
    ) -> Self {
        self.strategy = RetryStrategy::Exponential {
            initial_delay,
            max_delay,
            multiplier: 2.0,
            jitter: true,
        };
        self
    }

    /// Disable jitter
    pub fn no_jitter(mut self) -> Self {
        if let RetryStrategy::Exponential { ref mut jitter, .. } = self.strategy {
            *jitter = false;
        }
        self
    }

    /// Set custom retry predicate
    pub fn retry_if(mut self, predicate: fn(&KusanagiError) -> bool) -> Self {
        self.retryable_predicate = predicate;
        self
    }

    /// Build the policy
    pub fn build(self) -> RetryPolicy {
        RetryPolicy {
            max_attempts: self.max_attempts,
            strategy: self.strategy,
            retryable_predicate: self.retryable_predicate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_fixed_delay_success() {
        let policy = RetryPolicy::fixed_delay(3, Duration::from_millis(10));
        
        let result = policy.execute(&|| async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_fixed_delay_retry_then_success() {
        let policy = RetryPolicy::fixed_delay(3, Duration::from_millis(10));
        let counter = AtomicU32::new(0);
        
        let result = policy
            .execute(&|| async {
                let attempt = counter.fetch_add(1, Ordering::SeqCst);
                if attempt < 2 {
                    Err(KusanagiError::network("transient"))
                } else {
                    Ok(42)
                }
            })
            .await;
        
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_fixed_delay_max_retries_exceeded() {
        let policy = RetryPolicy::fixed_delay(2, Duration::from_millis(10));
        
        let result = policy
            .execute(&|| async { Err::<i32, _>(KusanagiError::network("always fails")) })
            .await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_non_retryable_error() {
        let policy = RetryPolicy::fixed_delay(3, Duration::from_millis(10));
        let counter = AtomicU32::new(0);
        
        let result = policy
            .execute(&|| async {
                counter.fetch_add(1, Ordering::SeqCst);
                // Validation error is not transient
                Err::<i32, _>(KusanagiError::validation("invalid input"))
            })
            .await;
        
        assert!(result.is_err());
        // Should not retry
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_calculate_delay_fixed() {
        let policy = RetryPolicy::fixed_delay(3, Duration::from_millis(100));
        
        assert_eq!(policy.calculate_delay(1), Duration::from_millis(100));
        assert_eq!(policy.calculate_delay(2), Duration::from_millis(100));
        assert_eq!(policy.calculate_delay(5), Duration::from_millis(100));
    }

    #[test]
    fn test_calculate_delay_exponential() {
        let policy = RetryPolicy::exponential_backoff_with_settings(
            5,
            Duration::from_millis(100),
            Duration::from_secs(10),
            2.0,
            false, // No jitter for predictable tests
        );
        
        assert_eq!(policy.calculate_delay(1), Duration::from_millis(100));
        assert_eq!(policy.calculate_delay(2), Duration::from_millis(200));
        assert_eq!(policy.calculate_delay(3), Duration::from_millis(400));
        assert_eq!(policy.calculate_delay(4), Duration::from_millis(800));
    }

    #[test]
    fn test_calculate_delay_exponential_with_max() {
        let policy = RetryPolicy::exponential_backoff_with_settings(
            10,
            Duration::from_millis(100),
            Duration::from_millis(500),
            2.0,
            false,
        );
        
        // Should be capped at max_delay
        assert_eq!(policy.calculate_delay(10), Duration::from_millis(500));
    }

    #[tokio::test]
    async fn test_builder() {
        let policy = RetryPolicyBuilder::new(5)
            .exponential_backoff(Duration::from_millis(100), Duration::from_secs(10))
            .no_jitter()
            .build();
        
        assert_eq!(policy.max_attempts(), 5);
        
        let result = policy.execute(&|| async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_execute_with_context() {
        let policy = RetryPolicy::fixed_delay(2, Duration::from_millis(10));
        
        let result = policy
            .execute_with_context("test-operation", &|| async { Ok(42) })
            .await;
        
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_custom_delay_function() {
        let policy: RetryPolicy = RetryPolicy::custom(3, |attempt| {
            Duration::from_millis(attempt as u64 * 10)
        });
        
        // 1st attempt: 10ms, 2nd: 20ms, 3rd: 30ms
        assert_eq!(policy.calculate_delay(1), Duration::from_millis(10));
        assert_eq!(policy.calculate_delay(2), Duration::from_millis(20));
        assert_eq!(policy.calculate_delay(3), Duration::from_millis(30));
        
        let result = policy.execute(&|| async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }
}
