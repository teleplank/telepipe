#!/bin/bash
# Test 7: Stress Test (100 commands)
# High volume test with mixed patterns
# Tests session stability under sustained load

set -e

source "$(dirname "$0")/test_common.sh"
source "$(dirname "$0")/cdp_commands.sh"

init_test "Stress Test (100 Commands Mixed Patterns)"

trap cleanup_test EXIT
setup_chrome_session || exit 1

echo "Sending 100 commands with mixed timing patterns..."
echo "This validates session stability under sustained load."
echo ""

for i in $(seq 1 100); do
    CMD=$(cdp_random_valid)

    # Vary timing: rapid fire for some, pauses for others
    if [ $((i % 10)) -eq 0 ]; then
        echo ""
        echo "=== Checkpoint $i/100 ==="
    fi

    echo -n "Command $i: "
    send_cdp_command "$CMD"

    # Variable delay pattern
    if [ $((i % 20)) -eq 0 ]; then
        # Longer pause every 20 commands
        sleep 0.5
    elif [ $((i % 5)) -eq 0 ]; then
        # Short pause every 5 commands
        sleep 0.1
    fi
    # Otherwise no delay (rapid fire)
done

echo ""
echo "Verifying session still active..."
if ./target/debug/teleport info --id chrome-test > /dev/null 2>&1; then
    echo -e "${GREEN}Session still active after 100 commands${NC}"
else
    echo -e "${RED}Session disappeared${NC}"
fi

print_summary "Stress Test (100 Commands)"
exit_code=$?

mkdir -p test-results/websocat-cdp
cat > test-results/websocat-cdp/stress-100-$(date +%Y%m%d-%H%M%S).md <<EOF
# Stress Test Results (100 Commands)

**Date:** $(date +"%Y-%m-%d %H:%M:%S")
**Pattern:** 100 commands with mixed timing (rapid-fire + pauses)
**Purpose:** Test session stability under sustained load

## Results

- Total Commands: $TOTAL_TESTS
- Passed: $PASSED_TESTS
- Failed: $FAILED_TESTS
- Success Rate: ${SUCCESS_RATE}%
- Duration: ${DURATION_S}s

## Load Pattern

- Rapid-fire commands: ~80% of commands
- 100ms pause every 5 commands
- 500ms pause every 20 commands
- Total volume: 100 commands

## Latency

- Min: ${MIN_LAT}ms
- Max: ${MAX_LAT}ms
- Average: ${AVG_LAT}ms
- P99: ${P99_LAT}ms

## Session Stability

Session remained active after all 100 commands: $(if ./target/debug/teleport info --id chrome-test > /dev/null 2>&1; then echo "YES"; else echo "NO"; fi)

## Conclusion

$(if [ $exit_code -eq 0 ]; then echo "PASSED - Session handles sustained load correctly"; else echo "FAILED - Stability issues under load detected"; fi)

This stress test validates that teleport can handle high-volume CDP
automation as would be required for extensive browser testing or complex
multi-step LLM workflows.
EOF

exit $exit_code
