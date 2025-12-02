#!/bin/bash
# Test 4: Slow Deliberate Pattern
# Simulates human-like or deliberative LLM with longer pauses
# Tests session persistence over longer idle periods

set -e

source "$(dirname "$0")/test_common.sh"
source "$(dirname "$0")/cdp_commands.sh"

init_test "Slow Deliberate (Human-Like Pauses)"

trap cleanup_test EXIT
setup_chrome_session || exit 1

echo "Sending 20 commands with 500ms delays..."
echo "This simulates deliberative LLM or human-interactive usage."
echo ""

for i in $(seq 1 20); do
    CMD=$(cdp_random_valid)
    echo -n "Command $i: "
    send_cdp_command "$CMD"

    # Deliberate pause
    if [ $i -lt 20 ]; then
        sleep 0.5
    fi
done

print_summary "Slow Deliberate Test"
exit_code=$?

mkdir -p test-results/websocat-cdp
cat > test-results/websocat-cdp/slow-deliberate-$(date +%Y%m%d-%H%M%S).md <<EOF
# Slow Deliberate Test Results

**Date:** $(date +"%Y-%m-%d %H:%M:%S")
**Pattern:** 500ms delays between commands
**Purpose:** Test session persistence with longer idle periods

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

$(if [ $exit_code -eq 0 ]; then echo "PASSED - Session persists correctly with deliberate pauses"; else echo "FAILED - Session persistence issues detected"; fi)

This test validates that teleport maintains session stability even with
longer pauses between commands, as would happen in interactive LLM sessions
or human-guided automation.
EOF

exit $exit_code
