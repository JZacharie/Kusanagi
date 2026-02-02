# Disk Monitoring Feature

## Overview

Cette fonctionnalité permet de monitorer l'utilisation du disque des nodes du cluster Kubernetes via Prometheus et de l'afficher dans l'interface Kusanagi.

## Architecture

### Entités

Le modèle `NodeResources` a été étendu pour inclure :

```rust
pub struct NodeResources {
    // ... champs existants ...
    pub disk_capacity: Option<String>,           // Capacité totale du disque (ephemeral-storage)
    pub disk_allocatable: Option<String>,        // Espace allocatable
    pub disk_usage_percent: Option<f64>,         // Pourcentage d'utilisation (depuis Prometheus)
    pub ephemeral_storage_capacity: Option<String>,
    pub ephemeral_storage_allocatable: Option<String>,
}
```

### Use Cases

#### 1. GetNodesWithDiskMetricsUseCase
Récupère la liste des nodes avec les métriques de disque depuis Prometheus.

**Métriques Prometheus utilisées :**
- `node_filesystem_size_bytes` - Taille totale du filesystem
- `node_filesystem_avail_bytes` - Espace disponible
- `node_filesystem_free_bytes` - Espace libre

**Calcul :**
```
disk_usage_percent = 100 - ((avail / size) * 100)
```

#### 2. GetNodeDiskUsageUseCase
Récupère les métriques détaillées pour un node spécifique.

**Retourne :**
- `usage_percent` - Pourcentage d'utilisation
- `total_gb` - Espace total en GiB
- `used_gb` - Espace utilisé en GiB
- `free_gb` - Espace libre en GiB

#### 3. GetClusterDiskSummaryUseCase
Récupère un résumé du disque pour tout le cluster.

**Retourne :**
- `average_usage_percent` - Moyenne d'utilisation
- `max_usage_percent` - Utilisation maximale
- `total_storage_tb` - Stockage total en TiB
- `status` - État (Healthy/Warning/Critical)

## API Endpoints

### GET /api/nodes/with-metrics
Retourne la liste des nodes avec le pourcentage d'utilisation du disque.

**Exemple de réponse :**
```json
[
  {
    "name": "node-1",
    "status": "Ready",
    "cpu_capacity": "4",
    "memory_capacity": "16Gi",
    "disk_capacity": "100Gi",
    "disk_usage_percent": 65.4,
    "pod_count": 15
  }
]
```

### GET /api/nodes/{name}/disk
Retourne les métriques détaillées du disque pour un node.

**Exemple de réponse :**
```json
{
  "node_name": "node-1",
  "usage_percent": 65.4,
  "total_gb": "100.0 Gi",
  "used_gb": "65.4 Gi",
  "free_gb": "34.6 Gi"
}
```

### GET /api/nodes/disk-summary
Retourne un résumé du disque pour tout le cluster.

**Exemple de réponse :**
```json
{
  "average_usage_percent": 58.2,
  "max_usage_percent": 87.5,
  "total_storage_tb": "2.5 Ti",
  "status": "Warning"
}
```

## Intégration avec l'UI

Pour afficher l'utilisation du disque dans le menu Nodes de Kusanagi :

1. Utiliser l'endpoint `/api/nodes/with-metrics` à la place de `/api/nodes`
2. Afficher la colonne `disk_usage_percent` avec un code couleur :
   - 🟢 < 75% : Healthy
   - 🟡 75-90% : Warning
   - 🔴 > 90% : Critical

## Configuration Prometheus

Les métriques utilisées nécessitent le node-exporter déployé sur les nodes :

```yaml
# ServiceMonitor pour node-exporter
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: node-exporter
spec:
  selector:
    matchLabels:
      app: node-exporter
  endpoints:
    - port: metrics
      interval: 30s
```

## Requêtes Prometheus

### Utilisation moyenne du cluster
```promql
avg(100 - ((node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"}) * 100))
```

### Utilisation par node
```promql
100 - ((node_filesystem_avail_bytes{mountpoint="/",instance=~"NODE_NAME:.*"} / node_filesystem_size_bytes{mountpoint="/",instance=~"NODE_NAME:.*"}) * 100)
```

### Nodes avec > 80% d'utilisation
```promql
100 - ((node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"}) * 100) > 80
```

## Dépannage

### disk_usage_percent est null
Vérifier que :
1. Prometheus est accessible
2. Le node-exporter est déployé sur les nodes
3. Les métriques `node_filesystem_*_bytes` sont disponibles

### Valeurs incorrectes
Vérifier le point de montage utilisé :
- Par défaut : `mountpoint="/"`
- Pour des montages spécifiques, ajuster la requête Prometheus

## Futures améliorations

- [ ] Alertes automatiques quand l'utilisation dépasse 90%
- [ ] Graphiques historiques d'utilisation
- [ ] Prévisions de remplissage
- [ ] Support pour plusieurs filesystems par node
