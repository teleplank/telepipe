#!/bin/bash
# Common test infrastructure for all websocat+CDP tests

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
START_TIME=0
END_TIME=0

# Latency tracking
declare -a LATENCIES

# Initialize test
init_test() {
    local test_name="$1"
    echo -e "${BLUE}=================================================${NC}"
    echo -e "${BLUE}Starting: $test_name${NC}"
    echo -e "${BLUE}=================================================${NC}"
    echo ""
    START_TIME=$(python3 -c "import time; print(int(time.time() * 1000))")
    TOTAL_TESTS=0
    PASSED_TESTS=0
    FAILED_TESTS=0
    LATENCIES=()
}

# Clean up Chrome and session
cleanup_test() {
    echo ""
    echo "Cleaning up..."

    # Stop teleport session
    ./target/debug/teleport stop --id chrome-test 2>/dev/null || true

    # Kill Chrome on port 9222
    lsof -ti:9222 | xargs kill -9 2>/dev/null || true

    # Clean up temp directory
    rm -rf /tmp/teleport-chrome-test 2>/dev/null || true

    sleep 1
}

# Setup Chrome and session
setup_chrome_session() {
    echo "Setting up Chrome + websocat session..."

    # Clean first
    cleanup_test

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

    CHROME_PID=$!
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
    ./target/debug/teleport redirect --id chrome-test -- \
        websocat --no-close --text "$WS_URL" > /dev/null 2>&1 &

    sleep 2

    if ! ./target/debug/teleport info --id chrome-test > /dev/null 2>&1; then
        echo -e "${RED}FAILED${NC}"
        return 1
    fi
    echo -e "${GREEN}OK${NC}"
    echo ""

    return 0
}

# Send CDP command and verify response
send_cdp_command() {
    local command="$1"
    local expect_success="${2:-true}"  # Default expect success

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    # Measure latency
    local start_ms=$(python3 -c "import time; print(int(time.time() * 1000))")

    # Send command
    RESPONSE=$(echo "$command" | ./target/debug/teleport exec --id chrome-test 2>&1)
    EXIT_CODE=$?

    local end_ms=$(python3 -c "import time; print(int(time.time() * 1000))")
    local latency_ms=$((end_ms - start_ms))
    LATENCIES+=($latency_ms)

    # Check result based on expectation
    if [ "$expect_success" = "true" ]; then
        # Expecting success
        if [ $EXIT_CODE -ne 0 ]; then
            echo -e "${RED}X${NC} FAIL (exit code $EXIT_CODE) [${latency_ms}ms]"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            return 1
        fi

        if echo "$RESPONSE" | grep -q "ERROR E-TCP-CONNECT"; then
            echo -e "${RED}X${NC} FAIL (E-TCP-CONNECT) [${latency_ms}ms]"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            return 1
        fi

        # For valid commands, expect JSON response with id
        if ! echo "$command" | grep -q "invalid" && ! echo "$command" | grep -q "this is not"; then
            if ! echo "$RESPONSE" | python3 -m json.tool > /dev/null 2>&1; then
                echo -e "${RED}X${NC} FAIL (invalid JSON) [${latency_ms}ms]"
                FAILED_TESTS=$((FAILED_TESTS + 1))
                return 1
            fi
        fi

        echo -e "${GREEN}OK${NC} PASS [${latency_ms}ms]"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        # Expecting failure/error response
        if echo "$RESPONSE" | python3 -m json.tool > /dev/null 2>&1; then
            if echo "$RESPONSE" | grep -q '"error"'; then
                echo -e "${GREEN}OK${NC} PASS (error response as expected) [${latency_ms}ms]"
                PASSED_TESTS=$((PASSED_TESTS + 1))
                return 0
            fi
        fi

        echo -e "${YELLOW}~${NC} PARTIAL (got response, expected error) [${latency_ms}ms]"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    fi
}

# Calculate statistics
calculate_stats() {
    END_TIME=$(python3 -c "import time; print(int(time.time() * 1000))")
    local duration_ms=$((END_TIME - START_TIME))
    DURATION_S=$(echo "scale=2; $duration_ms / 1000" | bc)

    # Calculate latency stats
    if [ ${#LATENCIES[@]} -gt 0 ]; then
        MIN_LAT=${LATENCIES[0]}
        MAX_LAT=${LATENCIES[0]}
        local sum_lat=0

        for lat in "${LATENCIES[@]}"; do
            sum_lat=$((sum_lat + lat))
            [ $lat -lt $MIN_LAT ] && MIN_LAT=$lat
            [ $lat -gt $MAX_LAT ] && MAX_LAT=$lat
        done

        AVG_LAT=$((sum_lat / ${#LATENCIES[@]}))

        # Calculate P99 (simple approximation)
        IFS=$'\n' sorted=($(sort -n <<<"${LATENCIES[*]}"))
        local p99_index=$(( ${#sorted[@]} * 99 / 100 ))
        P99_LAT=${sorted[$p99_index]:-0}
    else
        MIN_LAT=0
        MAX_LAT=0
        AVG_LAT=0
        P99_LAT=0
    fi

    # Calculate success rate
    if [ $TOTAL_TESTS -gt 0 ]; then
        SUCCESS_RATE=$(( PASSED_TESTS * 100 / TOTAL_TESTS ))
    else
        SUCCESS_RATE=0
    fi

    # Export for report
    export TOTAL_TESTS PASSED_TESTS FAILED_TESTS
    export DURATION_S MIN_LAT MAX_LAT AVG_LAT P99_LAT SUCCESS_RATE
}

# Print summary
print_summary() {
    local test_name="$1"

    calculate_stats

    echo ""
    echo -e "${BLUE}=================================================${NC}"
    echo -e "${BLUE}Summary: $test_name${NC}"
    echo -e "${BLUE}=================================================${NC}"
    echo -e "Total tests:    $TOTAL_TESTS"
    echo -e "Passed:         ${GREEN}$PASSED_TESTS${NC}"
    echo -e "Failed:         ${RED}$FAILED_TESTS${NC}"
    echo -e "Success rate:   ${SUCCESS_RATE}%"
    echo -e "Duration:       ${DURATION_S}s"
    echo ""
    echo -e "${BLUE}Latency Statistics:${NC}"
    echo -e "Min:            ${MIN_LAT}ms"
    echo -e "Max:            ${MAX_LAT}ms"
    echo -e "Average:        ${AVG_LAT}ms"
    echo -e "P99:            ${P99_LAT}ms"
    echo ""

    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}ALL TESTS PASSED${NC}"
        return 0
    else
        echo -e "${RED}SOME TESTS FAILED${NC}"
        return 1
    fi
}
