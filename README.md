# Kusanagi - Kubernetes Monitoring Platform

## Architecture Hexagonale

Plateforme de monitoring Kubernetes avec architecture hexagonale pure.

### Structure

```
kusanagi/
├── src/
│   ├── main.rs              # Application principale
│   ├── lib.rs               # Modules hexagonaux
│   ├── cache.rs             # Cache en mémoire
│   ├── application/         # Couche Application
│   ├── domain/              # Couche Domain
│   ├── infrastructure/      # Couche Infrastructure
│   └── interfaces/          # Couche Interface
└── kusanagi-hexagonal/      # Version complète standalone
```

### Utilisation

```bash
# Version hexagonale complète
cd kusanagi-hexagonal
cargo run --release
```

### Endpoints

- `GET /` - Service information
- `GET /health` - Health check hexagonal
- `GET /metrics` - Métriques Prometheus
- `GET /api/v1/*` - APIs Kubernetes

### Architecture

- **Application Layer**: Use Cases & Business Logic
- **Domain Layer**: Entities & Ports
- **Infrastructure Layer**: Adapters
- **Interface Layer**: HTTP Controllers
