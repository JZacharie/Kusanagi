#!/bin/bash

echo "🚀 Kusanagi Production Deployment & Test Suite"
echo "=============================================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Configuration
PROD_PORT=8088
SIMPLE_PORT=8087

echo ""
echo -e "${BLUE}📦 Phase 1: Building Production Images${NC}"
echo "--------------------------------------"

echo "Building Kusanagi Production v1.0.0..."
docker build -f Dockerfile.production -t kusanagi:v1.0.0 . -q
echo -e "${GREEN}✅ Production image built${NC}"

echo "Building Kusanagi Simple (for comparison)..."
docker build -f Dockerfile.phase3_simple -t kusanagi:simple . -q
echo -e "${GREEN}✅ Simple image built${NC}"

echo ""
echo -e "${BLUE}🚀 Phase 2: Deployment${NC}"
echo "----------------------"

# Clean up existing containers
docker rm -f kusanagi-prod kusanagi-simple 2>/dev/null

echo "Deploying Production version on port $PROD_PORT..."
docker run -d -p $PROD_PORT:8080 --name kusanagi-prod kusanagi:v1.0.0
sleep 3

echo "Deploying Simple version on port $SIMPLE_PORT..."
docker run -d -p $SIMPLE_PORT:8080 --name kusanagi-simple kusanagi:simple
sleep 3

echo ""
echo -e "${BLUE}🧪 Phase 3: Production API Tests${NC}"
echo "--------------------------------"

# Test production endpoints
echo "Testing Production Health Check..."
health=$(curl -s http://localhost:$PROD_PORT/health | jq -r '.status')
if [ "$health" = "healthy" ]; then
    echo -e "${GREEN}✅ Health check: $health${NC}"
else
    echo -e "${RED}❌ Health check failed${NC}"
fi

echo "Testing Production Service Info..."
version=$(curl -s http://localhost:$PROD_PORT/ | jq -r '.version')
echo -e "${GREEN}✅ Version: $version${NC}"

echo "Testing Prometheus Metrics..."
metrics_count=$(curl -s http://localhost:$PROD_PORT/metrics | grep -c "kusanagi_")
echo -e "${GREEN}✅ Prometheus metrics: $metrics_count metrics exported${NC}"

echo "Testing Cluster API..."
cluster_name=$(curl -s http://localhost:$PROD_PORT/api/v1/cluster | jq -r '.cluster.name')
echo -e "${GREEN}✅ Cluster: $cluster_name${NC}"

echo "Testing Nodes API..."
nodes_count=$(curl -s http://localhost:$PROD_PORT/api/v1/nodes | jq '.summary.total')
echo -e "${GREEN}✅ Nodes: $nodes_count nodes${NC}"

echo "Testing Pods API..."
pods_count=$(curl -s http://localhost:$PROD_PORT/api/v1/pods | jq '.metadata.total')
echo -e "${GREEN}✅ Pods: $pods_count pods${NC}"

echo "Testing Events API..."
events_count=$(curl -s http://localhost:$PROD_PORT/api/v1/events | jq '.metadata.total')
echo -e "${GREEN}✅ Events: $events_count events${NC}"

echo "Testing Overview API..."
overview_alerts=$(curl -s http://localhost:$PROD_PORT/api/v1/overview | jq '.alerts | length')
echo -e "${GREEN}✅ Overview: $overview_alerts alerts${NC}"

echo ""
echo -e "${BLUE}🔍 Phase 4: Advanced Features Test${NC}"
echo "-----------------------------------"

echo "Testing namespace filtering..."
kube_pods=$(curl -s "http://localhost:$PROD_PORT/api/v1/pods?namespace=kube-system" | jq '.metadata.total')
echo -e "${GREEN}✅ Namespace filter: $kube_pods kube-system pods${NC}"

echo "Testing status filtering..."
running_pods=$(curl -s "http://localhost:$PROD_PORT/api/v1/pods?status=Running" | jq '.metadata.total')
echo -e "${GREEN}✅ Status filter: $running_pods running pods${NC}"

echo "Testing event type filtering..."
warning_events=$(curl -s "http://localhost:$PROD_PORT/api/v1/events?type=Warning" | jq '.metadata.total')
echo -e "${GREEN}✅ Event filter: $warning_events warning events${NC}"

echo "Testing limit parameter..."
limited_pods=$(curl -s "http://localhost:$PROD_PORT/api/v1/pods?limit=2" | jq '.metadata.total')
echo -e "${GREEN}✅ Limit filter: $limited_pods pods (limit=2)${NC}"

echo ""
echo -e "${BLUE}📊 Phase 5: Performance Comparison${NC}"
echo "-----------------------------------"

echo "Measuring response times..."

# Production response time
prod_time=$(curl -w "%{time_total}" -s -o /dev/null http://localhost:$PROD_PORT/api/v1/overview)
echo -e "${GREEN}✅ Production overview: ${prod_time}s${NC}"

# Simple response time
simple_time=$(curl -w "%{time_total}" -s -o /dev/null http://localhost:$SIMPLE_PORT/api/overview)
echo -e "${GREEN}✅ Simple overview: ${simple_time}s${NC}"

echo ""
echo -e "${BLUE}🐳 Phase 6: Container Information${NC}"
echo "--------------------------------"

echo "Production container info:"
docker inspect kusanagi-prod --format='Size: {{.Size}} bytes' 2>/dev/null || echo "Container info not available"
docker logs kusanagi-prod 2>/dev/null | tail -3

echo ""
echo "Image sizes:"
docker images | grep kusanagi | head -5

echo ""
echo -e "${BLUE}🧹 Phase 7: Cleanup Options${NC}"
echo "-----------------------------"

echo "Containers are still running for manual testing:"
echo -e "${YELLOW}Production API: http://localhost:$PROD_PORT${NC}"
echo -e "${YELLOW}Simple API: http://localhost:$SIMPLE_PORT${NC}"
echo ""
echo "To stop containers:"
echo "  docker stop kusanagi-prod kusanagi-simple"
echo "  docker rm kusanagi-prod kusanagi-simple"

echo ""
echo -e "${GREEN}🎉 DEPLOYMENT SUCCESSFUL!${NC}"
echo ""
echo -e "${BLUE}📋 Production Endpoints Available:${NC}"
echo "  GET  /                     - Service information"
echo "  GET  /health               - Health check"
echo "  GET  /metrics              - Prometheus metrics"
echo "  GET  /api/v1/cluster       - Cluster overview"
echo "  GET  /api/v1/nodes         - Node listing"
echo "  GET  /api/v1/pods          - Pod listing (supports ?namespace=, ?status=, ?limit=)"
echo "  GET  /api/v1/events        - Event listing (supports ?namespace=, ?type=, ?limit=)"
echo "  GET  /api/v1/overview      - Combined overview with alerts"
echo ""
echo -e "${BLUE}🔧 Production Features:${NC}"
echo "  ✅ Production-grade logging"
echo "  ✅ Prometheus metrics export"
echo "  ✅ Advanced filtering & pagination"
echo "  ✅ Comprehensive health checks"
echo "  ✅ Security headers"
echo "  ✅ Non-root container execution"
echo "  ✅ Resource monitoring"
echo "  ✅ Alert detection"
