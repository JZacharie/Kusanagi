# Rust Conventions

## Error Handling
```rust
use crate::error::{KusanagiError, Result};

// Return Result<T> for fallible operations
pub async fn get_data() -> Result<Data> {
    match api_call().await {
        Ok(data) => Ok(data),
        Err(e) => Err(KusanagiError::external_service(format!("Error: {}", e)))
    }
}
```

## Handler Signatures
```rust
// CORRECT: Use impl Responder
pub async fn handler(use_case: web::Data<UseCase>) -> impl Responder {
    match use_case.execute().await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => HttpResponse::Ok().json(json!({"error": e.to_string()}))
    }
}

// WRONG: Avoid Result<HttpResponse> or HttpResponse directly
```

## Response Pattern (Always return JSON)
```rust
// Success
HttpResponse::Ok().json(data)

// Error (still 200 OK with JSON error)
HttpResponse::Ok().json(json!({
    "error": "message",
    "data": [],
}))
```

## Async Patterns
```rust
// Use tokio::join! for concurrent operations
let (a, b, c) = tokio::join!(
    fetch_a(),
    fetch_b(),
    fetch_c()
);

// Spawn background tasks
tokio::spawn(async move {
    // background work
});
```

## Tracing
```rust
use tracing::{debug, error, info, warn};

debug!("Debug info: {}", variable);
info!("Service started");
warn!("Cache miss for key: {}", key);
error!("Failed: {}", e);
```

## Feature Flags
Check `Cargo.toml` for features. Key deps:
- `kube`: Kubernetes API
- `reqwest`: HTTP client (native-tls)
- `actix-web`: Web framework
- `serde`: Serialization
