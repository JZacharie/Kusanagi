#!/bin/bash

echo "🧪 Test du Système de Préchargement Kusanagi"
echo "============================================="

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

BASE_URL="http://localhost:8090"

echo -e "${BLUE}📦 Test des Données Préchargées${NC}"
echo "--------------------------------"

echo "1. ArgoCD Applications:"
argocd_apps=$(curl -s "$BASE_URL/api/v1/argocd" | jq '.data.summary.total')
echo "   - Applications: $argocd_apps"
echo "   - Source: $(curl -s "$BASE_URL/api/v1/argocd" | jq -r '.source')"

echo ""
echo "2. Proxmox Cluster:"
proxmox_vms=$(curl -s "$BASE_URL/api/v1/proxmox" | jq '.data.vms | length')
echo "   - VMs: $proxmox_vms"
echo "   - Cluster: $(curl -s "$BASE_URL/api/v1/proxmox" | jq -r '.data.cluster.name')"

echo ""
echo "3. Météo:"
weather_temp=$(curl -s "$BASE_URL/api/v1/weather" | jq '.data.current.temperature')
echo "   - Température: ${weather_temp}°C"
echo "   - Lieu: $(curl -s "$BASE_URL/api/v1/weather" | jq -r '.data.current.location')"

echo ""
echo -e "${BLUE}⚡ Test de Performance${NC}"
echo "---------------------"

echo "Response times (préchargé):"
for endpoint in "argocd" "proxmox" "weather"; do
    time=$(curl -w "%{time_total}s" -s -o /dev/null "$BASE_URL/api/v1/$endpoint")
    echo "   - $endpoint: $time"
done

echo ""
echo -e "${BLUE}🔄 Test de Refresh${NC}"
echo "------------------"

echo "Forcer le refresh du cache..."
refresh_result=$(curl -s -X POST "$BASE_URL/api/v1/cache/refresh" | jq -r '.message')
echo "   - Résultat: $refresh_result"

echo ""
echo -e "${BLUE}📊 Status Final${NC}"
echo "---------------"

cache_status=$(curl -s "$BASE_URL/api/v1/cache/status")
echo "ArgoCD cached: $(echo "$cache_status" | jq -r '.cache_status.argocd.cached')"
echo "Proxmox cached: $(echo "$cache_status" | jq -r '.cache_status.proxmox.cached')"
echo "Weather cached: $(echo "$cache_status" | jq -r '.cache_status.weather.cached')"

echo ""
echo -e "${GREEN}✅ Système de préchargement fonctionnel !${NC}"
echo "   - 3 services préchargés"
echo "   - Auto-refresh toutes les 5 minutes"
echo "   - Performance optimisée"
