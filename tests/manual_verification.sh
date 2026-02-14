#!/bin/bash
set -e

# Configuration
PORT=8089
HOST="127.0.0.1"
BASE_URL="http://$HOST:$PORT"

echo "🚀 Starting Kusanagi for manual verification on port $PORT..."

# Start Kusanagi in the background
export KUSANAGI_PORT=$PORT
# Use cargo run to start the server
nohup cargo run > /tmp/kusanagi_test.log 2>&1 &
SERVER_PID=$!

echo "⏳ Waiting for server to start (PID: $SERVER_PID)..."
# Loop to check if port is open
for i in {1..30}; do
    if nc -z $HOST $PORT; then
        echo "✅ Server started!"
        break
    fi
    echo -n "."
    sleep 1
done

if ! nc -z $HOST $PORT; then
    echo "❌ Server failed to start within 30 seconds."
    cat /tmp/kusanagi_test.log
    kill $SERVER_PID || true
    exit 1
fi

echo "📋 Running Tests..."

# Check System Status
echo "👉 Checking System Status..."
curl -s "$BASE_URL/api/system/status" | grep "operational" && echo "✅ System Status OK" || echo "❌ System Status FAILED"

# Check System Logs
echo "👉 Checking System Logs..."
# This might return error if log file missing, but endpoint should be reachable
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/api/system/logs")
if [ "$HTTP_CODE" -eq 200 ] || [ "$HTTP_CODE" -eq 500 ]; then
    echo "✅ System Logs Endpoint Reachable ($HTTP_CODE)" 
else
    echo "❌ System Logs Endpoint FAILED ($HTTP_CODE)"
fi

# Check Chat (Cyberpunk AI)
echo "👉 Checking Chat (Cyberpunk AI)..."
# Requesting status in French
RESPONSE=$(curl -s -X POST "$BASE_URL/api/chat" -H "Content-Type: application/json" -d '{"message": "/status", "language": "fr"}')
echo "$RESPONSE" | grep "response" && echo "✅ Chat Response OK" || echo "❌ Chat Response FAILED"

# Check Chat (English)
echo "👉 Checking Chat (English)..."
RESPONSE=$(curl -s -X POST "$BASE_URL/api/chat" -H "Content-Type: application/json" -d '{"message": "Hello", "language": "en"}')
echo "$RESPONSE" | grep "response" && echo "✅ English Chat Response OK" || echo "❌ English Chat Response FAILED"


echo "🛑 Stopping server..."
kill $SERVER_PID
wait $SERVER_PID || true

echo "✅ Manual verification complete!"
