#!/bin/bash
set -e

# Couleurs
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🔧 Kusanagi Docker Build Script${NC}"

# Vérifier que l'ancien fichier k8s.js n'existe pas
if [ -f "static/js/k8s.js" ]; then
    echo -e "${RED}❌ Ancien fichier static/js/k8s.js trouvé ! Suppression...${NC}"
    rm -f static/js/k8s.js
fi

# Vérifier que les nouveaux modules existent
echo -e "${YELLOW}📦 Vérification des modules...${NC}"
for file in state.js pods.js nodes.js services.js storage.js argocd.js main.js; do
    if [ ! -f "static/js/k8s/$file" ]; then
        echo -e "${RED}❌ Module manquant: static/js/k8s/$file${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ $file${NC}"
done

# Build Rust
echo -e "${YELLOW}🔨 Build Rust...${NC}"
cargo build --release

# Build Docker sans cache
echo -e "${YELLOW}🐳 Build Docker (no cache)...${NC}"
docker build --no-cache -f Dockerfile --target release-ci \
    --build-arg PREBUILT_BINARY=target/release/kusanagi \
    -t kusanagi:v0.3.0 .

echo -e "${GREEN}✅ Build terminé !${NC}"
echo -e "${YELLOW}📋 Pour pousser:${NC}"
echo "docker tag kusanagi:v0.3.0 <registry>/kusanagi:v0.3.0"
echo "docker push <registry>/kusanagi:v0.3.0"
