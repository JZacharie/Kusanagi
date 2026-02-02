# Kusanagi - Corrections et Améliorations

## 📋 Résumé des problèmes corrigés

Ce document liste les corrections apportées suite aux erreurs identifiées dans les logs.

---

## ✅ Problèmes Corrigés

### 1. OpenObserve Auth - Récupération depuis K8s Secrets

**Problème** :
```
WARN kusanagi::telemetry: ⏱️ APM: No auth token configured, skipping OpenObserve send
```

**Solution** : 
- ✅ Auto-détection des credentials depuis les secrets Kubernetes
- ✅ Fallback sur les variables d'environnement
- ✅ Secret `openobserve-credentials` créé dans `deploy/rbac-fix.yaml`

**Configuration** :
```yaml
# Via secret K8s
kubectl create secret generic openobserve-credentials \
  --from-literal=endpoint=https://api.openobserve.ai/api/yourorg/yourstream/_json \
  --from-literal=token=your-auth-token \
  -n kusanagi

# Ou via variables d'environnement
export OPENOBSERVE_ENDPOINT=https://api.openobserve.ai/api/default/default/_json
export OPENOBSERVE_AUTH=your-base64-token
```

**Nouveaux endpoints de health check** :
- `GET /health/live` - Liveness probe
- `GET /health/ready` - Readiness probe
- `GET /health/full` - Health check complet avec tous les composants

---

### 2. RBAC - Permission `pods/log` manquante

**Problème** :
```
User "system:serviceaccount:kusanagi:kusanagi" cannot get resource "pods/log" in API group "" in the namespace "kusanagi": Forbidden
```

**Solution** :
- ✅ Ajout de la permission `pods/log` au ClusterRole
- ✅ Fichier `deploy/rbac-fix.yaml` avec toutes les permissions

**Application** :
```bash
kubectl apply -f deploy/rbac-fix.yaml
```

**Permissions ajoutées** :
```yaml
- apiGroups: [""]
  resources: ["pods", "pods/log", "pods/status"]  # <-- pods/log ajouté
  verbs: ["get", "list", "watch"]
```

---

### 3. Timeout K8s API - Augmentation à 30s

**Problème** :
```
ERROR kusanagi::pods: K8s API list pods timed out after 10s
```

**Solution** :
- ✅ Timeout augmenté de 10s à 30s
- ✅ Configuration centralisée via constantes
- ✅ Meilleure gestion des erreurs avec codes HTTP appropriés

**Configuration** (dans `src/pods.rs`) :
```rust
const K8S_API_TIMEOUT_SECS: u64 = 30;
const K8S_LOG_TIMEOUT_SECS: u64 = 15;
```

**Dans `kusanagi.toml`** :
```toml
[kubernetes]
timeout_secs = 30  # Peut être augmenté si nécessaire
```

---

### 4. Support Multi-Provider LLM avec LiteLLM

**Problème** :
```
WARN kusanagi::newsfeed: Failed to generate tags: Ollama request failed: operation timed out
```

**Solution** :
- ✅ Nouveau module `src/llm.rs` supportant multi-providers
- ✅ Support de LiteLLM, Ollama, OpenAI, Anthropic
- ✅ Retry automatique avec backoff
- ✅ Fallback vers Ollama direct si LiteLLM échoue

**Configuration via ConfigMap** :
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: kusanagi-llm-config
  namespace: kusanagi
data:
  LLM_PROVIDER: "litellm"
  LLM_BASE_URL: "http://litellm.default.svc.cluster.local:4000"
  LLM_MODEL: "gpt-3.5-turbo"
  LLM_TIMEOUT_SECS: "60"
  LLM_MAX_RETRIES: "3"
```

**Variables d'environnement** :
```bash
# Provider: litellm, ollama, openai, anthropic
export LLM_PROVIDER=litellm
export LLM_BASE_URL=http://litellm.default.svc.cluster.local:4000
export LLM_MODEL=gpt-3.5-turbo
export LLM_TIMEOUT_SECS=60
export LLM_MAX_RETRIES=3
export LLM_API_KEY=your-api-key  # Pour OpenAI/Anthropic
```

---

## 📁 Fichiers Créés/Modifiés

### Nouveaux fichiers
```
src/
├── llm.rs              # NOUVEAU - Client LLM multi-provider
deploy/
└── rbac-fix.yaml       # NOUVEAU - RBAC corrigé + ConfigMap LLM
```

### Fichiers modifiés
```
src/
├── telemetry.rs        # MODIFIÉ - Auth depuis secrets K8s
├── pods.rs            # MODIFIÉ - Timeouts configurables
├── translation.rs     # MODIFIÉ - Utilise le nouveau module LLM
├── config.rs          # MODIFIÉ - Ajout config LLM
└── main.rs            # MODIFIÉ - Init telemetry
```

---

## 🚀 Déploiement

### 1. Appliquer le RBAC corrigé
```bash
kubectl apply -f deploy/rbac-fix.yaml
```

### 2. Créer le secret OpenObserve
```bash
kubectl create secret generic openobserve-credentials \
  --from-literal=endpoint=https://api.openobserve.ai/api/yourorg/yourstream/_json \
  --from-literal=token=your-auth-token \
  -n kusanagi
```

### 3. Configurer LiteLLM (optionnel)
```bash
# Mettre à jour le ConfigMap avec vos valeurs
kubectl edit configmap kusanagi-llm-config -n kusanagi
```

### 4. Redémarrer Kusanagi
```bash
kubectl rollout restart deployment/kusanagi -n kusanagi
```

---

## 📊 Vérification

### Vérifier les logs
```bash
kubectl logs -n kusanagi deployment/kusanagi | grep -E "(telemetry|RBAC|LLM|timeout)"
```

### Tester l'API de logs
```bash
# Doit maintenant fonctionner sans erreur 403
curl http://kusanagi.p.zacharie.org/api/pods/kusanagi/kusanagi-xxx/logs
```

### Vérifier le health check
```bash
curl http://kusanagi.p.zacharie.org/health/full
```

### Tester la traduction
```bash
# Vérifier la config LLM
curl http://kusanagi.p.zacharie.org/api/config/llm
```

---

## 🔧 Configuration Complète (kusanagi.toml)

```toml
[server]
host = "0.0.0.0"
port = 8080
timeout_secs = 30

[kubernetes]
timeout_secs = 30

[integrations.openobserve]
# Si non défini ici, sera récupéré depuis le secret K8s
# endpoint = "https://api.openobserve.ai/api/default/default/_json"
# auth = "your-token"
sample_rate = 1.0

[integrations.llm]
provider = "litellm"
base_url = "http://litellm.default.svc.cluster.local:4000"
model = "gpt-3.5-turbo"
timeout_secs = 60
max_retries = 3
temperature = 0.7
max_tokens = 2048
```

---

## 📝 Notes

### Compatibilité
- Les anciennes variables `OLLAMA_URL` et `OLLAMA_MODEL` sont toujours supportées (fallback)
- Le module LLM tente d'abord LiteLLM, puis fallback sur Ollama direct

### Sécurité
- Ne jamais commiter les tokens API dans git
- Utiliser toujours les secrets Kubernetes pour les credentials sensibles
- Le ClusterRole a été étendu avec les permissions minimales nécessaires

### Performance
- Timeout K8s API augmenté à 30s pour les clusters lents
- Retry LLM configuré à 3 tentatives avec backoff
- Cache des réponses Prometheus réduit si nécessaire

---

**Date** : 2026-02-02  
**Version** : 0.2.0 → 0.3.0  
**Auteur** : AI Assistant
