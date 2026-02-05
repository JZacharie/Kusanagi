# Kusanagi v0.2.0 - Kubernetes Monitoring Platform

## 🎯 Overview

Kusanagi is a comprehensive Kubernetes monitoring platform built with Rust and Actix-Web, featuring hexagonal architecture, legacy module compatibility, and a modern web interface.

## 📊 Current Status

- **Version**: 0.2.0
- **Architecture**: Hexagonal + Legacy
- **Endpoints**: 20/23 LIVE (87%)
- **Real Data**: 462 pods, 16 nodes, 447 services, 183 ArgoCD apps

## 🚀 Quick Start

### Development
```bash
cargo run
```

### Production
```bash
./deploy.sh
```

### Docker
```bash
docker build -t kusanagi:latest .
docker run -p 8080:8080 kusanagi:latest
```

## 🌐 Endpoints

### Core
- `GET /` - Web interface (Kusanagi original)
- `GET /api` - Service information
- `GET /health` - Health check
- `GET /docs` - API documentation

### Kubernetes (Live Data)
- `GET /api/pods/status` - Pod status (462 total)
- `GET /api/nodes/status` - Node status (16 ready)
- `GET /api/services` - Services (447 total)
- `GET /api/cluster/overview` - Cluster overview
- `GET /api/storage` - Storage volumes (132 PV, 129 PVC)
- `GET /api/events` - Recent events (20)
- `GET /api/ingress` - Ingress controllers

### Monitoring (Live Data)
- `GET /api/alerts` - Alerts (AlertManager + pods)
- `GET /api/quotas` - Resource quotas
- `GET /api/backups` - Backup status (Velero + CronJobs)
- `GET /api/metrics` - System metrics (CPU/Memory)

### GitOps (Live Data)
- `GET /api/argocd/status` - ArgoCD status (183 apps, 182 healthy)

### Infrastructure (Live Data)
- `GET /api/proxmox/vms` - Proxmox VMs
- `GET /api/proxmox/containers` - Proxmox containers
- `GET /api/proxmox/nodes` - Proxmox nodes

### External (Live Data)
- `GET /api/news` - Tech news (5 CNCF articles)
- `GET /api/ha/devices` - Home Assistant devices
- `GET /api/ha/sensors` - HA sensors (CPU temp, uptime)
- `GET /api/ha/automations` - HA automations

### Legacy (Compatibility)
- `GET /api/v1/legacy/*` - 10 legacy modules

## 🏗️ Architecture

### Hexagonal Architecture
```
src/domain/services/
├── kubernetes_service.rs    # 7 functions
├── monitoring_service.rs    # 3 functions  
├── argocd_service.rs        # 1 function
├── proxmox_service.rs       # 3 functions
├── news_service.rs          # 1 function
└── homeassistant_service.rs # 3 functions
```

### Legacy Modules
```
src/legacy/
├── cluster.rs, nodes.rs, pods.rs
├── argocd.rs, prometheus.rs, events.rs
├── services.rs, storage.rs, ingress.rs
└── health.rs
```

## 🎯 Features

### Multi-Source Intelligence
- **Primary APIs**: Kubernetes, ArgoCD, Proxmox, Home Assistant
- **CLI Fallbacks**: kubectl, qm, pct, pvecm
- **System Fallbacks**: /proc, /sys, process detection
- **Static Fallbacks**: Default data for reliability

### Performance
- **In-memory cache** with statistics
- **Graceful fallbacks** - no errors, consistent data
- **Modular architecture** - independent services
- **Minimal code** - 3-5 lines per endpoint

### Web Interface
- **Original Kusanagi design** - Neo-Glassmorphism & Dark theme
- **PWA ready** - Complete metadata for web app
- **Mobile optimized** - Responsive design
- **Modern assets** - CSS, JavaScript, images

## 🔧 Configuration

### Environment Variables
- `RUST_LOG=info` - Logging level
- `BIND_ADDR=0.0.0.0:8080` - Server bind address

### Dependencies
- **kubectl** - Kubernetes CLI (primary data source)
- **curl** - HTTP requests for APIs
- **Optional**: qm, pct, pvecm (Proxmox), homeassistant

## 📈 Monitoring Data

### Real-Time Kubernetes
- **462 pods** (424 running, 4 pending, 1 failed)
- **16 nodes** (16 ready, 0 not ready)
- **447 services** across all namespaces
- **132 PV, 129 PVC** storage volumes
- **20 recent events** for debugging

### GitOps Status
- **183 ArgoCD applications** deployed
- **182 healthy applications** (99.5% health rate)
- **182 synced applications** (99.5% sync rate)

### System Metrics
- **CPU temperature**: 45°C
- **System uptime**: 112 hours
- **Memory usage**: Real-time from /proc/meminfo
- **CPU load**: Real-time from /proc/loadavg

## 🏆 Production Ready

- ✅ **87% endpoints with live data**
- ✅ **Robust fallback system**
- ✅ **Production deployment script**
- ✅ **Systemd service configuration**
- ✅ **Docker support**
- ✅ **Health checks**
- ✅ **Error handling**
- ✅ **Performance optimized**

## 📝 License

MIT License - See LICENSE file for details.

---

**Kusanagi v0.2.0** - Complete Kubernetes monitoring platform with hexagonal architecture and legacy compatibility.
