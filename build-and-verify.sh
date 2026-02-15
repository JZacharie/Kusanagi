#!/bin/bash
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     Kusanagi Docker Build & Verify Script v2           ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

# Vérification pré-build
echo -e "${YELLOW}🔍 Vérification des fichiers...${NC}"

if [ -f "static/js/k8s.js" ]; then
    echo -e "${RED}❌ ERREUR: static/js/k8s.js existe encore!${NC}"
    echo -e "${YELLOW}   Supprimez-le avant de build:${NC}"
    echo "   rm static/js/k8s.js"
    exit 1
fi

if [ ! -d "static/js/k8s" ]; then
    echo -e "${RED}❌ ERREUR: static/js/k8s/ n'existe pas!${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Structure modulaire vérifiée${NC}"
echo -e "${BLUE}   Modules trouvés:${NC}"
ls -1 static/js/k8s/*.js | xargs -n1 basename

echo ""
echo -e "${YELLOW}🔨 Build Rust...${NC}"
cargo build --release

echo ""
echo -e "${YELLOW}🐳 Build Docker (NO CACHE)...${NC}"
echo -e "${BLUE}   Cela peut prendre plusieurs minutes...${NC}"

# Generate cache bust timestamp
CACHE_BUST=$(date +%s)

# Build avec vérification explicite
docker build \
    --no-cache \
    --build-arg STATIC_VERSION=v2 \
    --build-arg CACHE_BUST=${CACHE_BUST} \
    -f Dockerfile \
    --target release-ci \
    --build-arg PREBUILT_BINARY=target/release/kusanagi \
    -t kusanagi:v0.3.0 \
    .

echo ""
echo -e "${YELLOW}🔍 Vérification de l'image...${NC}"

# Vérifier que l'image ne contient pas l'ancien fichier
if docker run --rm kusanagi:v0.3.0 ls /app/static/js/k8s.js 2>/dev/null; then
    echo -e "${RED}❌ ÉCHEC: k8s.js trouvé dans l'image!${NC}"
    exit 1
else
    echo -e "${GREEN}✅ Ancien fichier k8s.js absent${NC}"
fi

# Vérifier que les nouveaux modules sont présents
if docker run --rm kusanagi:v0.3.0 test -f /app/static/js/k8s/main.js; then
    echo -e "${GREEN}✅ Nouveaux modules présents${NC}"
    echo -e "${BLUE}   Contenu de /app/static/js/k8s/:${NC}"
    docker run --rm kusanagi:v0.3.0 ls -la /app/static/js/k8s/
else
    echo -e "${RED}❌ ÉCHEC: Modules manquants!${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║           ✅ BUILD RÉUSSI!                             ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}📤 Pour pousser l'image:${NC}"
echo "   docker tag kusanagi:v0.3.0 <registry>/kusanagi:v0.3.0"
echo "   docker push <registry>/kusanagi:v0.3.0"
echo ""
echo -e "${YELLOW}☸️  Pour déployer sur Kubernetes:${NC}"
echo "   helm upgrade --install kusanagi ./helmscharts/charts/kusanagi \\"
echo "     --set image.tag=v0.3.0 \\"
echo "     --set image.pullPolicy=Always"
