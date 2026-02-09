# 🔮 Kusanagi

> **"Your effort to remain what you are is what limits you."**  
> A comprehensive Kubernetes monitoring platform inspired by Ghost in the Shell

<p align="center">
  <img src="logo.png" alt="Kusanagi Logo" width="200">
</p>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.70+-orange.svg" alt="Rust"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
  <img src="https://img.shields.io/badge/status-production-green.svg" alt="Status">
  <img src="https://img.shields.io/github/workflow/status/JZacharie/Kusanagi/CI/CD" alt="CI/CD">
  <img src="https://img.shields.io/badge/coverage-15%25-yellow.svg" alt="Coverage">
</p>

<p align="center">
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-features">Features</a> •
  <a href="#-documentation">Documentation</a> •
  <a href="#-contributing">Contributing</a>
</p>

---

## 🚀 Quick Start

### Docker (30 seconds)

```bash
docker run -p 8080:8080 \
  -v ~/.kube:/home/kusanagi/.kube:ro \
  ghcr.io/jzacharie/kusanagi:latest
```

Then open: http://localhost:8080

### Kubernetes (Helm)

```bash
helm repo add kusanagi https://jzacharie.github.io/helmscharts
helm install kusanagi kusanagi/kusanagi \
  --namespace kusanagi \
  --create-namespace
```

### Development

```bash
# Clone
git clone https://github.com/JZacharie/Kusanagi.git
cd Kusanagi

# Run
make run

# Or with hot reload
cargo watch -x run
```

---

## 📸 Preview

<p align="center">
  <img src="docs/images/dashboard-preview.gif" alt="Dashboard Preview" width="800">
</p>

---

## ✨ Features

| Feature | Description | Status |
|---------|-------------|--------|
| 📊 **Real-time Monitoring** | 462 pods, 16 nodes, 447 services | ✅ Live |
| 🔄 **GitOps Integration** | ArgoCD 183 applications | ✅ Live |
| 🔒 **Security Scanning** | Trivy + AI enrichment | ✅ Live |
| 🌐 **Network Observability** | Cilium/Hubble flows | ✅ Live |
| 🤖 **AI Assistant** | LLM-powered cluster analysis | ✅ Live |
| 📱 **PWA Ready** | Mobile-optimized | ✅ Live |

---

## 🏗️ Architecture

```mermaid
graph TB
    subgraph "Interface Layer"
        HTTP[HTTP Handlers]
        WS[WebSocket]
        CLI[CLI]
    end
    
    subgraph "Application Layer"
        UC[Use Cases]
        DTO[DTOs]
    end
    
    subgraph "Domain Layer"
        ENT[Entities]
        SVC[Services]
        PORT[Ports]
    end
    
    subgraph "Infrastructure Layer"
        REPO[Repositories]
        CACHE[Cache]
        EXT[External APIs]
    end
    
    HTTP --> UC
    UC --> ENT
    UC --> SVC
    SVC --> PORT
    PORT --> REPO
    REPO --> EXT
    REPO --> CACHE
```

---

## 📖 Documentation

- [Getting Started](docs/getting-started.md) - Installation et configuration
- [API Reference](https://petstore.swagger.io/?url=https://raw.githubusercontent.com/JZacharie/Kusanagi/main/openapi.json) - Documentation API complète
- [Architecture](src/ARCHITECTURE.md) - Guide d'architecture
- [Contributing](CONTRIBUTING.md) - Guide de contribution

---

## 🔧 Configuration

### Environment Variables

```bash
# Server
BIND_ADDR=0.0.0.0:8080
RUST_LOG=info

# Kubernetes
KUBECONFIG=/path/to/kubeconfig

# Optional: ArgoCD
ARGOCD_SERVER=argocd-server.argocd.svc
ARGOCD_TOKEN=your-token

# Optional: AI/LLM
OLLAMA_URL=http://ollama:11434
OPENAI_API_KEY=sk-...
```

See [Configuration Reference](docs/configuration.md) for all options.

---

## 📊 Performance

- **Memory Usage**: ~50MB baseline
- **Response Time**: <100ms for cached data
- **Concurrent Users**: 100+ supported
- **Cache Hit Rate**: >90%

---

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md).

```bash
# Setup development environment
./scripts/dev-setup.sh

# Run tests
make test

# Format code
make fmt

# Build release
make build
```

### Good First Issues

- [ ] Add unit tests for domain services (#1)
- [ ] Improve error messages (#2)
- [ ] Add more Prometheus metrics (#3)

---

## 📜 License

MIT License - see [LICENSE](LICENSE) for details.

---

<p align="center">
  Built with ❤️ by the community<br>
  <strong>Kusanagi v0.2.0</strong>
</p>
