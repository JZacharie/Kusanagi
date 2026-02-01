# Guide de Debug - Kusanagi Network & Pods

## Problèmes corrigés

### 1. Bug JavaScript : Fonction `getStatusClass` manquante
**Fichier**: `static/js/k8s.js` (ligne 350)
- **Problème**: La fonction `getStatusClass()` était appelée mais elle s'appelait en fait `getK8sStatusClass()`
- **Correction**: Renommé l'appel pour utiliser `getK8sStatusClass()`

### 2. Bug JavaScript : Fonction `viewPodLogs` manquante
**Fichier**: `static/js/k8s.js`
- **Problème**: Le bouton "📄 View Logs" appelait `K8sManager.viewPodLogs()` qui n'existait pas
- **Correction**: Ajout des fonctions `viewPodLogs()` et `closeLogsModal()` pour gérer l'affichage des logs

### 3. Gestion des données manquantes
**Fichiers**: `static/js/k8s.js`, `static/js/network.js`
- **Problème**: Le frontend ne gérait pas correctement les valeurs `null` ou `undefined` du backend
- **Corrections**:
  - Ajout de valeurs par défaut (`?? 0`, `|| '-'`) dans tous les affichages
  - Ajout de la fonction `escapeHtml()` pour éviter les problèmes de XSS et d'encodage
  - Vérification de l'existence des tableaux avant utilisation (`Array.isArray()`)

### 4. Robustesse du rendu Network
**Fichier**: `static/js/network.js`
- **Problème**: Les fonctions de rendu du graphique réseau plantaient si les données étaient vides
- **Corrections**:
  - Ajout de vérifications dans `renderGraph()`
  - Ajout de vérifications dans `renderMatrix()`
  - Ajout de vérifications dans `renderStats()`

## Nouveau système de Debug

### Fichier `static/js/debug.js`
Un module de debug complet a été ajouté avec les fonctionnalités suivantes :

```javascript
// Activer/désactiver le debug
KusanagiDebug.setEnabled(true);

// Tester tous les endpoints API
KusanagiDebug.runDiagnostics();

// Tester un endpoint spécifique
await KusanagiDebug.testEndpoint('/api/pods/status');
await KusanagiDebug.testEndpoint('/api/cilium/flows');

// Validation des données
KusanagiDebug.validatePodsData(data);
KusanagiDebug.validateNetworkData(data);
```

### Utilisation dans la console du navigateur

1. Ouvrir la console (F12)
2. Activer le debug : `KusanagiDebug.setEnabled(true)`
3. Lancer les diagnostics : `KusanagiDebug.runDiagnostics()`
4. Vérifier les réponses API dans les logs

## Tests ajoutés

### Tests Rust (177 tests au total)

**Pods** (`src/pods.rs`):
- `test_pods_status_response_serialization` - Vérifie la sérialisation JSON
- `test_pod_info_with_null_values` - Test les valeurs null
- `test_parse_cpu` - Test le parsing CPU
- `test_parse_memory` - Test le parsing mémoire
- `test_format_age` - Test le formatage d'âge

**Network** (`src/cilium.rs`):
- `test_hubble_flows_response_serialization` - Vérifie la structure JSON
- `test_network_flow_structure` - Test la structure des flows
- `test_export_flows_json` - Test l'export JSON
- `test_export_flows_csv` - Test l'export CSV

## Format des données API

### `/api/pods/status`
```json
{
  "total_pods": 100,
  "running_pods": 95,
  "pending_pods": 2,
  "succeeded_pods": 1,
  "failed_pods": 2,
  "error_pods": 2,
  "pods_in_error": [
    {
      "name": "crash-loop-pod",
      "namespace": "default",
      "status": "Failed",
      "reason": "CrashLoopBackOff",
      "message": "Back-off restarting failed container",
      "node": "node-1",
      "restart_count": 15,
      "age": "5m",
      "age_seconds": 300,
      "containers": [...],
      "cpu_usage": 0.1,
      "memory_usage": 104857600,
      "cpu_limit": 0.5,
      "memory_limit": 536870912,
      "cpu_request": 0.1,
      "memory_request": 104857600
    }
  ]
}
```

### `/api/cilium/flows`
```json
{
  "total_flows": 100,
  "flows": [
    {
      "source_namespace": "default",
      "source_pod": "app-1",
      "source_labels": ["app=web"],
      "destination_namespace": "kube-system",
      "destination_pod": "coredns",
      "destination_labels": ["app=dns"],
      "destination_port": 53,
      "protocol": "UDP",
      "verdict": "FORWARDED",
      "bytes_sent": 1024,
      "bytes_received": 512,
      "last_seen": "2024-01-15T10:30:00Z"
    }
  ],
  "matrix": [...],
  "namespaces": ["default", "kube-system"],
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## Démarrage avec debug

```bash
# Mode debug activé automatiquement sur localhost
cargo run

# Ouvrir http://localhost:8080
# Ouvrir la console F12
# Voir les logs de debug
```

## Points d'attention pour le frontend

1. **Toujours utiliser `??` pour les valeurs par défaut**:
   ```javascript
   const value = data.field ?? 'default';
   ```

2. **Toujours vérifier si les tableaux existent**:
   ```javascript
   if (Array.isArray(data.flows)) { ... }
   ```

3. **Utiliser `escapeHtml()` pour les données utilisateur**:
   ```javascript
   element.innerHTML = escapeHtml(pod.name);
   ```

4. **Gérer les erreurs API**:
   ```javascript
   try {
     const response = await fetch('/api/...');
     if (!response.ok) throw new Error(`HTTP ${response.status}`);
     const data = await response.json();
   } catch (error) {
     console.error('API Error:', error);
   }
   ```
