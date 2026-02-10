# Plan d'Action : Migration Actix-web → Axum

## Résumé Exécutif

**Durée estimée**: 3 jours  
**Risque**: Moyen (WebSockets complexes)  
**Bénéfice attendu**: +25% perf, -28% latency, meilleure compatibilité Tower

## Architecture Cible

```
src/
├── main.rs                    # tokio::main + serve()
├── state.rs                   # AppState struct centralisée
├── router.rs                  # Route definitions
├── middleware.rs              # CORS, Compression, Trace
└── handlers/                  # Tous les handlers migrés
```

## Plan Détaillé

### Phase 1: Setup (Jour 1 - Matin)
- [ ] Créer branche `migration/axum`
- [ ] Modifier `Cargo.toml` (dépendances)
- [ ] Créer `src/state.rs` avec `AppState`
- [ ] Créer `src/router.rs` avec routes vides

### Phase 2: Handlers Core (Jour 1 - Après-midi)
- [ ] Migrer `weather_handlers.rs`
- [ ] Migrer `homeassistant_handlers.rs`
- [ ] Migrer `security_handlers.rs`
- [ ] Tests unitaires

### Phase 3: Infrastructure (Jour 2)
- [ ] Migrer `alert_handlers.rs`
- [ ] Migrer `backup_handlers.rs`
- [ ] Migrer legacy handlers
- [ ] Ajouter middlewares (CORS, Compression)

### Phase 4: WebSockets (Jour 3 - Matin)
- [ ] Migrer WebSocket notifications
- [ ] Tester connectivité temps réel
- [ ] Gérer reconnexion client

### Phase 5: Finalisation (Jour 3 - Après-midi)
- [ ] Migrer `main.rs` (bootstrap)
- [ ] Tests d'intégration complets
- [ ] Benchmark performance
- [ ] Documentation

## Changements Clés

### Dépendances
```toml
[dependencies]
# Ajouter
axum = { version = "0.7", features = ["ws"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["fs", "cors", "compression"] }

# Retirer
actix-web = "4.12"
actix-files = "0.6"
actix-web-actors = "4.3"
```

### State Centralisé
```rust
#[derive(Clone)]
pub struct AppState {
    pub weather_use_case: Arc<GetWeatherUseCase>,
    pub alerts_use_case: Arc<GetAlertsUseCase>,
    pub security_use_case: Arc<GetSecurityUseCase>,
    pub ha_use_case: Arc<GetHomeAssistantUseCase>,
    pub k8s_cache: Arc<AdvancedCache<String>>,
    pub config: Config,
    pub http_client: reqwest::Client,
}
```

### Main.rs Minimal
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = create_state().await;
    let app = create_router(state);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

## Points d'Attention

1. **WebSockets**: Actix utilise actors, Axum utilise callbacks
2. **State**: Doit implémenter `Clone` (utiliser `Arc`)
3. **Erreurs**: Axum utilise `IntoResponse`, pas de `Result<HttpResponse>`
4. **Static files**: `tower-http::ServeDir` remplace `actix-files`

## Validation

```bash
# Compilation
cargo build --release

# Tests
cargo test --release

# Bench
cargo bench  # Si disponible

# Check mémoire
valgrind --tool=massif target/release/kusanagi
```

## Rollback

```bash
# Si problème
git checkout main
git branch -D migration/axum
```

## Ressources

- Plan détaillé: `.agent/skill/10-migration-actix-to-axum.md`
- Exemples de code: `.agent/skill/11-migration-axum-example.md`
- Doc Axum: https://docs.rs/axum/latest/axum/
