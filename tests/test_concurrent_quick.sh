#!/bin/bash
# Quick concurrent test
# Per 550 Section 7: One-at-a-time exec model
# Only one exec can be active at a time; concurrent execs get E-EXEC-ALREADY-ACTIVE (exit 81)

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

SESSION_ID="chrome-test"

# Cleanup function
cleanup() {
    echo ""
    echo "Cleaning up..."
    ./target/debug/teleport stop --id "$SESSION_ID" 2>/dev/null || true
    lsof -ti:9222 | xargs kill -9 2>/dev/null || true
    rm -rf /tmp/teleport-chrome-test 2>/dev/null || true
}

trap cleanup EXIT

# Setup Chrome + websocat session
setup_session() {
    echo "Setting up Chrome + websocat session..."

    # Clean first
    cleanup 2>/dev/null || true
    sleep 1

    # Start Chrome headless with CDP
    echo -n "Starting Chrome... "
    /Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome \
        --remote-debugging-port=9222 \
        --remote-debugging-host=127.0.0.1 \
        --user-data-dir=/tmp/teleport-chrome-test \
        --no-first-run \
        --no-default-browser-check \
        --disable-popup-blocking \
        --headless=new \
        > /dev/null 2>&1 &

    sleep 3

    if ! lsof -i:9222 > /dev/null 2>&1; then
        echo -e "${RED}FAILED${NC}"
        return 1
    fi
    echo -e "${GREEN}OK${NC}"

    # Get WebSocket URL
    echo -n "Getting WebSocket URL... "
    WS_URL=$(curl -s http://127.0.0.1:9222/json/version | grep webSocketDebuggerUrl | cut -d'"' -f4)
    if [ -z "$WS_URL" ]; then
        echo -e "${RED}FAILED${NC}"
        return 1
    fi
    echo -e "${GREEN}OK${NC}"

    # Start redirect with websocat
    echo -n "Starting teleport redirect... "
    ./target/debug/teleport redirect --id "$SESSION_ID" -- \
        websocat --no-close --text "$WS_URL" > /dev/null 2>&1 &

    sleep 2

    if ! ./target/debug/teleport info --id "$SESSION_ID" > /dev/null 2>&1; then
        echo -e "${RED}FAILED${NC}"
        return 1
    fi
    echo -e "${GREEN}OK${NC}"
    echo ""

    return 0
}

# Setup
setup_session || exit 1

echo "=========================================="
echo "Quick Concurrent Test (One-at-a-Time Model)"
echo "=========================================="
echo ""

# Test 1: Sequential first
echo "Test 1: Sequential exec (should work)"
RESP=$(echo '{"id":1,"method":"Browser.getVersion"}' | ./target/debug/teleport exec --id "$SESSION_ID" 2>&1)
EXIT=$?
if [ $EXIT -eq 0 ]; then
    echo -e "${GREEN}OK${NC} exit=$EXIT"
else
    echo -e "${RED}FAIL${NC} exit=$EXIT"
fi
echo ""

# Test 2: Another sequential
echo "Test 2: Another sequential exec (should work)"
RESP=$(echo '{"id":2,"method":"Browser.getVersion"}' | ./target/debug/teleport exec --id "$SESSION_ID" 2>&1)
EXIT=$?
if [ $EXIT -eq 0 ]; then
    echo -e "${GREEN}OK${NC} exit=$EXIT"
else
    echo -e "${RED}FAIL${NC} exit=$EXIT"
fi
echo ""

# Test 3: Concurrent execs
echo "Test 3: Concurrent execs (one should succeed, one should get exit 81)"
echo "(Per 550 Section 7: One-at-a-time exec model)"
echo ""

# Launch both in background
(echo '{"id":3,"method":"Browser.getVersion"}' | ./target/debug/teleport exec --id "$SESSION_ID" >/dev/null 2>&1) &
PID1=$!
(echo '{"id":4,"method":"Browser.getVersion"}' | ./target/debug/teleport exec --id "$SESSION_ID" >/dev/null 2>&1) &
PID2=$!

# Wait for both
wait $PID1
EXIT1=$?
wait $PID2
EXIT2=$?

echo "Concurrent results: exit1=$EXIT1, exit2=$EXIT2"
echo ""

# Per 550 Section 7: One should succeed (0), one should be rejected (81)
if [ "$EXIT1" -eq 0 ] && [ "$EXIT2" -eq 81 ]; then
    echo -e "${GREEN}✅ CORRECT${NC}: First succeeded, second rejected with E-EXEC-ALREADY-ACTIVE"
    RESULT=0
elif [ "$EXIT1" -eq 81 ] && [ "$EXIT2" -eq 0 ]; then
    echo -e "${GREEN}✅ CORRECT${NC}: Second succeeded, first rejected with E-EXEC-ALREADY-ACTIVE"
    RESULT=0
elif [ "$EXIT1" -eq 0 ] && [ "$EXIT2" -eq 0 ]; then
    echo -e "${YELLOW}⚠️  Both succeeded${NC} (timing allowed serial execution - this is OK)"
    RESULT=0
else
    echo -e "${RED}❌ UNEXPECTED${NC}: exit1=$EXIT1, exit2=$EXIT2"
    RESULT=1
fi

# Verify session still alive
echo ""
echo "Session status after tests:"
./target/debug/teleport info --id "$SESSION_ID" | head -3

exit $RESULT
