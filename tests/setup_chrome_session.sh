#!/bin/bash
# Quick setup script for Chrome+teleport session

# Kill any existing
./target/debug/teleport stop --id chrome-test 2>/dev/null || true
killall "Google Chrome" 2>/dev/null || true
sleep 2

# Start Chrome
/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome \
    --remote-debugging-port=9222 \
    --headless=new \
    --disable-gpu \
    --no-sandbox \
    --disable-dev-shm-usage \
    --disable-extensions \
    --disable-background-networking \
    --disable-sync \
    --disable-translate \
    --disable-default-apps \
    --window-size=1920,1080 \
    "about:blank" 2>/dev/null &

sleep 3

# Get WebSocket URL
WS_URL=$(curl -s http://127.0.0.1:9222/json/version | grep webSocketDebuggerUrl | cut -d'"' -f4)
echo "WebSocket URL: $WS_URL"

if [ -z "$WS_URL" ]; then
    echo "Failed to get WebSocket URL"
    exit 1
fi

# Start teleport redirect
./target/debug/teleport redirect --id chrome-test -- \
    websocat --no-close --text "$WS_URL" > /dev/null 2>&1 &

sleep 2

# Verify
if ./target/debug/teleport info --id chrome-test > /dev/null 2>&1; then
    echo "Session ready!"
    ./target/debug/teleport info --id chrome-test | head -3
else
    echo "Session failed to start"
    exit 1
fi
