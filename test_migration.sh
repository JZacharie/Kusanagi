#!/bin/bash

echo "🧪 Kusanagi Migration Test Suite"
echo "================================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test results
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run test
run_test() {
    local test_name="$1"
    local command="$2"
    local expected_status="$3"
    
    echo -n "Testing $test_name... "
    
    if eval "$command" > /dev/null 2>&1; then
        if [ "$expected_status" = "success" ]; then
            echo -e "${GREEN}✅ PASS${NC}"
            ((TESTS_PASSED++))
        else
            echo -e "${RED}❌ FAIL (expected failure but got success)${NC}"
            ((TESTS_FAILED++))
        fi
    else
        if [ "$expected_status" = "fail" ]; then
            echo -e "${GREEN}✅ PASS (expected failure)${NC}"
            ((TESTS_PASSED++))
        else
            echo -e "${RED}❌ FAIL${NC}"
            ((TESTS_FAILED++))
        fi
    fi
}

# Function to test endpoint
test_endpoint() {
    local name="$1"
    local url="$2"
    local port="$3"
    
    echo -n "Testing $name... "
    
    response=$(curl -s -w "%{http_code}" "http://localhost:$port$url" -o /tmp/response.json)
    
    if [ "$response" = "200" ]; then
        echo -e "${GREEN}✅ PASS${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}❌ FAIL (HTTP $response)${NC}"
        ((TESTS_FAILED++))
    fi
}

echo ""
echo "📦 Phase 1: Docker Image Tests"
echo "------------------------------"

# Test image builds
run_test "Phase 3 Simple Build" "docker build -f Dockerfile.phase3_simple -t kusanagi:test-simple . -q" "success"
run_test "Phase 3 Test Build" "docker build -f Dockerfile.phase3_test -t kusanagi:test-real . -q" "success"

echo ""
echo "🚀 Phase 2: Container Startup Tests"
echo "-----------------------------------"

# Clean up any existing containers
docker rm -f kusanagi-test-simple kusanagi-test-real 2>/dev/null

# Start containers
echo "Starting Phase 3 Simple container..."
docker run -d -p 8085:8080 --name kusanagi-test-simple kusanagi:test-simple > /dev/null

echo "Starting Phase 3 Test container..."
docker run -d -p 8086:8080 --name kusanagi-test-real kusanagi:test-real > /dev/null

# Wait for startup
sleep 5

# Check if containers are running
run_test "Simple Container Running" "docker ps | grep kusanagi-test-simple" "success"
run_test "Test Container Running" "docker ps | grep kusanagi-test-real" "success"

echo ""
echo "🌐 Phase 3: API Endpoint Tests"
echo "------------------------------"

# Test Simple version endpoints
echo "Testing Simple Version (Mock Data):"
test_endpoint "Health Check" "/health" "8085"
test_endpoint "Service Info" "/" "8085"
test_endpoint "Cluster Overview" "/api/cluster" "8085"
test_endpoint "Nodes List" "/api/nodes" "8085"
test_endpoint "Pods List" "/api/pods" "8085"
test_endpoint "Pods Filtered" "/api/pods?namespace=kube-system" "8085"
test_endpoint "Events List" "/api/events" "8085"
test_endpoint "Metrics" "/api/metrics" "8085"
test_endpoint "Combined Overview" "/api/overview" "8085"

echo ""
echo "Testing Real Integration Version:"
test_endpoint "Health Check" "/health" "8086"
test_endpoint "Service Info" "/" "8086"
test_endpoint "K8s Connection Test" "/test/k8s" "8086"
test_endpoint "Prometheus Test" "/test/prometheus" "8086"
test_endpoint "Cluster Overview" "/api/cluster" "8086"
test_endpoint "Nodes List" "/api/nodes" "8086"
test_endpoint "Combined Overview" "/api/overview" "8086"

echo ""
echo "🔍 Phase 4: Data Validation Tests"
echo "---------------------------------"

# Test data structure
echo -n "Validating JSON responses... "
if curl -s http://localhost:8085/api/overview | jq '.timestamp' > /dev/null 2>&1; then
    echo -e "${GREEN}✅ PASS${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}❌ FAIL${NC}"
    ((TESTS_FAILED++))
fi

echo -n "Testing namespace filtering... "
pods_all=$(curl -s http://localhost:8085/api/pods | jq '. | length')
pods_filtered=$(curl -s http://localhost:8085/api/pods?namespace=kube-system | jq '. | length')

if [ "$pods_filtered" -lt "$pods_all" ]; then
    echo -e "${GREEN}✅ PASS${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}❌ FAIL${NC}"
    ((TESTS_FAILED++))
fi

echo ""
echo "🧹 Phase 5: Cleanup"
echo "-------------------"

# Stop and remove containers
docker stop kusanagi-test-simple kusanagi-test-real > /dev/null 2>&1
docker rm kusanagi-test-simple kusanagi-test-real > /dev/null 2>&1

echo "Containers cleaned up"

echo ""
echo "📊 Test Results Summary"
echo "======================="
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo -e "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 ALL TESTS PASSED! Migration successful!${NC}"
    exit 0
else
    echo -e "\n${RED}❌ Some tests failed. Check the output above.${NC}"
    exit 1
fi
