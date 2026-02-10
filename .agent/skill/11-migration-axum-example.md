# Migration Example: Handler Conversion

## Real Example: Weather Handler

### Before (Actix-web)

```rust
// src/interfaces/http/weather_handlers.rs
use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

pub async fn get_weather_handler(
    use_case: web::Data<GetWeatherUseCase>,
    query: web::Query<WeatherQuery>,
) -> impl Responder {
    let input = GetWeatherInput {
        force_refresh: query.refresh,
    };

    match use_case.execute(input).await {
        Ok(weather) => HttpResponse::Ok().json(weather),
        Err(e) => HttpResponse::Ok().json(serde_json::json!({
            "cities": [],
            "error": e.to_string()
        }))
    }
}

// In main.rs
App::new()
    .app_data(web::Data::new(weather_use_case.clone()))
    .route("/api/weather/current", web::get().to(get_weather_handler))
```

### After (Axum)

```rust
// src/interfaces/http/weather_handlers.rs
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

pub async fn get_weather_handler(
    State(use_case): State<Arc<GetWeatherUseCase>>,
    Query(query): Query<WeatherQuery>,
) -> impl IntoResponse {
    let input = GetWeatherInput {
        force_refresh: query.refresh,
    };

    match use_case.execute(input).await {
        Ok(weather) => Json(weather).into_response(),
        Err(e) => Json(serde_json::json!({
            "cities": [],
            "error": e.to_string()
        })).into_response()
    }
}

// In main.rs
#[derive(Clone)]
struct AppState {
    weather_use_case: Arc<GetWeatherUseCase>,
}

let app = Router::new()
    .route("/api/weather/current", get(get_weather_handler))
    .with_state(AppState { weather_use_case });
```

## Conversion Cheat Sheet

| From (Actix) | To (Axum) |
|-------------|-----------|
| `web::Data<T>` | `State<Arc<T>>` |
| `web::Query<T>` | `Query<T>` |
| `web::Path<T>` | `Path<T>` |
| `web::Json<T>` | `Json<T>` |
| `HttpResponse::Ok().json(x)` | `Json(x)` |
| `impl Responder` | `impl IntoResponse` |
| `web::get().to(handler)` | `get(handler)` |
| `App::new()` | `Router::new()` |
| `#[actix_web::main]` | `#[tokio::main]` |

## Quick Conversion Script

```bash
# Replace imports
sed -i 's/use actix_web::/use axum::/g' src/**/*.rs
sed -i 's/web::Data/State/g' src/**/*.rs
sed -i 's/web::Query/Query/g' src/**/*.rs
sed -i 's/web::Path/Path/g' src/**/*.rs
sed -i 's/impl Responder/impl IntoResponse/g' src/**/*.rs

# Replace types
sed -i 's/HttpResponse::Ok().json/Json/g' src/**/*.rs
sed -i 's/.await?/.await/g' src/main.rs  # Axum errors handled differently
```

## Common Compilation Errors

### Error 1: State not Clone
```
error: State does not implement Clone
```
**Fix**: Wrap in Arc: `State<Arc<T>>`

### Error 2: Handler bounds
```
error: the trait bound is not satisfied
```
**Fix**: Ensure handler returns `impl IntoResponse`

### Error 3: Missing extractor
```
error: no function or associated item named `extract` found
```
**Fix**: Import extractors: `use axum::extract::{State, Query};`
