# Guide de Migration main.rs

## Objectif
Réduire main.rs de 1,125 lignes à ~300 lignes en utilisant les modules créés.

## Structure Cible

```rust
// main.rs (~300 lignes)
mod config;
mod init;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 1. Init logging & config (20 lignes)
    init::setup_logging();
    let config = config::load_config();
    
    // 2. Init caches & clients (30 lignes)
    let (k8s_cache, argocd_cache, general_cache) = init::setup_caches();
    let (client, kube_client) = init::setup_clients().await;
    let mqtt_state = init::setup_mqtt(&config);
    
    // 3. Start background tasks (20 lignes)
    init::start_background_tasks(&kube_client);
    
    // 4. Configure & start server (30 lignes)
    HttpServer::new(move || {
        App::new()
            .configure(kusanagi::routes::configure_api_routes)
            .configure(kusanagi::routes::configure_k8s_routes)
            .configure(kusanagi::routes::configure_monitoring_routes)
            // ... app_data
    })
    .bind(bind_addr)?
    .run()
    .await
}
```

## Étapes de Migration

### Phase 1: Créer les modules d'initialisation

#### 1.1 Créer `src/init.rs`

```rust
use std::sync::Arc;
use crate::AdvancedCache;

pub fn setup_logging() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();
}

pub fn setup_caches() -> (
    Arc<AdvancedCache<String>>,
    Arc<AdvancedCache<String>>,
    Arc<AdvancedCache<String>>,
) {
    let k8s_cache = Arc::new(AdvancedCache::new(Duration::from_secs(60)));
    let argocd_cache = Arc::new(AdvancedCache::new(Duration::from_secs(600)));
    let general_cache = Arc::new(AdvancedCache::new(Duration::from_secs(120)));
    (k8s_cache, argocd_cache, general_cache)
}

pub async fn setup_clients() -> (reqwest::Client, Option<kube::Client>) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to create HTTP client");
    
    let kube_client = kube::Client::try_default().await.ok();
    (client, kube_client)
}

pub fn setup_mqtt(config: &Config) -> Arc<MqttState> {
    // ... existing MQTT setup code
}

pub fn start_background_tasks(kube_client: &Option<kube::Client>) {
    tokio::spawn(async {
        kusanagi::legacy::alertmanager::start_background_refresh().await;
    });
    
    // ... other background tasks
}
```

#### 1.2 Créer `src/config.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub mqtt: MqttConfig,
    // ... other configs
}

impl Default for AppConfig {
    fn default() -> Self {
        // ... existing default implementation
    }
}

pub fn load_config() -> AppConfig {
    AppConfig::default()
}
```

### Phase 2: Migrer les handlers existants

Pour chaque handler dans main.rs, le déplacer vers le module approprié dans `handlers/`.

#### Exemple: system_status

**Avant** (dans main.rs):
```rust
async fn system_status() -> impl Responder {
    let uptime_secs = START_TIME.get().map(|t| t.elapsed().as_secs()).unwrap_or(0);
    // ... 50 lignes de code
    HttpResponse::Ok().json(json!({ ... }))
}
```

**Après** (dans handlers/system.rs):
```rust
use std::sync::OnceLock;
use std::time::Instant;

static START_TIME: OnceLock<Instant> = OnceLock::new();

pub fn init_start_time() {
    START_TIME.set(Instant::now()).ok();
}

pub async fn system_status(
    general_cache: web::Data<Arc<crate::AdvancedCache<String>>>,
) -> impl Responder {
    // Try cache first
    if let Some(cached) = general_cache.get("system_status").await {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&cached) {
            return HttpResponse::Ok().json(json);
        }
    }

    let uptime_secs = START_TIME.get().map(|t| t.elapsed().as_secs()).unwrap_or(0);
    
    // ... rest of implementation
    
    let response = json!({ ... });
    
    // Cache for 5 seconds
    if let Ok(json_str) = serde_json::to_string(&response) {
        general_cache.set("system_status", json_str).await;
    }
    
    HttpResponse::Ok().json(response)
}
```

### Phase 3: Utiliser routes.rs

**Avant** (dans main.rs):
```rust
App::new()
    .route("/health", web::get().to(health_check))
    .route("/api", web::get().to(service_info))
    .route("/api/system/status", web::get().to(system_status))
    // ... 50+ routes
```

**Après** (dans main.rs):
```rust
App::new()
    .configure(kusanagi::routes::configure_api_routes)
    .configure(kusanagi::routes::configure_k8s_routes)
    .configure(kusanagi::routes::configure_monitoring_routes)
    .configure(kusanagi::routes::configure_static_routes)
```

### Phase 4: Checklist de migration

Pour chaque handler dans main.rs:

- [ ] `health_check` → `handlers/health.rs` ✅ (déjà fait)
- [ ] `service_info` → `handlers/health.rs` ✅ (déjà fait)
- [ ] `cache_stats` → `handlers/cache.rs` ✅ (déjà fait)
- [ ] `system_status` → `handlers/system.rs` (à compléter)
- [ ] `system_logs` → `handlers/system.rs` (à compléter)
- [ ] `get_cluster_overview` → `handlers/k8s.rs` (à compléter)
- [ ] `get_nodes_status` → `handlers/k8s.rs` (à compléter)
- [ ] `get_pods_status` → `handlers/k8s.rs` (à compléter)
- [ ] `get_alerts` → `handlers/monitoring.rs` (à compléter)
- [ ] `get_quotas` → `handlers/monitoring.rs` (à compléter)
- [ ] ... (liste complète à établir)

## Plan d'Exécution

### Semaine 1: Préparation
1. Créer `src/init.rs` avec toutes les fonctions d'initialisation
2. Créer `src/config.rs` avec la configuration
3. Tester que la compilation fonctionne

### Semaine 2: Migration handlers (batch 1)
1. Migrer 5 handlers les plus simples
2. Tester chaque handler individuellement
3. Mettre à jour routes.rs

### Semaine 3: Migration handlers (batch 2)
1. Migrer 5 handlers suivants
2. Tester l'intégration
3. Vérifier que tous les tests passent

### Semaine 4: Finalisation
1. Migrer les handlers restants
2. Nettoyer main.rs
3. Vérifier que main.rs fait ~300 lignes
4. Tests complets
5. Documentation

## Risques & Mitigation

### Risque 1: Régression fonctionnelle
**Mitigation**: 
- Migrer un handler à la fois
- Tester après chaque migration
- Garder une branche de backup

### Risque 2: Dépendances circulaires
**Mitigation**:
- Bien séparer les responsabilités
- Utiliser des traits pour l'abstraction
- Éviter les imports croisés

### Risque 3: État global (START_TIME, etc.)
**Mitigation**:
- Utiliser OnceLock pour l'état global
- Documenter clairement les dépendances
- Initialiser dans le bon ordre

## Commandes Utiles

```bash
# Vérifier la taille de main.rs
wc -l src/main.rs

# Lister tous les handlers dans main.rs
grep -n "^async fn.*-> impl Responder" src/main.rs

# Vérifier que tous les tests passent
cargo test

# Vérifier la compilation
cargo check

# Formater le code
cargo fmt

# Vérifier clippy
cargo clippy
```

## Résultat Attendu

**Avant**:
```
src/main.rs: 1,125 lignes
```

**Après**:
```
src/
  main.rs: ~300 lignes (init + config serveur)
  init.rs: ~150 lignes (initialisation)
  config.rs: ~100 lignes (configuration)
  handlers/: ~500 lignes (handlers métier)
  routes.rs: ~100 lignes (configuration routes)
```

**Total**: ~1,150 lignes (légèrement plus pour la clarté)
**Gain**: Code modulaire, testable, maintenable

## Validation

La migration est réussie quand:
- ✅ main.rs fait moins de 350 lignes
- ✅ Tous les tests passent
- ✅ Aucune régression fonctionnelle
- ✅ Code plus lisible et maintenable
- ✅ Handlers isolés et testables
- ✅ CI/CD passe sans erreur
