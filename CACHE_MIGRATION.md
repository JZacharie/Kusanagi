# Migration vers AdvancedCache - Guide

## 🎯 Objectif

Remplacer le cache simple par le cache avancé avec TTL dans Kusanagi.

## 📝 Exemple de Migration

### Avant (Cache Simple)

```rust
use kusanagi::InMemoryCache;

let cache = InMemoryCache::new();

// Ajouter une valeur
cache.set("pods_status".to_string(), pods_json.to_string()).await;

// Récupérer une valeur
if let Some(cached) = cache.get("pods_status").await {
    return HttpResponse::Ok().json(cached);
}
```

### Après (Cache Avancé)

```rust
use kusanagi::AdvancedCache;
use std::time::Duration;

// Créer le cache avec TTL de 30 secondes pour les données K8s
let cache = AdvancedCache::new(Duration::from_secs(30));

// Ajouter une valeur avec TTL par défaut
cache.set("pods_status".to_string(), pods_json.to_string(), None).await;

// Ajouter une valeur avec TTL personnalisé (5 minutes pour ArgoCD)
cache.set(
    "argocd_status".to_string(), 
    argocd_json.to_string(),
    Some(Duration::from_secs(300))
).await;

// Récupérer une valeur (retourne None si expiré)
if let Some(cached) = cache.get("pods_status").await {
    return HttpResponse::Ok().json(cached);
}
```

## 🔧 Configuration Recommandée par Type de Données

### Données Kubernetes (Haute Fréquence)

```rust
// TTL court: 30 secondes
let k8s_cache = AdvancedCache::new(Duration::from_secs(30));

// Pods, Nodes, Services
k8s_cache.set("pods".to_string(), data, None).await;
```

### Données ArgoCD (Moyenne Fréquence)

```rust
// TTL moyen: 5 minutes
let argocd_cache = AdvancedCache::new(Duration::from_secs(300));

// Applications, Sync status
argocd_cache.set("apps".to_string(), data, None).await;
```

### Données Trivy (Basse Fréquence)

```rust
// TTL long: 1 heure
let trivy_cache = AdvancedCache::new(Duration::from_secs(3600));

// Rapports de sécurité
trivy_cache.set("report_123".to_string(), data, None).await;
```

### Données Statiques (Très Basse Fréquence)

```rust
// TTL très long: 24 heures
let static_cache = AdvancedCache::new(Duration::from_secs(86400));

// News, Configuration
static_cache.set("news".to_string(), data, None).await;
```

## 🚀 Intégration dans main.rs

### Étape 1: Créer les Caches

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Cache pour données K8s (30s)
    let k8s_cache = AdvancedCache::new(Duration::from_secs(30));
    
    // Cache pour données ArgoCD (5min)
    let argocd_cache = AdvancedCache::new(Duration::from_secs(300));
    
    // Cache pour données Trivy (1h)
    let trivy_cache = AdvancedCache::new(Duration::from_secs(3600));
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(k8s_cache.clone()))
            .app_data(web::Data::new(argocd_cache.clone()))
            .app_data(web::Data::new(trivy_cache.clone()))
            .route("/api/pods/status", web::get().to(pods_status))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
```

### Étape 2: Utiliser dans les Handlers

```rust
async fn pods_status(
    cache: web::Data<AdvancedCache<String>>
) -> impl Responder {
    // Vérifier le cache
    if let Some(cached) = cache.get("pods_status").await {
        return HttpResponse::Ok()
            .insert_header(("X-Cache", "HIT"))
            .json(cached);
    }
    
    // Récupérer les données
    match kubernetes_service::get_pods_status().await {
        Ok(pods) => {
            let json_str = serde_json::to_string(&pods).unwrap();
            
            // Mettre en cache
            cache.set("pods_status".to_string(), json_str.clone(), None).await;
            
            HttpResponse::Ok()
                .insert_header(("X-Cache", "MISS"))
                .json(pods)
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e}))
    }
}
```

## 📊 Monitoring du Cache

### Endpoint de Statistiques

```rust
async fn cache_stats(
    k8s_cache: web::Data<AdvancedCache<String>>,
    argocd_cache: web::Data<AdvancedCache<String>>,
    trivy_cache: web::Data<AdvancedCache<String>>,
) -> impl Responder {
    let k8s_stats = k8s_cache.stats().await;
    let argocd_stats = argocd_cache.stats().await;
    let trivy_stats = trivy_cache.stats().await;
    
    HttpResponse::Ok().json(json!({
        "k8s": {
            "entries": k8s_stats.entries,
            "expired": k8s_stats.expired,
            "memory_bytes": k8s_stats.memory_bytes,
        },
        "argocd": {
            "entries": argocd_stats.entries,
            "expired": argocd_stats.expired,
            "memory_bytes": argocd_stats.memory_bytes,
        },
        "trivy": {
            "entries": trivy_stats.entries,
            "expired": trivy_stats.expired,
            "memory_bytes": trivy_stats.memory_bytes,
        }
    }))
}
```

### Métriques Prometheus

```rust
use prometheus::{IntGauge, Registry};

lazy_static! {
    static ref CACHE_ENTRIES: IntGauge = IntGauge::new(
        "kusanagi_cache_entries_total",
        "Total number of cache entries"
    ).unwrap();
    
    static ref CACHE_EXPIRED: IntGauge = IntGauge::new(
        "kusanagi_cache_expired_total",
        "Total number of expired cache entries"
    ).unwrap();
}

// Dans une tâche périodique
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    loop {
        interval.tick().await;
        let stats = cache.stats().await;
        CACHE_ENTRIES.set(stats.entries as i64);
        CACHE_EXPIRED.set(stats.expired as i64);
    }
});
```

## 🎯 Avantages

### Performance

- ✅ Réduction de la charge sur l'API Kubernetes
- ✅ Temps de réponse plus rapide (cache hit)
- ✅ Moins de requêtes réseau

### Fiabilité

- ✅ Données toujours fraîches (TTL)
- ✅ Cleanup automatique (pas de fuite mémoire)
- ✅ Fallback gracieux si cache expiré

### Observabilité

- ✅ Statistiques détaillées
- ✅ Métriques Prometheus
- ✅ Headers X-Cache pour debugging

## 📈 Résultats Attendus

### Avant (Sans Cache)

- Temps de réponse: 200-500ms
- Charge API K8s: 100%
- Latence réseau: Impact direct

### Après (Avec Cache)

- Temps de réponse: 5-20ms (cache hit)
- Charge API K8s: 10-20% (selon TTL)
- Latence réseau: Impact minimal

### Ratio Hit/Miss Attendu

- Pods/Nodes: 80-90% hit rate
- ArgoCD: 90-95% hit rate
- Trivy: 95-99% hit rate

## 🔄 Migration Progressive

### Phase 1: Endpoints Critiques

1. `/api/pods/status`
2. `/api/nodes/status`
3. `/api/cluster/overview`

### Phase 2: Endpoints Secondaires

1. `/api/argocd/status`
2. `/api/services`
3. `/api/storage`

### Phase 3: Endpoints Lents

1. `/api/security/vulnerabilities`
2. `/api/security/reports`
3. `/api/cilium/flows`

## ✅ Checklist de Migration

- [ ] Créer les instances de cache avec TTL appropriés
- [ ] Ajouter les caches dans App::new()
- [ ] Modifier les handlers pour utiliser le cache
- [ ] Ajouter les headers X-Cache
- [ ] Créer l'endpoint /api/cache/stats
- [ ] Ajouter les métriques Prometheus
- [ ] Tester les performances
- [ ] Monitorer le hit rate
- [ ] Ajuster les TTL si nécessaire
- [ ] Documenter les changements

## 🎉 Conclusion

Le cache avancé avec TTL améliore significativement les performances et la fiabilité de Kusanagi tout en réduisant la charge sur les APIs externes.

**Prêt à migrer !** 🚀
