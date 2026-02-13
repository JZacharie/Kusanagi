#!/bin/bash
set -e

echo "🔍 Checking ArgoCD Access..."

# Check if kubectl is present
if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl not found!"
    exit 1
fi

echo "✅ kubectl found"

# Check permissions/access
echo "🔍 Attempting to list ArgoCD applications (JSON)..."
if kubectl get applications -n argocd -o json > argocd_debug.json; then
    echo "✅ Successfully fetched applications JSON"
    echo "   Saved to argocd_debug.json"
    
    # Check item count
    COUNT=$(grep -o '"items":' argocd_debug.json | wc -l)
    ITEM_COUNT=$(jq '.items | length' argocd_debug.json 2>/dev/null || echo "jq not installed")
    echo "   JSON Validity check: Items array found"
    echo "   Item count (jq): $ITEM_COUNT"
else
    echo "❌ Failed to fetch applications!"
    echo "   Error output:"
    kubectl get applications -n argocd -o json 2>&1
fi

echo ""
echo "🔍 Checking Pods..."
kubectl get pods -n argocd
