# Resilience Patterns

## Overview

The resilience module provides patterns for building fault-tolerant applications:

- **Circuit Breaker**: Prevent cascading failures by stopping requests to failing services
- **Retry Policies**: Automatic retry with configurable backoff strategies
- **Timeout Handling**: Prevent hanging requests
- **Resilient Client**: Combines circuit breaker, retry, and timeout

## Circuit Breaker

### States

```
CLOSED ──failures──> OPEN ──timeout──> HALF_OPEN ──success──> CLOSED
   ↑                                              └─failure──> OPEN
   │                                                    
   └────────────────────────────────────────────────────┘
```

- **CLOSED**: Normal operation, requests pass through
- **OPEN**: Failure threshold reached, requests fail fast
- **HALF_OPEN**: Testing if service has recovered

### Usage

```rust
use crate::resilience::CircuitBreaker;
use std::time::Duration;

// Create a circuit breaker
let cb = CircuitBreaker::new("prometheus", 5, Duration::from_secs(60));

// Execute operation
match cb.call(|| async {
    fetch_metrics().await
}).await {
    Ok(metrics) => println!("Success: {:?}", metrics),
    Err(e) => println!("Circuit open or error: {}", e),
}
```

### Configuration

```rust
use crate::resilience::{CircuitBreaker, CircuitBreakerConfig};

let config = CircuitBreakerConfig {
    failure_threshold: 5,        // Open after 5 failures
    reset_timeout: Duration::from_secs(60),  // Try half-open after 60s
    success_threshold: 3,        // Close after 3 successes in half-open
    half_open_max_requests: 1,   // Allow 1 request in half-open state
};

let cb = CircuitBreaker::with_config("service", config);
```

### Monitoring

```rust
let metrics = cb.metrics().await;
println!("State: {:?}", metrics.state);
println!("Failures: {}", metrics.failures);
println!("Successes: {}", metrics.successes);
```

## Retry Policies

### Strategies

1. **Fixed Delay**: Constant wait time between retries
2. **Exponential Backoff**: Double wait time after each failure
3. **Custom**: Define your own delay function

### Usage

```rust
use crate::resilience::RetryPolicy;
use std::time::Duration;

// Fixed delay
let policy = RetryPolicy::fixed_delay(3, Duration::from_millis(100));

// Exponential backoff
let policy = RetryPolicy::exponential_backoff(5, Duration::from_millis(100));

// With custom settings
let policy = RetryPolicy::exponential_backoff_with_settings(
    5,                          // max attempts
    Duration::from_millis(100), // initial delay
    Duration::from_secs(10),    // max delay
    2.0,                        // multiplier
    true,                       // jitter
);

// Custom delay function
let policy = RetryPolicy::custom(3, |attempt| {
    Duration::from_millis(attempt as u64 * 100)
});

// Execute with retry
let result = policy.execute(&|| async {
    fetch_data().await
}).await;
```

### Builder Pattern

```rust
use crate::resilience::RetryPolicyBuilder;

let policy = RetryPolicyBuilder::new(5)
    .exponential_backoff(Duration::from_millis(100), Duration::from_secs(10))
    .no_jitter()
    .retry_if(|e| e.is_transient())
    .build();
```

### Context Logging

```rust
let result = policy.execute_with_context("fetch-user-data", &|| async {
    fetch_user().await
}).await;
```

## Timeout

### Usage

```rust
use crate::resilience::Timeout;
use std::time::Duration;

// Simple timeout
let timeout = Timeout::new(Duration::from_secs(5));
let result = timeout.execute(async {
    slow_operation().await
}).await;

// With description
let timeout = Timeout::with_description(Duration::from_secs(5), "database query");
let result = timeout.execute_result(async {
    query_database().await
}).await;
```

### Extension Trait

```rust
use crate::resilience::TimeoutExt;

// Add timeout to any future
let result = async { slow_operation().await }
    .timeout(Duration::from_secs(5))
    .await;

// With description
let result = async { slow_operation().await }
    .timeout_with_description(Duration::from_secs(5), "slow operation")
    .await;
```

### Helper Functions

```rust
use crate::resilience::{with_timeout, with_timeout_result};

// Simple timeout
let result = with_timeout(Duration::from_secs(5), "operation", async {
    slow_operation().await
}).await;

// For fallible operations
let result = with_timeout_result(Duration::from_secs(5), "query", async {
    query_database().await
}).await;
```

## Resilient Client

Combines circuit breaker, retry, and timeout for maximum resilience.

### Usage

```rust
use crate::resilience::ResilientClient;
use std::time::Duration;

// Create client
let client = ResilientClient::new("prometheus")
    .with_timeout(Duration::from_secs(10));

// Execute with full protection
let result = client.execute(|| async {
    fetch_metrics().await
}).await;
```

### Custom Configuration

```rust
use crate::resilience::{ResilientClient, CircuitBreaker, RetryPolicy};

let client = ResilientClient::new("service")
    .with_circuit_breaker(
        CircuitBreaker::new("cb", 3, Duration::from_secs(30))
    )
    .with_retry_policy(
        RetryPolicy::exponential_backoff(3, Duration::from_millis(50))
    )
    .with_timeout(Duration::from_secs(5));
```

## Integration with HTTP Clients

```rust
use crate::resilience::{CircuitBreaker, RetryPolicy, Timeout};

pub struct ResilientHttpClient {
    client: reqwest::Client,
    circuit_breaker: CircuitBreaker,
    retry_policy: RetryPolicy,
    timeout: Timeout,
}

impl ResilientHttpClient {
    pub async fn get(&self, url: &str) -> Result<Response> {
        self.circuit_breaker.call(|| async {
            self.retry_policy.execute(&|| async {
                self.timeout.execute_result(async {
                    self.client.get(url).send().await
                        .map_err(|e| KusanagiError::http(e.to_string()))
                }).await
            }).await
        }).await
    }
}
```

## Best Practices

### 1. Choose Appropriate Timeouts

```rust
// Fast operations: 1-5 seconds
let timeout = Timeout::new(Duration::from_secs(1));

// Normal operations: 10-30 seconds
let timeout = Timeout::new(Duration::from_secs(10));

// Slow operations: 30-60 seconds
let timeout = Timeout::new(Duration::from_secs(30));
```

### 2. Configure Circuit Breaker Based on Error Rate

```rust
// For stable services: higher threshold
let cb = CircuitBreaker::new("stable", 10, Duration::from_secs(60));

// For unstable services: lower threshold
let cb = CircuitBreaker::new("unstable", 3, Duration::from_secs(30));
```

### 3. Use Jitter to Prevent Thundering Herd

```rust
// Always use jitter in production
let policy = RetryPolicy::exponential_backoff_with_settings(
    5,
    Duration::from_millis(100),
    Duration::from_secs(10),
    2.0,
    true, // jitter
);
```

### 4. Only Retry Transient Errors

```rust
// Don't retry validation errors
let policy = RetryPolicy::fixed_delay(3, Duration::from_millis(100))
    .with_retryable_predicate(|e| e.is_transient());
```

### 5. Monitor Circuit Breaker State

```rust
// Log state transitions
tracing::info!(
    circuit_breaker = %cb.name(),
    state = %cb.state().await,
    "Circuit breaker state changed"
);
```

## Testing

Run resilience tests:

```bash
cargo test resilience::
```

All tests:

```bash
cargo test
```

## Statistics

| Pattern | Tests | Coverage |
|---------|-------|----------|
| Circuit Breaker | 9 | State transitions, failure handling |
| Retry | 10 | Fixed, exponential, custom strategies |
| Timeout | 8 | Success, expiration, extensions |
| Resilient Client | 2 | Integration tests |
| **Total** | **29** | **100%** |
