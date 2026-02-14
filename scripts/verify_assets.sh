#!/bin/bash
set -e

# Start the server in the background
echo "🚀 Starting server for verification..."
BINARY=${BINARY_PATH:-"./target/debug/kusanagi"}
echo "Using binary: $BINARY"
$BINARY &
SERVER_PID=$!

# Wait for server to be ready
echo "⏳ Waiting for server..."
sleep 5

# Fetch the index page
echo "📥 Fetching index.html..."
CONTENT=$(curl -s http://localhost:8080/)

# Kill the server
kill $SERVER_PID || true

# Check for version placeholder replacement
if echo "$CONTENT" | grep -q "{{VERSION}}"; then
    echo "❌ FAILED: Found unreplaced {{VERSION}} placeholder!"
    exit 1
fi

if echo "$CONTENT" | grep -q "?v="; then
    echo "✅ SUCCESS: Found versioned assets!"
    # Extract one version to show it
    VERSION=$(echo "$CONTENT" | grep -o 'v=[^"]*' | head -n1)
    echo "ℹ️  Asset version: $VERSION"
else
    echo "❌ FAILED: No versioned assets found!"
    exit 1
fi
