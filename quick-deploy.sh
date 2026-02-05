#!/bin/bash
set -e

echo "🚀 Quick Deploy Kusanagi"

# Build
echo "📦 Building..."
cargo build --release

# Stop existing
echo "🛑 Stopping existing service..."
pkill -f kusanagi || true

# Deploy
echo "🚢 Deploying..."
cp target/release/kusanagi /tmp/kusanagi-new
mv /tmp/kusanagi-new ./kusanagi

# Start
echo "▶️  Starting..."
nohup ./kusanagi > kusanagi.log 2>&1 &

echo "✅ Deployed! PID: $!"
echo "📋 Logs: tail -f kusanagi.log"
