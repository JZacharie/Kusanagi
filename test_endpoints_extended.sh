#!/bin/bash

echo "🧪 Testing Kusanagi Phase 3 Extended Endpoints"

BASE_URL="http://localhost:8083"

echo ""
echo "📋 Testing all endpoints..."

# Test service info
echo "1. Testing GET /"
curl -s "$BASE_URL/" | jq '.' || echo "❌ Failed"

# Test health check
echo ""
echo "2. Testing GET /health"
curl -s "$BASE_URL/health" | jq '.' || echo "❌ Failed"

# Test cluster overview
echo ""
echo "3. Testing GET /api/cluster"
curl -s "$BASE_URL/api/cluster" | jq '.' || echo "❌ Failed"

# Test nodes
echo ""
echo "4. Testing GET /api/nodes"
curl -s "$BASE_URL/api/nodes" | jq '.' || echo "❌ Failed"

# Test pods
echo ""
echo "5. Testing GET /api/pods"
curl -s "$BASE_URL/api/pods" | jq '.' || echo "❌ Failed"

# Test pods with namespace filter
echo ""
echo "6. Testing GET /api/pods?namespace=kube-system"
curl -s "$BASE_URL/api/pods?namespace=kube-system" | jq '.' || echo "❌ Failed"

# Test events
echo ""
echo "7. Testing GET /api/events"
curl -s "$BASE_URL/api/events" | jq '.' || echo "❌ Failed"

# Test metrics
echo ""
echo "8. Testing GET /api/metrics"
curl -s "$BASE_URL/api/metrics" | jq '.' || echo "❌ Failed"

# Test combined overview
echo ""
echo "9. Testing GET /api/overview (Combined K8s + Prometheus)"
curl -s "$BASE_URL/api/overview" | jq '.' || echo "❌ Failed"

echo ""
echo "✅ All endpoint tests completed!"
echo ""
echo "🔍 To run this test:"
echo "   docker run -d -p 8080:8080 --name kusanagi-test kusanagi:extended"
echo "   ./test_endpoints_extended.sh"
echo "   docker stop kusanagi-test && docker rm kusanagi-test"
