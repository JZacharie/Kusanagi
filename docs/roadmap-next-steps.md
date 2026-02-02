# 🎯 Prochaines Étapes Prioritaires - Kusanagi

## 📋 Ce qui vient immédiatement (Next 30 days)

### 1. Compléter le Refactoring Hexagonal

**Objectif**: Passer de 13/35 (37%) à 20/35 (57%) modules refactorisés

#### Modules prioritaires (P0):
```
- [ ] Cilium (Network)     → Ports: CiliumPort
- [ ] Health System        → Ports: HealthPort
- [ ] Database (PostgreSQL) → Ports: DatabasePort
- [ ] Configuration        → Ports: ConfigPort
```

**Effort estimé**: 2-3 semaines
**Impact**: Meilleure testabilité, maintenabilité

---

### 2. Tests Automatisés

**Objectif**: Atteindre 40% de coverage (vs ~15% actuel)

```rust
// Exemple de test pour les use cases
#[tokio::test]
async fn test_get_nodes_with_disk_metrics() {
    let mock_k8s = Arc::new(MockK8sRepo::with_nodes(vec![test_node()]));
    let mock_metrics = Arc::new(MockMetricsRepo::with_disk_usage(65.4));
    
    let use_case = GetNodesWithDiskMetricsUseCase::new(mock_k8s, mock_metrics);
    let result = use_case.execute().await.unwrap();
    
    assert_eq!(result[0].resources.disk_usage_percent, Some(65.4));
}
```

**Modules à tester prioritairement**:
1. `application/use_cases/node_metrics_use_cases.rs`
2. `application/use_cases/pod_use_cases.rs`
3. `application/use_cases/alert_use_cases.rs`

---

### 3. Feature: SLO/SLI Tracking

**User Story**: En tant qu'admin, je veux définir des SLOs pour mes services et être alerté quand ils ne sont pas atteints.

**Implementation**:
```yaml
# Example SLO Config
slos:
  - name: "API Availability"
    target: 99.9%
    window: 30d
    burn_rate_alerts:
      - name: "Fast Burn"
        multiplier: 14.4
        window: 1h
  
  - name: "Response Time"
    target: 95% < 200ms
    window: 7d
```

**Endpoints API**:
```
GET  /api/slos              # Liste des SLOs
POST /api/slos              # Créer un SLO
GET  /api/slos/{id}/status  # Statut actuel
GET  /api/slos/{id}/burn    # Burn rate
```

---

### 4. Feature: Smart Alerting (Anti-bruit)

**Problème actuel**: ~20 faux positifs/jour

**Solution**: Grouping + Silencing intelligent

```rust
// Alert Grouper
pub struct AlertGrouper {
    // Groupe les alertes par:
    // - Namespace
    // - Type d'erreur
    // - Fenêtre temporelle (5 min)
}

// Exemple: Au lieu de 50 alertes "PodCrash", envoyer 1 alerte groupée
// "15 pods crashing in namespace 'dev' (affected deployments: X, Y, Z)"
```

---

### 5. Documentation API (OpenAPI)

**Générer automatiquement**:
```bash
# Utiliser utoipa pour générer OpenAPI
 cargo doc --open  # Générer docs Rust
# + frontend OpenAPI spec
```

**Swagger UI accessible** à `/api/docs`

---

## 🎨 Améliorations UI/UX Prioritaires

### Dashboard Node - Colonne Disk
```javascript
// Composant React/Vue pour afficher l'usage disque
<NodeDiskCell 
  usage={65.4}
  capacity="100Gi"
  thresholdWarning={75}
  thresholdCritical={90}
/>

// Affichage:
// [███████░░░] 65% (65Gi / 100Gi)
// Couleur: 🟢 vert si < 75%, 🟡 jaune si 75-90%, 🔴 rouge si > 90%
```

### Graphiques Historiques
- Utilisation disque sur 7j/30j
- Corrélation CPU/Memory/Disk
- Prédiction de saturation

---

## 🏗️ Architecture - Prochaines Décisions

### 1. Event Sourcing
```
Problème: Comment tracker l'historique des états?
Solution: Event Store pour les changements critiques

Events:
- NodeStatusChanged { node, from, to, timestamp }
- PodRescheduled { pod, from_node, to_node }
- AlertFired { alert, severity }
```

### 2. CQRS pour les métriques
```
Command Side: Write models (Kubernetes API)
Query Side: Read models (cached, dénormalisés)

Benefits:
- Performances de lecture
- Scalabilité indépendante
```

### 3. Plugin System (Vision v2.0)
```rust
// WASM Plugins
#[wasm_bindgen]
pub trait KusanagiPlugin {
    fn name(&self) -> String;
    fn execute(&self, context: PluginContext) -> PluginResult;
}

// Exemple: Plugin de notification Discord
// Exemple: Plugin de vérification custom
```

---

## 📊 Métriques à Implémenter

### Golden Signals (pour chaque service)
1. **Latency**: Temps de réponse p99
2. **Traffic**: Requêtes/sec
3. **Errors**: Taux d'erreur 5xx
4. **Saturation**: CPU/Memory utilization

### USE Method (pour chaque resource)
1. **Utilization**: % utilisation
2. **Saturation**: Queue length, wait time
3. **Errors**: Error count/rate

### RED Method (pour chaque microservice)
1. **Rate**: Requests per second
2. **Errors**: Error rate
3. **Duration**: Response time distribution

---

## 🚀 Quick Wins (Low Effort, High Impact)

| Feature | Effort | Impact | Fichier |
|---------|--------|--------|---------|
| Health check endpoint (/health) | XS | 🔥🔥🔥 | `src/health.rs` |
| Version API dans réponses | XS | 🔥🔥 | `src/interfaces/http/mod.rs` |
| CORS configuration | XS | 🔥🔥 | `src/main.rs` |
| Request ID logging | S | 🔥🔥 | middleware |
| Alert count badge | S | 🔥🔥🔥 | UI |
| Dark mode toggle | S | 🔥🔥 | CSS |
| Keyboard shortcuts | M | 🔥🔥 | UI |
| Export CSV/JSON | M | 🔥🔥 | handlers |

---

## 🔄 Workflow de Développement Proposé

### 1. Branch Strategy
```
main
├── develop
│   ├── feature/refactor-cilium
│   ├── feature/disk-monitoring
│   └── feature/slo-tracking
├── hotfix/security-patch
└── release/v1.2.0
```

### 2. Definition of Done
- [ ] Code review approuvée
- [ ] Tests unitaires >80% coverage
- [ ] Tests d'intégration passent
- [ ] Documentation à jour
- [ ] Clippy zero warnings
- [ ] CHANGELOG.md mis à jour

### 3. Release Process
1. Version bump dans `Cargo.toml`
2. Tag git `v1.2.0`
3. Build Docker image
4. Release notes
5. Deploy staging → production

---

**Prochaine réunion de planification**: À définir
**Priorité immédiate**: Compléter refactoring Cilium + Tests
