#!/bin/bash
# Load agent context for AI assistants
# Usage: source .agent/load-context.sh

echo "=== Kusanagi Agent Context Loader ==="
echo ""
echo "📋 Core Context (REQUIRED):"
echo "   - .agent/AGENT_CONTEXT.md"
echo ""
echo "📚 Skills (as needed):"
echo "   - .agent/skill/00-project-overview.md"
echo "   - .agent/skill/01-backend-architecture.md"
echo "   - .agent/skill/02-frontend-patterns.md"
echo "   - .agent/skill/03-rust-conventions.md"
echo "   - .agent/skill/04-api-reference.md"
echo ""
echo "📁 Archived reports: .agent/archive/"
echo ""
echo "Total tokens: ~3,000 for complete context"
echo ""
echo "Quick start:"
echo "   cat .agent/AGENT_CONTEXT.md"
