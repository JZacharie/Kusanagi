#!/bin/bash
# Build RageCheck image locally and push to registry
# Usage: ./build-ragecheck.sh [registry-host]

set -e

REGISTRY="${1:-localhost:5000}"
IMAGE_NAME="${REGISTRY}/ragecheck:latest"

echo "🏗️ Building RageCheck image..."
echo "📤 Target: $IMAGE_NAME"

# Create temp build directory
BUILD_DIR=$(mktemp -d)
trap "rm -rf $BUILD_DIR" EXIT

# Clone repo
echo "📥 Cloning repository..."
git clone --depth 1 https://github.com/aagoldberg/ragecheck.git "$BUILD_DIR"

# Build Docker image
echo "🔨 Building Docker image..."
docker build -t "$IMAGE_NAME" -f - "$BUILD_DIR" << 'DOCKERFILE'
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci --prefer-offline --no-audit
COPY . .
RUN npm run build

FROM node:20-alpine
WORKDIR /app
ENV NODE_ENV=production
ENV PORT=3000
COPY --from=builder /app/package*.json ./
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/public ./public
COPY --from=builder /app/node_modules ./node_modules
EXPOSE 3000
USER node
CMD ["npm", "start"]
DOCKERFILE

# Push to registry
echo "📤 Pushing to registry..."
docker push "$IMAGE_NAME"

echo "✅ Build complete!"
echo ""
echo "Update the deployment to use:"
echo "  image: $IMAGE_NAME"
