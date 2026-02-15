#!/bin/bash
set -e

# Kusanagi CI Build Script - Full Build Without Cache
# Usage: ./ci-build.sh [tag]

TAG="${1:-v0.3.0}"
REGISTRY="${2:-}"  # Optional: registry prefix like "ghcr.io/username/"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║           Kusanagi CI Build - Full Rebuild                    ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}🏷️  Tag: ${TAG}${NC}"
echo -e "${YELLOW}📦 Registry: ${REGISTRY:-local}${NC}"
echo ""

# Step 1: Clean previous builds
echo -e "${BLUE}[1/6] 🧹 Cleaning previous builds...${NC}"
cargo clean 2>/dev/null || true
rm -rf target/
rm -f static/js/k8s.js
echo -e "${GREEN}✅ Clean complete${NC}"

# Step 2: Verify structure
echo -e "${BLUE}[2/6] 🔍 Verifying modular structure...${NC}"
for file in state.js pods.js nodes.js services.js storage.js argocd.js main.js; do
    if [ ! -f "static/js/k8s/$file" ]; then
        echo -e "${RED}❌ Missing: static/js/k8s/$file${NC}"
        exit 1
    fi
done
if [ -f "static/js/k8s.js" ]; then
    echo -e "${RED}❌ Old k8s.js still exists!${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Structure OK${NC}"

# Step 3: Build Rust binary
echo -e "${BLUE}[3/6] 🔨 Building Rust binary...${NC}"
cargo build --release 2>&1 | tee build.log
echo -e "${GREEN}✅ Rust build complete${NC}"

# Step 4: Verify binary
echo -e "${BLUE}[4/6] 🔍 Verifying binary...${NC}"
if [ ! -f "target/release/kusanagi" ]; then
    echo -e "${RED}❌ Binary not found!${NC}"
    exit 1
fi
ls -lh target/release/kusanagi
echo -e "${GREEN}✅ Binary OK${NC}"

# Step 5: Build Docker image
echo -e "${BLUE}[5/6] 🐳 Building Docker image (NO CACHE)...${NC}"
FULL_TAG="${REGISTRY}kusanagi:${TAG}"
docker build \
    --no-cache \
    --target release-ci \
    --build-arg PREBUILT_BINARY=target/release/kusanagi \
    -t "${FULL_TAG}" \
    .
echo -e "${GREEN}✅ Docker build complete${NC}"

# Step 6: Verify image
echo -e "${BLUE}[6/6] 🔍 Verifying Docker image...${NC}"

# Check no old k8s.js
if docker run --rm "${FULL_TAG}" test -f /app/static/js/k8s.js 2>/dev/null; then
    echo -e "${RED}❌ FAIL: Old k8s.js found in image!${NC}"
    exit 1
fi

# Check new modules exist
docker run --rm "${FULL_TAG}" test -f /app/static/js/k8s/main.js 2>/dev/null || {
    echo -e "${RED}❌ FAIL: New modules not found in image!${NC}"
    exit 1
}

# Show image size
IMAGE_SIZE=$(docker images --format "{{.Size}}" "${FULL_TAG}")
echo -e "${GREEN}✅ Image verified${NC}"
echo -e "${BLUE}   Size: ${IMAGE_SIZE}${NC}"

echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              ✅ BUILD SUCCESSFUL                               ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}📤 To push:${NC}"
echo "   docker push ${FULL_TAG}"
echo ""
echo -e "${YELLOW}☸️  To deploy:${NC}"
echo "   helm upgrade --install kusanagi ./helmscharts/charts/kusanagi \\"
echo "     --set image.tag=${TAG} \\"
echo "     --set image.repository=${REGISTRY:-your-registry/}kusanagi"
