#!/bin/bash
# Test 5: Command Variety
# Tests different CDP command types to ensure broad compatibility
# Validates response handling for various command structures

set -e

source "$(dirname "$0")/test_common.sh"
source "$(dirname "$0")/cdp_commands.sh"

init_test "Command Variety (Different CDP Methods)"

trap cleanup_test EXIT
setup_chrome_session || exit 1

echo "Testing variety of CDP command types..."
echo "This ensures teleport works with different CDP methods and payloads."
echo ""

# Test each command type
echo -n "Browser.getVersion: "
send_cdp_command "$(cdp_browser_version)"

echo -n "Target.getTargets: "
send_cdp_command "$(cdp_browser_targets)"

echo -n "Runtime.evaluate (simple): "
send_cdp_command "$(cdp_runtime_evaluate '1+1')"

echo -n "Runtime.evaluate (navigator): "
send_cdp_command "$(cdp_runtime_evaluate 'navigator.userAgent')"

echo -n "Network.enable: "
send_cdp_command "$(cdp_network_enable)"

echo -n "Network.disable: "
send_cdp_command "$(cdp_network_disable)"

# Repeat with mixed commands
echo ""
echo "Sending 20 mixed commands..."
for i in $(seq 1 20); do
    CMD=$(cdp_random_valid)
    echo -n "Command $i: "
    send_cdp_command "$CMD"
    sleep 0.1
done

print_summary "Command Variety Test"
exit_code=$?

mkdir -p test-results/websocat-cdp
cat > test-results/websocat-cdp/command-variety-$(date +%Y%m%d-%H%M%S).md <<EOF
# Command Variety Test Results

**Date:** $(date +"%Y-%m-%d %H:%M:%S")
**Pattern:** Different CDP methods (Browser, Runtime, Network, Target)
**Purpose:** Test compatibility with various CDP command types

## Results

- Total Commands: $TOTAL_TESTS
- Passed: $PASSED_TESTS
- Failed: $FAILED_TESTS
- Success Rate: ${SUCCESS_RATE}%
- Duration: ${DURATION_S}s

## Command Types Tested

- Browser.getVersion
- Target.getTargets
- Runtime.evaluate (multiple expressions)
- Network.enable/disable
- Mixed random commands

## Latency

- Min: ${MIN_LAT}ms
- Max: ${MAX_LAT}ms
- Average: ${AVG_LAT}ms
- P99: ${P99_LAT}ms

## Conclusion

$(if [ $exit_code -eq 0 ]; then echo "PASSED - All CDP command types work correctly"; else echo "FAILED - Some command types have issues"; fi)

This test validates that teleport correctly handles diverse CDP command
structures, ensuring compatibility with the full range of Chrome DevTools
Protocol methods that LLMs might use.
EOF

exit $exit_code
