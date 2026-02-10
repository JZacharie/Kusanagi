# Backend Architecture

## Hexagonal Architecture Pattern

```
┌──────────────────────────────────────────────┐
│ Interfaces (HTTP Handlers)                   │
│ ├── weather_handlers.rs                      │
│ ├── homeassistant_handlers.rs                │
│ ├── security_handlers.rs                     │
│ ├── alert_handlers.rs                        │
│ └── backup_handlers.rs                       │
├──────────────────────────────────────────────┤
│ Application (Use Cases)                      │
│ ├── GetWeatherUseCase                        │
│ ├── GetAlertsUseCase                         │
│ ├── GetSecurityUseCase                       │
│ └── BackupUseCase                            │
├──────────────────────────────────────────────┤
│ Domain                                       │
│ ├── Entities (WeatherInfo, Alert, etc.)      │
│ ├── Ports (Traits)                           │
│ └── Services (Domain logic)                  │
├──────────────────────────────────────────────┤
│ Infrastructure                               │
│ └── Repositories (impl Ports)                │
└──────────────────────────────────────────────┘
```

## Key Patterns

### Repository Pattern
```rust
// Port (Domain)
#[async_trait]
pub trait WeatherRepository: Send + Sync {
    async fn get_multi_city_weather(&self, force: bool) -> Result<WeatherResponse>;
}

// Implementation (Infrastructure)
pub struct WeatherRepositoryImpl { ... }
#[async_trait]
impl WeatherRepository for WeatherRepositoryImpl { ... }
```

### Use Case Pattern
```rust
pub struct GetWeatherUseCase {
    repository: Arc<dyn WeatherRepository>,
}

impl GetWeatherUseCase {
    pub async fn execute(&self, input: GetWeatherInput) -> Result<WeatherResponse> {
        self.repository.get_multi_city_weather(input.force_refresh).await
    }
}
```

## Caching Strategy
- **Cache**: `AdvancedCache<String>` (in-memory, TTL support)
- **Services/Ingress**: 3 min TTL, refresh on focus
- **Weather**: 1 hour TTL
- **Alerts**: Cache with background refresh
