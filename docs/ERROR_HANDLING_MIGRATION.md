# Migration Guide: Error Handling with `thiserror`

This document describes how to migrate modules from the old `String`-based error handling to the new structured `KusanagiError` system.

## Overview

The new error system provides:
- **Type-safe errors**: Each error variant has a specific meaning
- **Automatic conversions**: From `kube::Error`, `reqwest::Error`, etc.
- **HTTP status mapping**: Errors automatically map to appropriate HTTP status codes
- **User-friendly messages**: Technical and user-facing error messages
- **Transient error detection**: Know if an error is retryable

## Quick Reference

### Old Pattern
```rust
pub async fn get_data() -> Result<Data, String> {
    let result = some_operation()
        .await
        .map_err(|e| format!("Operation failed: {}", e))?;
    Ok(result)
}
```

### New Pattern
```rust
use crate::error::Result;

pub async fn get_data() -> Result<Data> {
    let result = some_operation().await?; // Automatic conversion
    Ok(result)
}
```

## Migration Steps

### 1. Update Imports

Replace:
```rust
// Old
pub async fn func() -> Result<T, String>
```

With:
```rust
// New
use crate::error::Result;

pub async fn func() -> Result<T>
```

### 2. Remove Manual Error Mapping

Replace:
```rust
let data = client
    .get("url")
    .send()
    .await
    .map_err(|e| format!("Request failed: {}", e))?;
```

With:
```rust
let data = client
    .get("url")
    .send()
    .await?; // reqwest::Error -> KusanagiError automatically
```

### 3. Use Specific Error Variants

Replace:
```rust
if not_found {
    return Err(format!("Resource {} not found", name));
}
```

With:
```rust
if not_found {
    return Err(KusanagiError::not_found("Pod", name));
}
```

## Error Variants by Domain

### Kubernetes Errors
```rust
// Automatic conversion from kube::Error
let pods: Api<Pod> = Api::namespaced(client, namespace);
pods.get(name).await? // Returns KusanagiError::NotFound if 404
```

### HTTP/External API Errors
```rust
// Automatic conversion from reqwest::Error
let response = reqwest::get(url).await?; // Timeout -> KusanagiError::Timeout

// Manual for external APIs
return Err(KusanagiError::external_api("Proxmox", "Connection refused"));
```

### Configuration Errors
```rust
let token = std::env::var("API_TOKEN")
    .map_err(|_| KusanagiError::config("API_TOKEN not set"))?;
```

## HTTP Handler Updates

### Old Handler
```rust
#[get("/api/resource")]
async fn get_resource() -> impl Responder {
    match module::get_data().await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => {
            tracing::error!("Failed: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": e}))
        }
    }
}
```

### New Handler (Module uses KusanagiError)
```rust
#[get("/api/resource")]
async fn get_resource() -> impl Responder {
    match module::get_data().await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => {
            tracing::error!("Failed: {}", e);
            e.error_response() // Uses appropriate HTTP status code
        }
    }
}
```

## Example: Full Module Migration

### Before: `src/events.rs`
```rust
pub async fn get_events(client: &Client) -> Result<EventsResponse, String> {
    let events_api: Api<Event> = Api::all(client.clone());

    let events = events_api
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list events: {}", e))?;
    
    Ok(process_events(events))
}
```

### After: `src/events.rs`
```rust
use crate::error::Result;

pub async fn get_events(client: &Client) -> Result<EventsResponse> {
    let events_api: Api<Event> = Api::all(client.clone());

    // Automatic conversion from kube::Error
    let events = events_api.list(&ListParams::default()).await?;
    
    Ok(process_events(events))
}
```

## Migration Status

| Module | Status | Notes |
|--------|--------|-------|
| `error` | ✅ Migrated | New error types |
| `events` | ✅ Migrated | Uses `KusanagiError` |
| `prometheus` | ✅ Migrated | Uses `KusanagiError` |
| `main` | 🔄 Partial | Handlers updated |
| Other 30 modules | ⏳ Pending | See guide above |

## Benefits

1. **Type Safety**: Can't accidentally return wrong error type
2. **Better Debugging**: Structured logging with error contexts
3. **User Experience**: Appropriate HTTP status codes and friendly messages
4. **Maintainability**: Centralized error definitions
5. **Observability**: Easy to track error rates by variant
