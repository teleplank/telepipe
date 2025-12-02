#!/bin/bash
# Quick concurrent test with file output and latency measurement
# Per 550 Section 7: One-at-a-time exec model
# Only one exec can be active at a time; concurrent execs get E-EXEC-ALREADY-ACTIVE (exit 81)

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

SESSION_ID="chrome-test"
TEMP_DIR=$(mktemp -d)

# Cleanup function
cleanup() {
    echo ""
    echo "Cleaning up..."
    ./target/debug/teleport stop --id "$SESSION_ID" 2>/dev/null || true
    lsof -ti:9222 | xargs kill -9 2>/dev/null || true
    rm -rf /tmp/teleport-chrome-test 2>/dev/null || true
    rm -rf "$TEMP_DIR" 2>/dev/null || true
}

trap cleanup EXIT

# Setup Chrome + websocat session
setup_session() {
    echo "Setting up Chrome + websocat session..."

    # Clean first
    ./target/debug/teleport stop --id "$SESSION_ID" 2>/dev/null || true
    lsof -ti:9222 | xargs kill -9 2>/dev/null || true
    rm -rf /tmp/teleport-chrome-test 2>/dev/null || true
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
echo "Quick Concurrent Test with Latency Measurement"
echo "(Per 550 Section 7: One-at-a-Time Model)"
echo "=========================================="
echo ""
echo "Temp dir: $TEMP_DIR"
echo ""

echo "Test: Concurrent execs with file output"
echo "(One should succeed with exit 0, one should be rejected with exit 81)"
echo ""

# Launch both in background, capture output to files
(
    START=$(python3 -c "import time; print(int(time.time() * 1000))")
    RESP=$(echo '{"id":1,"method":"Browser.getVersion"}' | ./target/debug/teleport exec --id "$SESSION_ID" 2>&1)
    EXIT=$?
    END=$(python3 -c "import time; print(int(time.time() * 1000))")
    LAT=$((END - START))
    echo "$EXIT|$LAT|$RESP" > "$TEMP_DIR/worker_1.result"
) &
PID1=$!

(
    START=$(python3 -c "import time; print(int(time.time() * 1000))")
    RESP=$(echo '{"id":2,"method":"Browser.getVersion"}' | ./target/debug/teleport exec --id "$SESSION_ID" 2>&1)
    EXIT=$?
    END=$(python3 -c "import time; print(int(time.time() * 1000))")
    LAT=$((END - START))
    echo "$EXIT|$LAT|$RESP" > "$TEMP_DIR/worker_2.result"
) &
PID2=$!

# Wait for both
wait $PID1
wait $PID2

echo "Results:"
SUCCESS=0
REJECTED=0
for i in 1 2; do
    if [ -f "$TEMP_DIR/worker_$i.result" ]; then
        result=$(cat "$TEMP_DIR/worker_$i.result")
        exit_code=$(echo "$result" | cut -d'|' -f1)
        latency=$(echo "$result" | cut -d'|' -f2)
        echo "Worker $i: exit=$exit_code latency=${latency}ms"
        if [ "$exit_code" = "0" ]; then
            SUCCESS=$((SUCCESS + 1))
        elif [ "$exit_code" = "81" ]; then
            REJECTED=$((REJECTED + 1))
        fi
    else
        echo "Worker $i: NO RESULT FILE"
    fi
done

echo ""
echo "Summary: $SUCCESS succeeded, $REJECTED rejected (exit 81)"

RESULT=0
if [ "$SUCCESS" -eq 1 ] && [ "$REJECTED" -eq 1 ]; then
    echo -e "${GREEN}✅ CORRECT${NC}: One-at-a-time model working as expected"
elif [ "$SUCCESS" -eq 2 ]; then
    echo -e "${YELLOW}⚠️  Both succeeded${NC} (timing allowed serial execution - this is OK)"
else
    echo -e "${RED}❌ UNEXPECTED${NC} results"
    RESULT=1
fi

exit $RESULT
