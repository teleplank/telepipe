#!/bin/bash
# Chaos Engineering Infrastructure
# Provides chaos injection, concurrency testing, and extreme stress utilities

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Chaos settings
CHAOS_ENABLED=${CHAOS_ENABLED:-false}
CHAOS_FAILURE_RATE=${CHAOS_FAILURE_RATE:-0.1}  # 10% failure injection rate

# Concurrency tracking
declare -a CONCURRENT_PIDS
declare -a CONCURRENT_RESULTS
declare -a CONCURRENT_LATENCIES

# Inject random failure
chaos_maybe_inject_failure() {
    if [ "$CHAOS_ENABLED" = "true" ]; then
        local rand=$((RANDOM % 100))
        local threshold=$(echo "$CHAOS_FAILURE_RATE * 100" | bc | cut -d. -f1)

        if [ $rand -lt $threshold ]; then
            local chaos_types=("invalid_json" "invalid_method" "network_delay" "timeout")
            local chaos_idx=$((RANDOM % ${#chaos_types[@]}))
            echo "${chaos_types[$chaos_idx]}"
            return 0
        fi
    fi
    echo "none"
    return 1
}

# Inject network delay
chaos_network_delay() {
    if [ "$CHAOS_ENABLED" = "true" ]; then
        local delay=$((RANDOM % 500 + 100))  # 100-600ms random delay
        sleep $(echo "scale=3; $delay / 1000" | bc)
    fi
}

# Send concurrent command (background)
send_concurrent_cdp() {
    local command="$1"
    local worker_id="$2"
    local output_file="$3"

    local start_ns=$(python3 -c "import time; print(int(time.time() * 1000000000))")

    # Send command
    RESPONSE=$(echo "$command" | ./target/debug/teleport exec --id chrome-test 2>&1)
    EXIT_CODE=$?

    local end_ns=$(python3 -c "import time; print(int(time.time() * 1000000000))")
    local latency_ms=$(( (end_ns - start_ns) / 1000000 ))

    # Write result to file
    echo "$EXIT_CODE|$latency_ms|$RESPONSE" > "$output_file"
}

# Wait for all concurrent commands
# Per 550 Section 7: One-at-a-time exec model
# Only one exec succeeds; others get E-EXEC-ALREADY-ACTIVE (exit 81)
wait_concurrent_commands() {
    local num_workers=$1
    local temp_dir="$2"

    # Wait for all background jobs
    wait

    local passed=0
    local rejected=0  # Exit 81 - E-EXEC-ALREADY-ACTIVE (expected for concurrent)
    local failed=0
    declare -a latencies

    # Collect results
    for i in $(seq 1 $num_workers); do
        if [ -f "$temp_dir/worker_$i.result" ]; then
            IFS='|' read -r exit_code latency response < "$temp_dir/worker_$i.result"

            latencies+=($latency)

            if [ "$exit_code" -eq 0 ]; then
                if ! echo "$response" | grep -q "ERROR E-TCP-CONNECT"; then
                    if echo "$response" | python3 -m json.tool > /dev/null 2>&1; then
                        passed=$((passed + 1))
                        echo -e "  Worker $i: ${GREEN}OK${NC} [$latency ms]"
                    else
                        failed=$((failed + 1))
                        echo -e "  Worker $i: ${RED}X${NC} (invalid JSON) [$latency ms]"
                    fi
                else
                    failed=$((failed + 1))
                    echo -e "  Worker $i: ${RED}X${NC} (E-TCP-CONNECT) [$latency ms]"
                fi
            elif [ "$exit_code" -eq 81 ]; then
                # Per 550 Section 7: E-EXEC-ALREADY-ACTIVE is expected for concurrent execs
                rejected=$((rejected + 1))
                echo -e "  Worker $i: ${YELLOW}REJECTED${NC} (exit 81 - one-at-a-time) [$latency ms]"
            else
                failed=$((failed + 1))
                echo -e "  Worker $i: ${RED}X${NC} (exit $exit_code) [$latency ms]"
            fi
        else
            failed=$((failed + 1))
            echo -e "  Worker $i: ${RED}X${NC} (no result file)"
        fi
    done

    # Calculate concurrency stats
    # Per 550 Section 7: Only count actual failures (rejected is expected behavior)
    local total=$((passed + rejected + failed))
    local success_rate=0
    [ $total -gt 0 ] && success_rate=$(( (passed + rejected) * 100 / total ))

    local min_lat=${latencies[0]:-0}
    local max_lat=${latencies[0]:-0}
    local sum_lat=0

    for lat in "${latencies[@]}"; do
        sum_lat=$((sum_lat + lat))
        [ $lat -lt $min_lat ] && min_lat=$lat
        [ $lat -gt $max_lat ] && max_lat=$lat
    done

    local avg_lat=0
    [ ${#latencies[@]} -gt 0 ] && avg_lat=$((sum_lat / ${#latencies[@]}))

    echo ""
    echo "  Concurrent Results: $passed succeeded, $rejected rejected (one-at-a-time), $failed failed"
    echo "  Latency: min=${min_lat}ms, max=${max_lat}ms, avg=${avg_lat}ms"

    # Export for reporting
    export CONCURRENT_PASSED=$passed
    export CONCURRENT_REJECTED=$rejected
    export CONCURRENT_FAILED=$failed
    export CONCURRENT_SUCCESS_RATE=$success_rate
    export CONCURRENT_MIN_LAT=$min_lat
    export CONCURRENT_MAX_LAT=$max_lat
    export CONCURRENT_AVG_LAT=$avg_lat

    # Per 550 Section 7: One-at-a-time model means:
    # - Exactly 1 should pass (the one that got the lock)
    # - Others should be rejected (exit 81) - this is CORRECT behavior
    # - Only actual errors (failed > 0) indicate a problem
    return $([ $failed -eq 0 ] && [ $passed -ge 1 ] && echo 0 || echo 1)
}

# Verify no response mixing (critical for concurrency)
verify_no_response_mixing() {
    local temp_dir="$1"
    local num_workers=$2

    echo "Verifying response IDs (no mixing)..."

    local mixed=0

    for i in $(seq 1 $num_workers); do
        if [ -f "$temp_dir/worker_$i.result" ]; then
            IFS='|' read -r exit_code latency response < "$temp_dir/worker_$i.result"

            # Extract ID from response if JSON
            if echo "$response" | python3 -m json.tool > /dev/null 2>&1; then
                local response_id=$(echo "$response" | python3 -c "import sys, json; print(json.load(sys.stdin).get('id', -1))" 2>/dev/null)

                # Each worker should get response with ID matching their request
                if [ "$response_id" != "$i" ] && [ "$response_id" != "-1" ]; then
                    echo -e "  ${RED}X${NC} Worker $i got response ID $response_id (MIXED!)"
                    mixed=$((mixed + 1))
                fi
            fi
        fi
    done

    if [ $mixed -eq 0 ]; then
        echo -e "  ${GREEN}OK${NC} No response mixing detected"
        return 0
    else
        echo -e "  ${RED}X${NC} Response mixing detected: $mixed cases"
        return 1
    fi
}

# Kill and restart session (chaos)
chaos_kill_and_restart_session() {
    echo -e "${YELLOW}[CHAOS] Killing and restarting session...${NC}"

    # Kill teleport session
    ./target/debug/teleport stop --id chrome-test 2>/dev/null || true
    sleep 1

    # Get WebSocket URL
    WS_URL=$(curl -s http://127.0.0.1:9222/json/version | grep webSocketDebuggerUrl | cut -d'"' -f4)

    # Restart redirect
    ./target/debug/teleport redirect --id chrome-test -- \
        websocat --no-close --text "$WS_URL" > /dev/null 2>&1 &

    sleep 2

    if ./target/debug/teleport info --id chrome-test > /dev/null 2>&1; then
        echo -e "${GREEN}[CHAOS] Session restarted successfully${NC}"
        return 0
    else
        echo -e "${RED}[CHAOS] Session restart FAILED${NC}"
        return 1
    fi
}

# Random chaos event
chaos_random_event() {
    local events=("network_delay" "cpu_spike" "nothing" "nothing" "nothing")
    local event_idx=$((RANDOM % ${#events[@]}))
    local event="${events[$event_idx]}"

    case $event in
        "network_delay")
            echo -e "${YELLOW}[CHAOS] Injecting network delay...${NC}"
            sleep $((RANDOM % 2 + 1))
            ;;
        "cpu_spike")
            echo -e "${YELLOW}[CHAOS] Simulating CPU spike...${NC}"
            # Simulate CPU load (brief)
            for i in {1..5}; do
                yes > /dev/null 2>&1 &
            done
            sleep 0.5
            killall yes 2>/dev/null || true
            ;;
        *)
            # Most of the time, no chaos
            ;;
    esac
}
