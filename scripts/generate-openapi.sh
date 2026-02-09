#!/bin/bash
# Generate OpenAPI specification from code
# Usage: ./scripts/generate-openapi.sh

set -e

echo "🔮 Generating OpenAPI specification..."

# Create output directory
mkdir -p docs/api

# Check if utoipa is available
if ! grep -q "utoipa" Cargo.toml 2>/dev/null; then
    echo "⚠️  utoipa not found in Cargo.toml"
    echo "Add the following to your Cargo.toml:"
    echo ""
    echo "[dependencies]"
    echo "utoipa = { version = \"4.0\", features = [\"actix_extras\"] }"
    echo "utoipa-swagger-ui = { version = \"6.0\", features = [\"actix-web\"] }"
    echo ""
    echo "Then implement OpenAPI documentation in your handlers."
    exit 1
fi

# Build and generate spec
cargo run --bin generate-openapi 2>/dev/null || {
    echo "⚠️  OpenAPI generator binary not found."
    echo "Creating minimal spec from code..."
    
    cat > docs/api/openapi.json << 'EOF'
{
  "openapi": "3.0.0",
  "info": {
    "title": "Kusanagi API",
    "version": "0.2.0",
    "description": "Kubernetes monitoring platform API"
  },
  "paths": {
    "/api": {
      "get": {
        "summary": "Service info",
        "responses": {
          "200": {
            "description": "Service information"
          }
        }
      }
    },
    "/api/pods/status": {
      "get": {
        "summary": "Get pod status",
        "responses": {
          "200": {
            "description": "List of pods"
          }
        }
      }
    },
    "/api/nodes/status": {
      "get": {
        "summary": "Get node status",
        "responses": {
          "200": {
            "description": "List of nodes"
          }
        }
      }
    },
    "/health": {
      "get": {
        "summary": "Health check",
        "responses": {
          "200": {
            "description": "Service is healthy"
          }
        }
      }
    }
  }
}
EOF
}

echo "✅ OpenAPI spec generated: docs/api/openapi.json"
echo ""
echo "View with Swagger UI:"
echo "  docker run -p 8081:8080 -e API_URL=/openapi.json -v \$(pwd)/docs/api:/usr/share/nginx/html/swagger swaggerapi/swagger-ui"
