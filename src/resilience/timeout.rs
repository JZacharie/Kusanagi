//! Timeout handling for async operations
//!
//! Provides timeout wrappers for async operations to prevent hanging.
//!
//! # Example
//!
//! ```rust
//! use crate::resilience::Timeout;
//! use std::time::Duration;
//!
//! let timeout = Timeout::new(Duration::from_secs(5));
//!
//! match timeout.execute(async { slow_operation().await }).await {
//!     Ok(result) => println!("Success: {:?}", result),
//!     Err(e) => println!("Timeout: {}", e),
//! }
//! ```

use crate::error::{KusanagiError, Result};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::{sleep, Sleep};
use tracing::warn;

/// Timeout wrapper for async operations
pub struct Timeout {
    duration: Duration,
    description: String,
}

impl Timeout {
    /// Create a new timeout
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            description: "operation".to_string(),
        }
    }

    /// Create a new timeout with a description
    pub fn with_description(duration: Duration, description: impl Into<String>) -> Self {
        Self {
            duration,
            description: description.into(),
        }
    }

    /// Execute a future with timeout
    pub async fn execute<F, T>(&self, future: F) -> Result<T>
    where
        F: Future<Output = T>,
    {
        match tokio::time::timeout(self.duration, future).await {
            Ok(result) => Ok(result),
            Err(_) => {
                warn!(
                    operation = %self.description,
                    timeout_secs = self.duration.as_secs(),
                    "Operation timed out"
                );
                Err(KusanagiError::timeout(
                    self.duration.as_secs(),
                    &self.description,
                ))
            }
        }
    }

    /// Execute a fallible future with timeout
    pub async fn execute_result<F, T>(&self, future: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        match tokio::time::timeout(self.duration, future).await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(e)) => Err(e),
            Err(_) => {
                warn!(
                    operation = %self.description,
                    timeout_secs = self.duration.as_secs(),
                    "Operation timed out"
                );
                Err(KusanagiError::timeout(
                    self.duration.as_secs(),
                    &self.description,
                ))
            }
        }
    }

    /// Get the timeout duration
    pub fn duration(&self) -> Duration {
        self.duration
    }
}

/// Future that times out after a duration
pub struct TimeoutFuture<F> {
    future: F,
    sleep: Pin<Box<Sleep>>,
}

impl<F> TimeoutFuture<F> {
    /// Create a new timeout future
    pub fn new(future: F, timeout: Duration) -> Self {
        Self {
            future,
            sleep: Box::pin(sleep(timeout)),
        }
    }
}

impl<F: Future> Future for TimeoutFuture<F> {
    type Output = Result<F::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        
        // Poll the main future
        let future = unsafe { Pin::new_unchecked(&mut this.future) };
        match future.poll(cx) {
            Poll::Ready(result) => Poll::Ready(Ok(result)),
            Poll::Pending => {
                // Check if timeout has expired
                match Pin::new(&mut this.sleep).poll(cx) {
                    Poll::Ready(_) => Poll::Ready(Err(KusanagiError::timeout(
                        0,
                        "operation timed out",
                    ))),
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }
}

/// Extension trait for adding timeout to futures
pub trait TimeoutExt: Future + Sized {
    /// Add a timeout to this future
    fn timeout(self, duration: Duration) -> TimeoutFuture<Self> {
        TimeoutFuture::new(self, duration)
    }

    /// Add a timeout with description
    fn timeout_with_description(
        self,
        duration: Duration,
        description: impl Into<String>,
    ) -> TimeoutFutureWithDescription<Self> {
        TimeoutFutureWithDescription::new(self, duration, description)
    }
}

impl<T: Future + Sized> TimeoutExt for T {}

/// Timeout future with description
pub struct TimeoutFutureWithDescription<F> {
    future: F,
    sleep: Pin<Box<Sleep>>,
    description: String,
    timeout_secs: u64,
}

impl<F> TimeoutFutureWithDescription<F> {
    /// Create a new timeout future with description
    pub fn new(future: F, timeout: Duration, description: impl Into<String>) -> Self {
        Self {
            future,
            sleep: Box::pin(sleep(timeout)),
            description: description.into(),
            timeout_secs: timeout.as_secs(),
        }
    }
}

impl<F: Future> Future for TimeoutFutureWithDescription<F> {
    type Output = Result<F::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        
        // Poll the main future
        let future = unsafe { Pin::new_unchecked(&mut this.future) };
        match future.poll(cx) {
            Poll::Ready(result) => Poll::Ready(Ok(result)),
            Poll::Pending => {
                // Check if timeout has expired
                match Pin::new(&mut this.sleep).poll(cx) {
                    Poll::Ready(_) => {
                        warn!(
                            operation = %this.description,
                            timeout_secs = this.timeout_secs,
                            "Operation timed out"
                        );
                        Poll::Ready(Err(KusanagiError::timeout(
                            this.timeout_secs,
                            &this.description,
                        )))
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }
}

/// Execute a function with timeout
pub async fn with_timeout<F, T>(duration: Duration, description: impl Into<String>, f: F) -> Result<T>
where
    F: Future<Output = T>,
{
    Timeout::with_description(duration, description).execute(f).await
}

/// Execute a fallible function with timeout
pub async fn with_timeout_result<F, T>(
    duration: Duration,
    description: impl Into<String>,
    f: F,
) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    Timeout::with_description(duration, description)
        .execute_result(f)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timeout_success() {
        let timeout = Timeout::new(Duration::from_millis(100));
        
        let result = timeout.execute(async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            42
        }).await;
        
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_timeout_expires() {
        let timeout = Timeout::new(Duration::from_millis(10));
        
        let result = timeout.execute(async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            42
        }).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Timeout"));
    }

    #[tokio::test]
    async fn test_timeout_result_success() {
        let timeout = Timeout::new(Duration::from_millis(100));
        
        let result = timeout.execute_result(async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok::<i32, KusanagiError>(42)
        }).await;
        
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_timeout_result_error() {
        let timeout = Timeout::new(Duration::from_millis(100));
        
        let result = timeout.execute_result(async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Err::<i32, _>(KusanagiError::internal("test error"))
        }).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("test error"));
    }

    #[tokio::test]
    async fn test_timeout_result_timeout() {
        let timeout = Timeout::new(Duration::from_millis(10));
        
        let result = timeout.execute_result(async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok::<i32, KusanagiError>(42)
        }).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Timeout"));
    }

    #[tokio::test]
    async fn test_timeout_extension() {
        use crate::resilience::TimeoutExt;
        
        let result = async { 42 }
            .timeout(Duration::from_millis(100))
            .await;
        
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_timeout_extension_with_description() {
        use crate::resilience::TimeoutExt;
        
        let result = async { 42 }
            .timeout_with_description(Duration::from_millis(100), "test operation")
            .await;
        
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_helper_functions() {
        let result = with_timeout(
            Duration::from_millis(100),
            "test",
            async { 42 }
        ).await;
        
        assert_eq!(result.unwrap(), 42);
        
        let result = with_timeout_result(
            Duration::from_millis(100),
            "test",
            async { Ok::<i32, KusanagiError>(42) }
        ).await;
        
        assert_eq!(result.unwrap(), 42);
    }
}
