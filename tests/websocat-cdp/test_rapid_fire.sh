#!/bin/bash
# Test 1: Rapid-Fire Pattern
# Simulates LLM firing commands with no delays (e.g., Claude Code, API usage)
# Tests for race conditions and connection handling under rapid requests

set -e

# Source common infrastructure
source "$(dirname "$0")/test_common.sh"
source "$(dirname "$0")/cdp_commands.sh"

# Initialize
init_test "Rapid-Fire (LLM Speed - No Delays)"

# Setup
trap cleanup_test EXIT
setup_chrome_session || exit 1

echo "Sending 50 commands with NO delays between them..."
echo "This simulates an LLM executing commands at maximum speed."
echo ""

# Send 50 commands with no delays
for i in $(seq 1 50); do
    CMD=$(cdp_random_valid)
    echo -n "Command $i: "
    send_cdp_command "$CMD"
    # NO SLEEP - Fire immediately!
done

# Summary
print_summary "Rapid-Fire Test"
exit_code=$?

# Save results
mkdir -p test-results/websocat-cdp
cat > test-results/websocat-cdp/rapid-fire-$(date +%Y%m%d-%H%M%S).md <<EOF
# Rapid-Fire Test Results

**Date:** $(date +"%Y-%m-%d %H:%M:%S")
**Pattern:** No delays between commands (LLM maximum speed)
**Purpose:** Test race conditions and rapid connection handling

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

$(if [ $exit_code -eq 0 ]; then echo "PASSED - Session handles rapid-fire commands correctly"; else echo "FAILED - Race conditions or connection issues detected"; fi)

This test validates that teleport can handle LLM tools firing CDP commands
at maximum speed without delays, as would happen with Claude Code or API-driven
automation.
EOF

exit $exit_code
