# Optimisations CPU - Kusanagi

## 🎯 Objectif
Réduire l'utilisation CPU en optimisant les intervalles de polling et les caches.

## 📊 Optimisations Appliquées

### 1. Intervalles de Polling Augmentés

| Composant | Avant | Après | Réduction |
|-----------|-------|-------|-----------|
| Cilium refresh | 45s | 120s | -62% |
| WebSocket heartbeat | 5s | 10s | -50% |
| WebSocket timeout | 10s | 20s | -50% |
| WebSocket alerts | 30s | 60s | -50% |
| MQTT keep-alive | 5s | 30s | -83% |

### 2. Cache TTL Augmentés

| Cache | Avant | Après | Impact |
|-------|-------|-------|--------|
| K8s | 30s | 60s | -50% requêtes |
| ArgoCD | 300s | 600s | -50% requêtes |
| General | 60s | 120s | -50% requêtes |

### 3. Module de Monitoring Ajouté

**Nouveau fichier:** `src/perf_monitor.rs`

Métriques trackées :
- `cache_hits` - Nombre de hits cache
- `cache_misses` - Nombre de misses cache
- `api_calls` - Nombre d'appels API
- `k8s_queries` - Nombre de requêtes Kubernetes

**Endpoint:** `/api/perf/stats` (à implémenter)

Logs automatiques toutes les 60s avec :
- Taux de hit cache (%)
- Nombre d'appels API
- Nombre de requêtes K8s

## 🔍 Analyse des Problèmes Identifiés

### Boucles Infinies Actives
- ✅ Cilium background refresh (optimisé)
- ✅ WebSocket heartbeat (optimisé)
- ✅ MQTT keep-alive (optimisé)
- ⚠️ News feed refresh (1800s - OK)
- ⚠️ Security audit (3600s - OK)
- ⚠️ System check (300s - OK)

### Requêtes Fréquentes
- ✅ Cache K8s avec TTL augmenté
- ✅ Cache ArgoCD avec TTL augmenté
- ✅ Cilium refresh moins fréquent

## 📈 Impact Attendu

**Réduction CPU estimée:** 40-60%

**Calcul:**
- Cilium: 45s → 120s = -62% de cycles
- WebSocket: 5s → 10s = -50% de heartbeats
- MQTT: 5s → 30s = -83% de keep-alives
- Caches: TTL x2 = -50% de requêtes

**Trade-offs:**
- Latence légèrement augmentée (acceptable pour monitoring)
- Données moins fraîches (mais toujours < 2 minutes)
- Meilleure stabilité système

## 🚀 Prochaines Étapes

### Monitoring
1. Déployer et observer l'utilisation CPU
2. Vérifier les logs de perf_monitor
3. Ajuster les intervalles si nécessaire

### Optimisations Futures
1. Implémenter lazy loading pour les données rarement consultées
2. Ajouter un système de backpressure pour les WebSockets
3. Optimiser les requêtes K8s avec des filtres plus précis
4. Implémenter un cache distribué (Redis) si nécessaire

### Debug Avancé
Si CPU toujours élevé :
```bash
# Profiling avec perf
perf record -F 99 -p $(pidof kusanagi) -g -- sleep 30
perf report

# Ou avec cargo flamegraph
cargo install flamegraph
cargo flamegraph --bin kusanagi

# Strace pour voir les syscalls
strace -c -p $(pidof kusanagi)
```

## 📝 Configuration Recommandée

Pour environnement de production :
```rust
// Cache TTL
k8s_cache: 120s      // Données K8s changent peu
argocd_cache: 900s   // ArgoCD encore plus stable
general_cache: 180s  // Cache général

// Polling
cilium_refresh: 180s // Réseau stable
ws_heartbeat: 15s    // Balance latence/CPU
mqtt_keepalive: 60s  // MQTT standard
```

Pour environnement de dev/test :
```rust
// Cache TTL (valeurs actuelles)
k8s_cache: 60s
argocd_cache: 600s
general_cache: 120s

// Polling (valeurs actuelles)
cilium_refresh: 120s
ws_heartbeat: 10s
mqtt_keepalive: 30s
```

## ✅ Tests

Tous les tests passent après optimisations :
- 30 tests unitaires ✅
- Compilation sans erreur ✅
- Formatage correct ✅
- Clippy sans warnings ✅
