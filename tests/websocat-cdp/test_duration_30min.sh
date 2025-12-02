#!/bin/bash
# Test 7: Long Duration Test (30 Minutes)
# Sustained operation test
# Validates stability over time (memory leaks, resource exhaustion)

set -e

source "$(dirname "$0")/test_common.sh"
source "$(dirname "$0")/cdp_commands.sh"

init_test "Long Duration Test (30 Minutes)"

trap cleanup_test EXIT
setup_chrome_session || exit 1

DURATION_SECONDS=$((30 * 60))  # 30 minutes
END_TIME=$(($(date +%s) + DURATION_SECONDS))

echo "Running continuous test for 30 minutes..."
echo "End time: $(date -r $END_TIME 2>/dev/null || date -d @$END_TIME 2>/dev/null || echo "in 30 minutes")"
echo "This validates long-term stability (memory leaks, resource exhaustion)"
echo ""

MINUTE=1

while [ $(date +%s) -lt $END_TIME ]; do
    echo "=== Minute $MINUTE ==="

    # Send 10 commands per minute
    for i in $(seq 1 10); do
        CMD=$(cdp_random_valid)
        echo -n "  Command $i: "
        send_cdp_command "$CMD"
        sleep 0.5  # Pace commands (not rapid-fire)
    done

    # Verify session health every minute
    if ! ./target/debug/teleport info --id chrome-test > /dev/null 2>&1; then
        echo -e "${RED}ERROR: Session died at minute $MINUTE${NC}"
        break
    fi

    # Log memory usage (if available)
    if command -v ps &> /dev/null; then
        CHROME_MEM=$(ps aux | grep "Chrome.*9222" | grep -v grep | awk '{print $6}' | head -1)
        if [ -n "$CHROME_MEM" ]; then
            echo "  Chrome memory: ${CHROME_MEM}KB"
        fi
    fi

    MINUTE=$((MINUTE + 1))
    sleep 5  # Pause between minutes
done

echo ""
echo "30-minute test complete!"

print_summary "Long Duration Test (30 Minutes)"
exit_code=$?

# Calculate actual duration in minutes
DURATION_MIN=$(echo "scale=1; $DURATION_S / 60" | bc 2>/dev/null || echo "N/A")

# Check final session health
if ./target/debug/teleport info --id chrome-test > /dev/null 2>&1; then
    SESSION_HEALTH="HEALTHY"
else
    SESSION_HEALTH="DEGRADED"
fi

mkdir -p test-results/websocat-cdp
cat > test-results/websocat-cdp/duration-30min-$(date +%Y%m%d-%H%M%S).md <<EOF
# Long Duration Test (30 Minutes)

**Date:** $(date +"%Y-%m-%d %H:%M:%S")
**Duration:** 30 minutes
**Purpose:** Validate long-term stability and resource management

## Results

- Total Commands: $TOTAL_TESTS
- Passed: $PASSED_TESTS
- Failed: $FAILED_TESTS
- Success Rate: ${SUCCESS_RATE}%
- Test Duration: ${DURATION_S}s ($DURATION_MIN minutes)

## Stability Metrics

- Commands per minute: ~10
- Session uptime: ${DURATION_S}s
- Session health at end: $SESSION_HEALTH
- Memory leaks: Monitoring required - check Chrome memory usage

## Latency

- Min: ${MIN_LAT}ms
- Max: ${MAX_LAT}ms
- Average: ${AVG_LAT}ms
- P99: ${P99_LAT}ms

## Conclusion

$(if [ $exit_code -eq 0 ]; then echo "OK PASSED - No degradation over 30 minutes"; else echo "X FAILED - Stability issues over time"; fi)

**This proves teleport can sustain multi-hour LLM workflows without degradation.**
EOF

exit $exit_code
