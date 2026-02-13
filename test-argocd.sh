#!/bin/bash
set -e

echo "🧪 Testing ArgoCD data collection..."
echo ""

# Test kubectl access
echo "1️⃣ Testing kubectl access to ArgoCD namespace..."
if kubectl get ns argocd &>/dev/null; then
    echo "   ✅ ArgoCD namespace exists"
else
    echo "   ❌ ArgoCD namespace not found"
    exit 1
fi

# Test ArgoCD applications
echo ""
echo "2️⃣ Testing ArgoCD applications..."
APP_COUNT=$(kubectl get applications -n argocd --no-headers 2>/dev/null | wc -l)
echo "   Found $APP_COUNT applications"

if [ "$APP_COUNT" -gt 0 ]; then
    echo ""
    echo "   Applications (JSON Summary):"
    kubectl get applications -n argocd -o json | jq -r '.items[] | "Name: \(.metadata.name) | Health: \(.status.health.status) | Sync: \(.status.sync.status)"'
fi

# Test API endpoint
echo ""
echo "3️⃣ Testing Kusanagi API endpoint..."
if pgrep -f "target/release/kusanagi" > /dev/null; then
    echo "   ✅ Kusanagi is running"
    
    echo ""
    echo "   Fetching /api/argocd/status..."
    curl -s http://localhost:8080/api/argocd/status | jq '.'
else
    echo "   ⚠️  Kusanagi not running. Start it with: cd Kusanagi && ./target/release/kusanagi"
fi

echo ""
echo "✅ Test complete"
