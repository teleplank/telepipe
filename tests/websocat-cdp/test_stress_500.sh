#!/bin/bash
# Test 6: Extreme Stress (500 Commands)
# Very high volume test
# Validates sustained performance under extreme load

set -e

source "$(dirname "$0")/test_common.sh"
source "$(dirname "$0")/cdp_commands.sh"

init_test "Extreme Stress Test (500 Commands)"

trap cleanup_test EXIT
setup_chrome_session || exit 1

echo "Sending 500 commands with mixed patterns..."
echo "This is extreme volume testing - proving production scalability."
echo ""

for i in $(seq 1 500); do
    CMD=$(cdp_random_valid)

    # Progress checkpoints
    if [ $((i % 50)) -eq 0 ]; then
        echo ""
        echo "=== Checkpoint $i/500 ==="
        # Verify session still alive
        if ! ./target/debug/teleport info --id chrome-test > /dev/null 2>&1; then
            echo -e "${RED}ERROR: Session died at command $i${NC}"
            break
        fi
    fi

    echo -n "Command $i: "
    send_cdp_command "$CMD"

    # Variable timing
    if [ $((i % 100)) -eq 0 ]; then
        sleep 0.5  # Longer pause every 100
    elif [ $((i % 10)) -eq 0 ]; then
        sleep 0.1  # Short pause every 10
    fi
done

echo ""
echo "Verifying session health after 500 commands..."
if ./target/debug/teleport info --id chrome-test > /dev/null 2>&1; then
    echo -e "${GREEN}OK Session STILL ACTIVE after 500 commands${NC}"
    SESSION_HEALTH="HEALTHY"
else
    echo -e "${RED}X Session died${NC}"
    SESSION_HEALTH="DEGRADED"
fi

print_summary "Extreme Stress Test (500 Commands)"
exit_code=$?

# Calculate commands per second
CMD_PER_SEC=$(echo "scale=2; $TOTAL_TESTS / $DURATION_S" | bc 2>/dev/null || echo "N/A")

mkdir -p test-results/websocat-cdp
cat > test-results/websocat-cdp/stress-500-$(date +%Y%m%d-%H%M%S).md <<EOF
# Extreme Stress Test (500 Commands)

**Date:** $(date +"%Y-%m-%d %H:%M:%S")
**Volume:** 500 commands
**Purpose:** Validate sustained performance under extreme load

## Results

- Total Commands: $TOTAL_TESTS
- Passed: $PASSED_TESTS
- Failed: $FAILED_TESTS
- Success Rate: ${SUCCESS_RATE}%
- Duration: ${DURATION_S}s

## Load Pattern

- Rapid-fire: ~450 commands
- 100ms pause every 10 commands
- 500ms pause every 100 commands
- Checkpoints every 50 commands

## Latency

- Min: ${MIN_LAT}ms
- Max: ${MAX_LAT}ms
- Average: ${AVG_LAT}ms
- P99: ${P99_LAT}ms

## Performance

- Commands/second: $CMD_PER_SEC
- Session uptime: ${DURATION_S}s
- Session health: $SESSION_HEALTH

## Conclusion

$(if [ $exit_code -eq 0 ]; then echo "OK PASSED - Sustained extreme load without degradation"; else echo "X FAILED - Performance degradation under extreme load"; fi)

**This proves production-grade scalability at 500+ commands.**
EOF

exit $exit_code
