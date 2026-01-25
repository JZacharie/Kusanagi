# Kusanagi v0.9.0 - Deployment Guide

## Prerequisites

- Kubernetes cluster with ArgoCD installed
- Access to Proxmox VE API
- Access to Home Assistant instance
- kubectl configured for your cluster

## Step 1: Create Configuration Secret

First, configure your Proxmox and Home Assistant credentials:

```bash
# Set your values
export PROXMOX_URL="https://your-proxmox.example.com:8006"
export PROXMOX_USER="root@pam"
export PROXMOX_TOKEN_ID="kusanagi"
export PROXMOX_TOKEN_SECRET="your-secret-here"
export HOME_ASSISTANT_URL="http://your-homeassistant.local:8123"
export HOME_ASSISTANT_TOKEN="your-ha-token-here"

# Create the secret
kubectl create secret generic kusanagi-config \
  --from-literal=proxmox-url="$PROXMOX_URL" \
  --from-literal=proxmox-user="$PROXMOX_USER" \
  --from-literal=proxmox-token-id="$PROXMOX_TOKEN_ID" \
  --from-literal=proxmox-token-secret="$PROXMOX_TOKEN_SECRET" \
  --from-literal=ha-url="$HOME_ASSISTANT_URL" \
  --from-literal=ha-token="$HOME_ASSISTANT_TOKEN" \
  --from-literal=prometheus-url="http://kube-prometheus-stack-prometheus.kube-prometheus-stack.svc:9090" \
  -n kusanagi \
  --dry-run=client -o yaml | kubectl apply -f -
```

## Step 2: Update Deployment to Use Secrets

The deployment needs to be updated to inject these environment variables. This can be done via Helm chart or by patching the deployment:

### Option A: Patch Existing Deployment

```bash
kubectl patch deployment kusanagi -n kusanagi --type='json' -p='[
  {
    "op": "add",
    "path": "/spec/template/spec/containers/0/env",
    "value": [
      {
        "name": "PROXMOX_URL",
        "valueFrom": {
          "secretKeyRef": {
            "name": "kusanagi-config",
            "key": "proxmox-url"
          }
        }
      },
      {
        "name": "PROXMOX_USER",
        "valueFrom": {
          "secretKeyRef": {
            "name": "kusanagi-config",
            "key": "proxmox-user"
          }
        }
      },
      {
        "name": "PROXMOX_TOKEN_ID",
        "valueFrom": {
          "secretKeyRef": {
            "name": "kusanagi-config",
            "key": "proxmox-token-id"
          }
        }
      },
      {
        "name": "PROXMOX_TOKEN_SECRET",
        "valueFrom": {
          "secretKeyRef": {
            "name": "kusanagi-config",
            "key": "proxmox-token-secret"
          }
        }
      },
      {
        "name": "HOME_ASSISTANT_URL",
        "valueFrom": {
          "secretKeyRef": {
            "name": "kusanagi-config",
            "key": "ha-url"
          }
        }
      },
      {
        "name": "HOME_ASSISTANT_TOKEN",
        "valueFrom": {
          "secretKeyRef": {
            "name": "kusanagi-config",
            "key": "ha-token"
          }
        }
      },
      {
        "name": "PROMETHEUS_URL",
        "valueFrom": {
          "secretKeyRef": {
            "name": "kusanagi-config",
            "key": "prometheus-url"
          }
        }
      }
    ]
  }
]'
```

### Option B: Update Helm Chart (Recommended)

Update your Helm values file to include the secret reference.

## Step 3: Sync ArgoCD Application

Wait for GitHub Actions to complete building the new Docker image, then sync:

```bash
# Check if image is available
docker pull ghcr.io/jzacharie/kusanagi:latest

# Sync ArgoCD application
kubectl patch application kusanagi -n argocd --type merge -p '{"operation":{"initiatedBy":{"username":"admin"},"sync":{"revision":"main"}}}'

# Or use ArgoCD CLI
argocd app sync kusanagi

# Watch the rollout
kubectl rollout status deployment/kusanagi -n kusanagi
```

## Step 4: Verify Deployment

```bash
# Check pod status
kubectl get pods -n kusanagi

# Check pod logs
kubectl logs -n kusanagi deployment/kusanagi --tail=50

# Check if new endpoints are available
kubectl exec -n kusanagi deployment/kusanagi -- wget -qO- http://localhost:8080/api/proxmox/vms
kubectl exec -n kusanagi deployment/kusanagi -- wget -qO- http://localhost:8080/api/ha/sensors
```

## Step 5: Test UI

1. Open Kusanagi dashboard: `https://kusanagi.p.zacharie.org`
2. Click on "Proxmox" tab
3. Verify VMs and containers are displayed
4. Click on "Home Assistant" tab
5. Verify sensors and automations are displayed

## Step 6: Monitor

```bash
# Watch logs for errors
kubectl logs -n kusanagi deployment/kusanagi -f

# Check OpenObserve for telemetry
# Visit: https://o2-openobserve.p.zacharie.org
```

## Troubleshooting

### Proxmox Connection Issues

```bash
# Test Proxmox connectivity from pod
kubectl exec -n kusanagi deployment/kusanagi -- sh -c '
  apk add curl
  curl -k -H "Authorization: PVEAPIToken=root@pam!kusanagi=YOUR_TOKEN" \
    https://your-proxmox:8006/api2/json/nodes
'
```

### Home Assistant Connection Issues

```bash
# Test HA connectivity from pod
kubectl exec -n kusanagi deployment/kusanagi -- sh -c '
  apk add curl
  curl -H "Authorization: Bearer YOUR_HA_TOKEN" \
    http://your-homeassistant:8123/api/states
'
```

### Check Environment Variables

```bash
# Verify env vars are set in pod
kubectl exec -n kusanagi deployment/kusanagi -- env | grep -E '(PROXMOX|HOME_ASSISTANT)'
```

## Rollback

If issues occur, rollback to previous version:

```bash
# Rollback deployment
kubectl rollout undo deployment/kusanagi -n kusanagi

# Or sync to previous commit
argocd app sync kusanagi --revision 85f8c75
```

## Next Steps

1. Configure monitoring alerts in OpenObserve
2. Set up automated health checks
3. Document any custom configurations
4. Train team on new features
