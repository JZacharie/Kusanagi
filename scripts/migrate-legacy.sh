#!/bin/bash
# Legacy to Hexagonal Migration Helper
# Usage: ./scripts/migrate-legacy.sh <module_name>

set -e

MODULE_NAME=$1

if [ -z "$MODULE_NAME" ]; then
    echo "Usage: $0 <module_name>"
    echo ""
    echo "Example:"
    echo "  $0 security"
    echo "  $0 alertmanager"
    echo ""
    echo "Available legacy modules:"
    ls -1 src/legacy/*.rs 2>/dev/null | xargs -n1 basename -s .rs | grep -v mod || echo "  (none found)"
    exit 1
fi

LEGACY_FILE="src/legacy/${MODULE_NAME}.rs"

if [ ! -f "$LEGACY_FILE" ]; then
    echo "❌ Legacy file not found: $LEGACY_FILE"
    exit 1
fi

echo "🔮 Kusanagi Legacy Migration Tool"
echo "================================="
echo ""
echo "Module: $MODULE_NAME"
echo "Source: $LEGACY_FILE"
echo ""

# Count lines in legacy file
LINES=$(wc -l < "$LEGACY_FILE")
echo "📊 Statistics:"
echo "  Lines of code: $LINES"
echo ""

# Create directory structure
echo "📁 Creating directory structure..."

mkdir -p src/domain/entities
touch src/domain/entities/mod.rs

mkdir -p src/domain/ports
touch src/domain/ports/mod.rs

mkdir -p src/application/use_cases
mkdir -p src/application/dtos
mkdir -p src/application/mappers

mkdir -p src/infrastructure/repositories
touch src/infrastructure/repositories/mod.rs

mkdir -p src/interfaces/http
touch src/interfaces/http/mod.rs

echo "✅ Directory structure created"
echo ""

# Generate template files
echo "📝 Generating template files..."

# Domain Entity
ENTITY_FILE="src/domain/entities/${MODULE_NAME}.rs"
if [ ! -f "$ENTITY_FILE" ]; then
    cat > "$ENTITY_FILE" << EOF
//! ${MODULE_NAME} domain entity
//! 
//! This entity represents the core business object for ${MODULE_NAME}.
//! It should be independent of any infrastructure concerns.

use serde::{Deserialize, Serialize};

/// ${MODULE_NAME} entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ${MODULE_NAME^} {
    // TODO: Define entity fields based on legacy code
    pub id: String,
    pub name: String,
}

impl ${MODULE_NAME^} {
    pub fn new(id: String, name: String) -> Self {
        Self { id, name }
    }
}
EOF
    echo "  ✅ Created: $ENTITY_FILE"
else
    echo "  ⚠️  Exists: $ENTITY_FILE"
fi

# Domain Port (Repository Interface)
PORT_FILE="src/domain/ports/${MODULE_NAME}_repository.rs"
if [ ! -f "$PORT_FILE" ]; then
    cat > "$PORT_FILE" << EOF
//! ${MODULE_NAME} repository port
//!
//! This port defines the interface that the domain layer expects
//! from any ${MODULE_NAME} repository implementation.

use async_trait::async_trait;
use crate::domain::entities::${MODULE_NAME}::${MODULE_NAME^};
use anyhow::Result;

/// Port for ${MODULE_NAME} repository operations
#[async_trait]
pub trait ${MODULE_NAME^}Repository: Send + Sync {
    /// Find by ID
    async fn find_by_id(&self, id: &str) -> Result<Option<${MODULE_NAME^}>>;
    
    /// Find all
    async fn find_all(&self) -> Result<Vec<${MODULE_NAME^}>>;
    
    // TODO: Add more methods based on legacy code requirements
}
EOF
    echo "  ✅ Created: $PORT_FILE"
else
    echo "  ⚠️  Exists: $PORT_FILE"
fi

# Use Case
USECASE_FILE="src/application/use_cases/${MODULE_NAME}_use_cases.rs"
if [ ! -f "$USECASE_FILE" ]; then
    cat > "$USECASE_FILE" << EOF
//! ${MODULE_NAME} use cases
//!
//! Application layer use cases for ${MODULE_NAME} operations.

use std::sync::Arc;
use anyhow::Result;
use crate::domain::ports::${MODULE_NAME}_repository::${MODULE_NAME^}Repository;

/// Use case for listing ${MODULE_NAME}s
pub struct List${MODULE_NAME^}sUseCase<R: ${MODULE_NAME^}Repository> {
    repository: Arc<R>,
}

impl<R: ${MODULE_NAME^}Repository> List${MODULE_NAME^}sUseCase<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }
    
    pub async fn execute(&self) -> Result<Vec<String>> {
        // TODO: Implement based on legacy code
        let items = self.repository.find_all().await?;
        Ok(items.into_iter().map(|i| i.name).collect())
    }
}

// TODO: Add more use cases based on legacy code requirements
EOF
    echo "  ✅ Created: $USECASE_FILE"
else
    echo "  ⚠️  Exists: $USECASE_FILE"
fi

# Repository Implementation
REPO_FILE="src/infrastructure/repositories/${MODULE_NAME}_repository.rs"
if [ ! -f "$REPO_FILE" ]; then
    cat > "$REPO_FILE" << EOF
//! ${MODULE_NAME} repository implementation
//!
//! Infrastructure layer implementation of the ${MODULE_NAME} repository port.

use async_trait::async_trait;
use anyhow::Result;
use crate::domain::entities::${MODULE_NAME}::${MODULE_NAME^};
use crate::domain::ports::${MODULE_NAME}_repository::${MODULE_NAME^}Repository;

/// Implementation of ${MODULE_NAME}Repository
pub struct ${MODULE_NAME^}RepositoryImpl {
    // TODO: Add required clients/connections
}

impl ${MODULE_NAME^}RepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ${MODULE_NAME^}Repository for ${MODULE_NAME^}RepositoryImpl {
    async fn find_by_id(&self, id: &str) -> Result<Option<${MODULE_NAME^}>> {
        // TODO: Implement based on legacy code
        todo!("Implement find_by_id")
    }
    
    async fn find_all(&self) -> Result<Vec<${MODULE_NAME^}>> {
        // TODO: Implement based on legacy code
        todo!("Implement find_all")
    }
}
EOF
    echo "  ✅ Created: $REPO_FILE"
else
    echo "  ⚠️  Exists: $REPO_FILE"
fi

# HTTP Handler
HANDLER_FILE="src/interfaces/http/${MODULE_NAME}_handlers.rs"
if [ ! -f "$HANDLER_FILE" ]; then
    cat > "$HANDLER_FILE" << EOF
//! ${MODULE_NAME} HTTP handlers
//!
//! Interface layer handlers for ${MODULE_NAME} endpoints.

use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

/// Get all ${MODULE_NAME}s
pub async fn get_${MODULE_NAME}s() -> impl Responder {
    // TODO: Implement using use case
    HttpResponse::Ok().json(serde_json::json!({
        "message": "${MODULE_NAME} list endpoint - TODO: implement"
    }))
}

/// Configure routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/api/${MODULE_NAME}s")
            .route(web::get().to(get_${MODULE_NAME}s))
    );
}
EOF
    echo "  ✅ Created: $HANDLER_FILE"
else
    echo "  ⚠️  Exists: $HANDLER_FILE"
fi

echo ""

# Migration Checklist
echo "📋 Migration Checklist for '$MODULE_NAME':"
echo "==================================="
echo ""
echo "1. Analyze legacy code:"
echo "   cat $LEGACY_FILE"
echo ""
echo "2. Define domain entity in:"
echo "   $ENTITY_FILE"
echo ""
echo "3. Define repository port in:"
echo "   $PORT_FILE"
echo ""
echo "4. Create use cases in:"
echo "   $USECASE_FILE"
echo ""
echo "5. Implement repository in:"
echo "   $REPO_FILE"
echo ""
echo "6. Create HTTP handlers in:"
echo "   $HANDLER_FILE"
echo ""
echo "7. Register handlers in src/interfaces/http/mod.rs"
echo ""
echo "8. Update src/domain/mod.rs to export new modules"
echo ""
echo "9. Write tests:"
echo "   - Unit tests for use cases"
echo "   - Integration tests for handlers"
echo ""
echo "10. Remove legacy file:"
echo "    rm $LEGACY_FILE"
echo ""
echo "11. Update src/legacy/mod.rs to remove the module"
echo ""
echo "✨ Happy refactoring!"
