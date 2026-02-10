# Kusanagi - Project Overview

## Identity
**Name**: Kusanagi v0.3.0  
**Type**: Kubernetes Monitoring Platform  
**Language**: Rust (Axum + Tower)  
**Architecture**: Hexagonal (Clean) + Legacy Modules  

## Core Purpose
Real-time K8s monitoring with GitOps integration, security scanning, and multi-infrastructure support.

## Key Metrics
- 462 pods monitored
- 16 nodes
- 447 services
- 183 ArgoCD apps

## Directory Structure
```
src/
├── application/use_cases/    # Business logic
├── domain/
│   ├── entities/            # Domain models
│   ├── ports/               # Interfaces (traits)
│   └── services/            # Domain services
├── infrastructure/
│   └── repositories/        # Implementations
├── interfaces/http/         # HTTP handlers
├── handlers/                # Legacy handlers
└── legacy/                  # Deprecated modules

static/js/                   # Frontend (PWA)
├── k8s.js                   # K8s management
├── dashboard.js             # Core dashboard
├── monitors.js              # Monitoring
└── weather.js               # Weather widget
```

## Tech Stack
- **Backend**: Rust + Actix-Web + Tokio
- **Frontend**: Vanilla JS (PWA)
- **Cache**: In-memory (AdvancedCache)
- **External**: kube-rs, reqwest, rustls
