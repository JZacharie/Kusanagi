#!/bin/bash

echo "🧪 Tests de Validation Finale Kusanagi"
echo "======================================"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

PASSED=0
FAILED=0

test_endpoint() {
    local name="$1"
    local url="$2"
    
    echo -n "Testing $name... "
    
    if [ "$name" = "Prometheus Metrics" ]; then
        response=$(curl -s "$url")
        if echo "$response" | grep -q "kusanagi_"; then
            echo -e "${GREEN}✅ PASS${NC}"
            ((PASSED++))
        else
            echo -e "${RED}❌ FAIL${NC}"
            ((FAILED++))
        fi
    else
        response=$(curl -s "$url")
        status=$?
        
        if [ $status -eq 0 ] && echo "$response" | jq . > /dev/null 2>&1; then
            echo -e "${GREEN}✅ PASS${NC}"
            ((PASSED++))
        else
            echo -e "${RED}❌ FAIL${NC}"
            ((FAILED++))
        fi
    fi
}

echo -e "${BLUE}📊 Tests API Production${NC}"
echo "----------------------"

BASE_URL="http://localhost:8088"

test_endpoint "Service Info" "$BASE_URL/"
test_endpoint "Health Check" "$BASE_URL/health"
test_endpoint "Prometheus Metrics" "$BASE_URL/metrics"
test_endpoint "Cluster API" "$BASE_URL/api/v1/cluster"
test_endpoint "Nodes API" "$BASE_URL/api/v1/nodes"
test_endpoint "Pods API" "$BASE_URL/api/v1/pods"
test_endpoint "Events API" "$BASE_URL/api/v1/events"
test_endpoint "Overview API" "$BASE_URL/api/v1/overview"

echo ""
echo -e "${BLUE}🔍 Tests Filtrage${NC}"
echo "-----------------"

test_endpoint "Pods Namespace Filter" "$BASE_URL/api/v1/pods?namespace=kube-system"
test_endpoint "Pods Status Filter" "$BASE_URL/api/v1/pods?status=Running"
test_endpoint "Events Type Filter" "$BASE_URL/api/v1/events?type=Warning"
test_endpoint "Pods Limit" "$BASE_URL/api/v1/pods?limit=2"
test_endpoint "Events Limit" "$BASE_URL/api/v1/events?limit=1"

echo ""
echo -e "${BLUE}⚡ Tests Performance${NC}"
echo "-------------------"

echo -n "Response time < 10ms... "
time=$(curl -w "%{time_total}" -s -o /dev/null "$BASE_URL/api/v1/overview")
if (( $(echo "$time < 0.01" | bc -l) )); then
    echo -e "${GREEN}✅ PASS ($time s)${NC}"
    ((PASSED++))
else
    echo -e "${RED}❌ FAIL ($time s)${NC}"
    ((FAILED++))
fi

echo ""
echo -e "${BLUE}📈 Tests Métriques${NC}"
echo "------------------"

echo -n "Prometheus metrics count... "
metrics_count=$(curl -s "$BASE_URL/metrics" | grep -c "kusanagi_")
if [ "$metrics_count" -ge 20 ]; then
    echo -e "${GREEN}✅ PASS ($metrics_count metrics)${NC}"
    ((PASSED++))
else
    echo -e "${RED}❌ FAIL ($metrics_count metrics)${NC}"
    ((FAILED++))
fi

echo ""
echo -e "${BLUE}🔒 Tests Sécurité${NC}"
echo "----------------"

echo -n "Security headers... "
headers=$(curl -I -s "$BASE_URL/" | grep -i "x-version")
if [ -n "$headers" ]; then
    echo -e "${GREEN}✅ PASS${NC}"
    ((PASSED++))
else
    echo -e "${RED}❌ FAIL${NC}"
    ((FAILED++))
fi

echo ""
echo "📊 Résultats Finaux"
echo "==================="
echo -e "Tests réussis: ${GREEN}$PASSED${NC}"
echo -e "Tests échoués: ${RED}$FAILED${NC}"
echo -e "Total: $((PASSED + FAILED))"

if [ $FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 TOUS LES TESTS RÉUSSIS !${NC}"
    echo -e "${GREEN}✅ Kusanagi Production v1.0.0 est VALIDÉ${NC}"
    exit 0
else
    echo -e "\n${RED}❌ Certains tests ont échoué${NC}"
    exit 1
fi
