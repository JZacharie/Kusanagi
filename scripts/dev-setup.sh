#!/bin/bash
# Kusanagi Development Setup Script
# Usage: ./scripts/dev-setup.sh

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔮 Kusanagi Development Setup${NC}"
echo "================================"

# Check OS
OS="$(uname -s)"
echo -e "${BLUE}Detected OS: $OS${NC}"

# =============================================================================
# Check Prerequisites
# =============================================================================
echo ""
echo -e "${BLUE}Checking prerequisites...${NC}"

# Check Rust
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo -e "${GREEN}✓ Rust installed: $RUST_VERSION${NC}"
else
    echo -e "${RED}✗ Rust not found${NC}"
    echo "Please install Rust: https://rustup.rs/"
    exit 1
fi

# Check Cargo
if command -v cargo &> /dev/null; then
    CARGO_VERSION=$(cargo --version)
    echo -e "${GREEN}✓ Cargo installed: $CARGO_VERSION${NC}"
else
    echo -e "${RED}✗ Cargo not found${NC}"
    exit 1
fi

# Check Docker (optional)
if command -v docker &> /dev/null; then
    DOCKER_VERSION=$(docker --version)
    echo -e "${GREEN}✓ Docker installed: $DOCKER_VERSION${NC}"
else
    echo -e "${YELLOW}⚠ Docker not found (recommended for full development)${NC}"
fi

# Check kubectl (optional)
if command -v kubectl &> /dev/null; then
    KUBECTL_VERSION=$(kubectl version --client -o json | grep -o '"gitVersion": "[^"]*"' | head -1 | cut -d'"' -f4)
    echo -e "${GREEN}✓ kubectl installed: $KUBECTL_VERSION${NC}"
else
    echo -e "${YELLOW}⚠ kubectl not found (optional)${NC}"
fi

# =============================================================================
# Install Rust Tools
# =============================================================================
echo ""
echo -e "${BLUE}Installing Rust development tools...${NC}"

TOOLS=(
    "cargo-watch:Hot reload"
    "cargo-tarpaulin:Code coverage"
    "cargo-audit:Security audit"
    "cargo-deny:Dependency checking"
    "cargo-udeps:Unused dependencies"
    "cargo-edit:Dependency management"
    "cargo-expand:Macro expansion"
    "cargo-nextest:Better test runner"
)

for tool_info in "${TOOLS[@]}"; do
    IFS=':' read -r tool desc <<< "$tool_info"
    if cargo install --list | grep -q "^$tool "; then
        echo -e "${GREEN}✓ $tool already installed${NC}"
    else
        echo -e "${BLUE}  Installing $tool ($desc)...${NC}"
        cargo install --locked $tool 2>&1 | grep -E "(Installing|Updated|Finished)" || true
    fi
done

# =============================================================================
# Setup Git Hooks
# =============================================================================
echo ""
echo -e "${BLUE}Setting up Git hooks...${NC}"

mkdir -p .git/hooks

# Pre-commit hook
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
# Kusanagi Pre-commit Hook

echo "🔍 Running pre-commit checks..."

# Format check
echo "  Checking formatting..."
if ! cargo fmt -- --check; then
    echo "❌ Formatting check failed. Run 'cargo fmt' to fix."
    exit 1
fi

# Clippy check
echo "  Running clippy..."
if ! cargo clippy -- -D warnings; then
    echo "❌ Clippy warnings found. Please fix them."
    exit 1
fi

# Unit tests
echo "  Running unit tests..."
if ! cargo test --lib; then
    echo "❌ Tests failed. Please fix them."
    exit 1
fi

echo "✅ All checks passed!"
EOF

chmod +x .git/hooks/pre-commit
echo -e "${GREEN}✓ Pre-commit hook installed${NC}"

# Pre-push hook (optional, lighter than full CI)
cat > .git/hooks/pre-push << 'EOF'
#!/bin/bash
# Kusanagi Pre-push Hook

echo "🚀 Running pre-push checks..."

# Quick check
if ! cargo check; then
    echo "❌ Compilation failed."
    exit 1
fi

# Clippy
if ! cargo clippy -- -D warnings; then
    echo "❌ Clippy warnings found."
    exit 1
fi

echo "✅ Pre-push checks passed!"
EOF

chmod +x .git/hooks/pre-push
echo -e "${GREEN}✓ Pre-push hook installed${NC}"

# =============================================================================
# Setup IDE Configuration
# =============================================================================
echo ""
echo -e "${BLUE}Setting up IDE configuration...${NC}"

# VSCode settings
if command -v code &> /dev/null || [ -d ".vscode" ]; then
    mkdir -p .vscode
    
    # settings.json
    cat > .vscode/settings.json << 'EOF'
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.checkOnSave.extraArgs": ["--", "-D", "warnings"],
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "files.watcherExclude": {
        "**/target/**": true,
        "**/.git/**": true
    }
}
EOF
    echo -e "${GREEN}✓ VSCode settings created${NC}"
    
    # extensions.json
    cat > .vscode/extensions.json << 'EOF'
{
    "recommendations": [
        "rust-lang.rust-analyzer",
        "serayuzgur.crates",
        "vadimcn.vscode-lldb",
        "tamasfe.even-better-toml",
        "streetsidesoftware.code-spell-checker"
    ]
}
EOF
    echo -e "${GREEN}✓ VSCode extensions recommendations created${NC}"
fi

# =============================================================================
# Environment Setup
# =============================================================================
echo ""
echo -e "${BLUE}Setting up environment...${NC}"

# Create .env file if it doesn't exist
if [ ! -f ".env" ]; then
    cp .env.template .env
    echo -e "${GREEN}✓ Created .env from template${NC}"
    echo -e "${YELLOW}⚠ Please edit .env with your configuration${NC}"
else
    echo -e "${GREEN}✓ .env already exists${NC}"
fi

# =============================================================================
# Build Project
# =============================================================================
echo ""
echo -e "${BLUE}Building project...${NC}"

if cargo build; then
    echo -e "${GREEN}✓ Project built successfully${NC}"
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi

# =============================================================================
# Summary
# =============================================================================
echo ""
echo "================================"
echo -e "${GREEN}✅ Setup Complete!${NC}"
echo ""
echo -e "${BLUE}Available commands:${NC}"
echo "  ${GREEN}make run${NC}          - Run in development mode"
echo "  ${GREEN}make test${NC}         - Run all tests"
echo "  ${GREEN}make lint${NC}         - Run linter"
echo "  ${GREEN}make coverage${NC}     - Generate coverage report"
echo "  ${GREEN}cargo watch -x run${NC} - Run with hot reload"
echo ""
echo -e "${BLUE}Development workflow:${NC}"
echo "  1. Edit code"
echo "  2. Git hooks will run checks automatically"
echo "  3. Use 'cargo watch -x run' for continuous development"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "  1. Edit ${GREEN}.env${NC} with your configuration"
echo "  2. Run ${GREEN}make run${NC} to start the server"
echo "  3. Open http://localhost:8080"
echo ""
echo -e "${BLUE}Happy coding! 🦀${NC}"
