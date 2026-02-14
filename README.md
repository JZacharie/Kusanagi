# Kusanagi v0.3.0 🔮

> **"Your effort to remain what you are is what limits you."**  
> A comprehensive Kubernetes monitoring platform inspired by Ghost in the Shell

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-production-green.svg)](https://github.com/JZacharie/Kusanagi)

## 🎯 Overview

Kusanagi is a production-ready Kubernetes monitoring platform built with Rust and Axum. It provides real-time monitoring, GitOps integration, security scanning, and multi-infrastructure support through a modern web interface.

### Key Features

- 🚀 **Real-time Monitoring**: 462 pods, 16 nodes, 447 services
- 🔄 **GitOps Integration**: 183 ArgoCD applications
- 🔒 **Security Scanning**: Trivy vulnerability reports with AI enrichment
- 🌐 **Multi-Infrastructure**: Kubernetes, Proxmox, Home Assistant
- 📊 **Network Observability**: Cilium/Hubble flow monitoring
- 💬 **AI Assistant**: LLM-powered cluster analysis (Ollama/OpenAI/Anthropic)
- 📱 **PWA Ready**: Mobile-optimized progressive web app
- 🎨 **Modern UI**: Neo-Glassmorphism design with dark theme

## 📊 Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Kusanagi Platform                        │
├─────────────────────────────────────────────────────────────┤
│  Frontend (PWA)                                              │
│  ├── Dashboard (Real-time metrics)                           │
│  ├── Network Visualization (Cilium flows)                    │
│  ├── Security Reports (Trivy + AI)                           │
│  └── AI Chat (Cluster assistant)                             │
├─────────────────────────────────────────────────────────────┤
│  Backend (Rust + Axum + Tower)                               │
│  ├── Domain Services (Kubernetes, ArgoCD, Proxmox)           │
│  ├── Legacy Modules (Backward compatibility)                 │
│  ├── Telemetry (OpenObserve RUM)                             │
│  └── Cache Layer (In-memory)                                 │
├─────────────────────────────────────────────────────────────┤
│  Data Sources                                                │
│  ├── Kubernetes API                                          │
│  ├── ArgoCD API                                              │
│  ├── Trivy Server                                            │
│  ├── Cilium/Hubble                                           │
│  ├── Proxmox API                                             │
│  ├── Home Assistant API                                      │
│  ├── MQTT Broker                                             │
│  └── S3 Storage (AWS/MinIO)                                  │
└─────────────────────────────────────────────────────────────┘
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+
- Kubernetes cluster with kubectl configured
- Optional: ArgoCD, Trivy, Cilium, Proxmox

### Development

```bash
# Clone the repository
git clone https://github.com/JZacharie/Kusanagi.git
cd Kusanagi

# Run in development mode
cargo run

# Access the interface
open http://localhost:8080
```

### Production Deployment

```bash
# Build release binary
cargo build --release

# Deploy with systemd
./deploy.sh

# Or use Docker
docker build -t kusanagi:latest .
docker run -p 8080:8080 kusanagi:latest
```

### Kubernetes Deployment

```bash
# Using Helm
helm repo add kusanagi https://jzacharie.github.io/helmscharts
helm install kusanagi kusanagi/kusanagi

# Or using ArgoCD
kubectl apply -f deploy/argocd-app.yaml
```

## 📡 API Endpoints

### Core Endpoints

| Endpoint | Description | Status |
|----------|-------------|--------|
| `GET /` | Web interface | ✅ Live |
| `GET /api` | Service info | ✅ Live |
| `GET /health` | Health check | ✅ Live |
| `GET /metrics` | Prometheus metrics | ✅ Live |

### Kubernetes Monitoring

| Endpoint | Description | Data |
|----------|-------------|------|
| `GET /api/pods/status` | Pod status | 462 pods |
| `GET /api/nodes/status` | Node status | 16 nodes |
| `GET /api/services` | Services | 447 services |
| `GET /api/cluster/overview` | Cluster overview | Real-time |
| `GET /api/storage` | Storage volumes | 132 PV, 129 PVC |
| `GET /api/events` | Recent events | Last 20 |
| `GET /api/ingress` | Ingress controllers | Live |

### GitOps & Monitoring

| Endpoint | Description | Data |
|----------|-------------|------|
| `GET /api/argocd/status` | ArgoCD apps | 183 apps |
| `GET /api/alerts` | Active alerts | AlertManager |
| `GET /api/backups` | Backup status | Velero + CronJobs |
| `GET /api/quotas` | Resource quotas | Live |

### Security

| Endpoint | Description | Data |
|----------|-------------|------|
| `GET /api/security/vulnerabilities` | Vulnerability summary | Trivy |
| `GET /api/security/reports` | Security reports | S3 cached |
| `GET /api/security/reports/{id}` | Report details | AI enriched |

### Network Observability

| Endpoint | Description | Data |
|----------|-------------|------|
| `GET /api/cilium/flows` | Network flows | Hubble |
| `GET /api/cilium/namespaces` | Namespaces | Live |
| `GET /api/cilium/matrix` | Flow matrix | Real-time |

### Infrastructure

| Endpoint | Description | Data |
|----------|-------------|------|
| `GET /api/proxmox/vms` | Proxmox VMs | Live |
| `GET /api/proxmox/containers` | LXC containers | Live |
| `GET /api/proxmox/nodes` | Proxmox nodes | Live |
| `GET /api/ha/devices` | HA devices | Live |
| `GET /api/ha/sensors` | HA sensors | Live |

### AI & Chat

| Endpoint | Description | Data |
|----------|-------------|------|
| `POST /api/chat` | AI assistant | LLM powered |
| `GET /api/news` | Tech news | Aggregated |

### WebSocket

| Endpoint | Description | Data |
|----------|-------------|------|
| `WS /api/ws/notifications` | Real-time notifications | Live |

## 🔧 Configuration

### Environment Variables

```bash
# Server
BIND_ADDR=0.0.0.0:8080
RUST_LOG=info

# Kubernetes
KUBECONFIG=/path/to/kubeconfig
POD_NAMESPACE=kusanagi

# ArgoCD
ARGOCD_SERVER=argocd-server.argocd.svc
ARGOCD_TOKEN=your-token

# Trivy
TRIVY_SERVER_URL=http://trivy-server:8080

# Cilium/Hubble
HUBBLE_RELAY_URL=http://hubble-relay:80

# Proxmox
PROXMOX_HOST=proxmox.example.com
PROXMOX_USER=root@pam
PROXMOX_PASSWORD=your-password

# Home Assistant
HA_URL=http://homeassistant:8123
HA_TOKEN=your-token

# MQTT
MQTT_BROKER=mqtt.example.com:1883
MQTT_USERNAME=user
MQTT_PASSWORD=password

# LLM (Optional)
OLLAMA_URL=http://ollama:11434
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...

# S3 Storage
AWS_REGION=us-east-1
S3_BUCKET=kusanagi-data
S3_ENDPOINT=https://s3.amazonaws.com

# Telemetry (Optional)
OPENOBSERVE_URL=http://openobserve:5080
OPENOBSERVE_ORG=default
OPENOBSERVE_STREAM=kusanagi
OPENOBSERVE_TOKEN=your-token

# Slack (Optional)
SLACK_WEBHOOK_URL=https://hooks.slack.com/...
SLACK_BOT_TOKEN=xoxb-...
```

### Kubernetes RBAC

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: kusanagi
  namespace: kusanagi
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: kusanagi
rules:
  - apiGroups: [""]
    resources: ["pods", "nodes", "services", "events", "persistentvolumes", "persistentvolumeclaims"]
    verbs: ["get", "list", "watch"]
  - apiGroups: ["apps"]
    resources: ["deployments", "statefulsets", "daemonsets"]
    verbs: ["get", "list", "watch"]
  - apiGroups: ["batch"]
    resources: ["cronjobs", "jobs"]
    verbs: ["get", "list", "watch"]
  - apiGroups: ["networking.k8s.io"]
    resources: ["ingresses"]
    verbs: ["get", "list", "watch"]
```

## 🎨 Features in Detail

### 1. Real-time Monitoring

- **Pod Status**: Running, Pending, Failed, CrashLoopBackOff
- **Node Health**: CPU, Memory, Disk usage
- **Resource Quotas**: Namespace limits and usage
- **Events**: Real-time Kubernetes events

### 2. GitOps Integration

- **ArgoCD Apps**: Health and sync status
- **Auto-sync Detection**: Identify out-of-sync apps
- **Deployment History**: Track changes over time

### 3. Security Scanning

- **Trivy Integration**: Container vulnerability scanning
- **AI Enrichment**: LLM-powered vulnerability analysis
- **S3 Caching**: Fast report retrieval
- **Severity Breakdown**: Critical, High, Medium, Low

### 4. Network Observability

- **Flow Visualization**: D3.js network graphs
- **Bandwidth Metrics**: Real-time traffic analysis
- **Namespace Filtering**: Focus on specific workloads
- **Anomaly Detection**: Identify unusual patterns

### 5. AI Assistant

- **Cluster Analysis**: Ask questions about your cluster
- **Multi-LLM Support**: Ollama, OpenAI, Anthropic, LiteLLM
- **Context-Aware**: Uses real cluster data
- **Markdown Responses**: Formatted answers

### 6. Multi-Infrastructure

- **Proxmox**: VM and container management
- **Home Assistant**: IoT device monitoring
- **MQTT**: Message broker integration

## 📊 Performance

- **Memory Usage**: ~50MB baseline
- **Response Time**: <100ms for cached data
- **Concurrent Users**: 100+ supported
- **Cache Hit Rate**: >90% for frequent queries

## 🔒 Security

- **No Secrets in Code**: Environment-based configuration
- **RBAC Compliant**: Minimal Kubernetes permissions
- **TLS Support**: HTTPS ready
- **Input Validation**: All user inputs sanitized

## 🧪 Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration_test

# Run with coverage
cargo tarpaulin --out Html
```

## 📈 Monitoring Kusanagi

Kusanagi exposes Prometheus metrics at `/metrics`:

```
# HELP kusanagi_info Service information
# TYPE kusanagi_info gauge
kusanagi_info{version="0.3.0",architecture="hexagonal"} 1

# HELP kusanagi_requests_total Total HTTP requests
# TYPE kusanagi_requests_total counter
kusanagi_requests_total{endpoint="/api/pods/status",status="200"} 1234

# HELP kusanagi_cache_hits_total Cache hit count
# TYPE kusanagi_cache_hits_total counter
kusanagi_cache_hits_total 5678
```

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

## 📝 License

MIT License - See [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- Inspired by Ghost in the Shell (攻殻機動隊)
- Built with Rust and Axum
- UI design inspired by cyberpunk aesthetics
- Community contributions and feedback

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/JZacharie/Kusanagi/issues)
- **Discussions**: [GitHub Discussions](https://github.com/JZacharie/Kusanagi/discussions)
- **Documentation**: [Wiki](https://github.com/JZacharie/Kusanagi/wiki)

---

**Kusanagi v0.3.0** - Built with ❤️ by the community
