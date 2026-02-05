# ArgoCD Fixes - Kusanagi

## Problèmes Identifiés

### 1. Format de Réponse Incompatible
**Problème:** Le backend retournait un format simplifié qui ne correspondait pas aux attentes du frontend.

**Backend attendait:**
```json
{
  "healthy": true,
  "apps": 5,
  "healthy_apps": 4,
  "synced_apps": 3
}
```

**Frontend attendait:**
```json
{
  "total": 5,
  "healthy": 4,
  "unhealthy": 1,
  "synced": 3,
  "out_of_sync": 2,
  "progressing": 0,
  "upgrades_available": 1,
  "apps_with_issues": [...],
  "apps_with_upgrades": [...]
}
```

### 2. Gestion d'Erreur Insuffisante
- Pas de vérification du statut HTTP
- Pas de gestion des valeurs null/undefined
- Messages d'erreur peu informatifs

### 3. Parsing Incomplet des Applications
- Pas de détails sur chaque application
- Pas de distinction entre issues et upgrades
- Pas d'URL ArgoCD pour chaque app

## Corrections Appliquées

### ✅ Backend: `/src/domain/services/argocd_service.rs`

**Changements:**
1. Nouvelle fonction `parse_argocd_apps()` pour parser correctement les applications
2. Retour du format complet attendu par le frontend
3. Ajout des détails pour chaque application:
   - name, namespace, health_status, sync_status
   - current_revision, argocd_url, message
   - can_sync flag
4. Séparation entre `apps_with_issues` et `apps_with_upgrades`
5. Messages informatifs quand ArgoCD n'est pas accessible

**Format de sortie:**
```rust
{
    "total": items.len(),
    "healthy": healthy,
    "unhealthy": unhealthy,
    "synced": synced,
    "out_of_sync": out_of_sync,
    "progressing": progressing,
    "upgrades_available": apps_with_upgrades.len(),
    "apps_with_issues": [...],
    "apps_with_upgrades": [...],
    "source": "argocd_api" | "kubectl" | "pods_check"
}
```

### ✅ Frontend: `/static/js/k8s.js`

**Changements:**
1. Vérification du statut HTTP de la réponse
2. Gestion des tableaux vides avec `|| []`
3. Valeurs par défaut `|| 0` pour tous les compteurs
4. Messages d'erreur plus détaillés
5. Affichage des messages informatifs du backend

**Améliorations:**
```javascript
// Avant
const data = await response.json();
this.updateArgoIssuesTable(data.apps_with_issues);

// Après
if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
}
const data = await response.json();
this.updateArgoIssuesTable(data.apps_with_issues || []);
```

## Flux de Données

```
1. Frontend: fetch('/api/argocd/status')
   ↓
2. Backend: argocd_status() handler
   ↓
3. Service: argocd_service::get_argocd_status()
   ↓
4. Tentative 1: curl http://localhost:8081/api/v1/applications
   ↓ (si échec)
5. Tentative 2: kubectl get applications -n argocd
   ↓ (si échec)
6. Tentative 3: kubectl get pods -n argocd
   ↓
7. Parse avec parse_argocd_apps()
   ↓
8. Retour JSON formaté
   ↓
9. Frontend: updateArgoStats() + updateArgoIssuesTable()
```

## Fallbacks Implémentés

1. **ArgoCD API** (port 8081) - Méthode préférée
2. **kubectl** - Si API non accessible
3. **Pods check** - Si kubectl échoue mais ArgoCD installé
4. **Empty state** - Message d'erreur clair

## Tests à Effectuer

```bash
# 1. Vérifier que ArgoCD est accessible
kubectl get applications -n argocd

# 2. Tester l'API directement
curl http://localhost:8081/api/v1/applications

# 3. Tester l'endpoint Kusanagi
curl http://localhost:8080/api/argocd/status | jq

# 4. Vérifier les logs backend
# Regarder les logs pour voir quelle méthode est utilisée (source: argocd_api/kubectl/pods_check)
```

## Résultat Attendu

### Si ArgoCD fonctionne:
- ✅ Stats affichées correctement
- ✅ Liste des applications avec issues
- ✅ Liste des upgrades disponibles
- ✅ Liens vers ArgoCD UI fonctionnels
- ✅ Boutons de sync actifs

### Si ArgoCD non accessible:
- ✅ Message d'erreur clair
- ✅ Stats à 0
- ✅ Pas de crash du frontend
- ✅ Message informatif dans la console

## Fichiers Modifiés

1. `/src/domain/services/argocd_service.rs` - Service backend complet
2. `/static/js/k8s.js` - Gestion d'erreur et null safety

## Prochaines Étapes

1. Compiler et redéployer le backend
2. Rafraîchir le frontend
3. Vérifier les logs pour identifier la source de données utilisée
4. Configurer le port-forward si nécessaire:
   ```bash
   kubectl port-forward -n argocd svc/argocd-server 8081:443
   ```
