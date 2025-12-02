#!/bin/bash
# Test 3: Mixed Timing Pattern
# Simulates unpredictable real-world patterns with random delays
# Tests robustness under variable timing conditions

set -e

source "$(dirname "$0")/test_common.sh"
source "$(dirname "$0")/cdp_commands.sh"

init_test "Mixed Timing (Random Delays 0-500ms)"

trap cleanup_test EXIT
setup_chrome_session || exit 1

echo "Sending 30 commands with random delays (0-500ms)..."
echo "This simulates unpredictable real-world LLM behavior patterns."
echo ""

for i in $(seq 1 30); do
    CMD=$(cdp_random_valid)
    DELAY=$((RANDOM % 500))

    echo -n "Command $i (delay: ${DELAY}ms): "
    send_cdp_command "$CMD"

    # Random delay in milliseconds
    sleep $(echo "scale=3; $DELAY / 1000" | bc)
done

print_summary "Mixed Timing Test"
exit_code=$?

mkdir -p test-results/websocat-cdp
cat > test-results/websocat-cdp/mixed-timing-$(date +%Y%m%d-%H%M%S).md <<EOF
# Mixed Timing Test Results

**Date:** $(date +"%Y-%m-%d %H:%M:%S")
**Pattern:** Random delays between 0-500ms
**Purpose:** Test robustness under unpredictable timing

## Results

- Total Commands: $TOTAL_TESTS
- Passed: $PASSED_TESTS
- Failed: $FAILED_TESTS
- Success Rate: ${SUCCESS_RATE}%
- Duration: ${DURATION_S}s

## Latency

- Min: ${MIN_LAT}ms
- Max: ${MAX_LAT}ms
- Average: ${AVG_LAT}ms
- P99: ${P99_LAT}ms

## Conclusion

$(if [ $exit_code -eq 0 ]; then echo "PASSED - Session handles variable timing correctly"; else echo "FAILED - Timing sensitivity issues detected"; fi)

This test validates that teleport works correctly regardless of timing
variability, ensuring robustness in real-world scenarios where LLM response
times and command intervals are unpredictable.
EOF

exit $exit_code
