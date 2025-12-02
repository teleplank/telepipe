#!/bin/bash
# Test 6: Error Recovery
# Injects invalid commands and verifies session continues working
# Tests robustness and error handling

set -e

source "$(dirname "$0")/test_common.sh"
source "$(dirname "$0")/cdp_commands.sh"

init_test "Error Recovery (Failure Injection)"

trap cleanup_test EXIT
setup_chrome_session || exit 1

echo "Testing error recovery by injecting invalid commands..."
echo "This validates that session remains stable after errors."
echo ""

# Pattern: valid, invalid, valid, invalid JSON, valid

echo "Phase 1: Valid command"
echo -n "Command 1 (valid): "
send_cdp_command "$(cdp_browser_version)"

echo ""
echo "Phase 2: Invalid method (should return CDP error)"
echo -n "Command 2 (invalid method): "
send_cdp_command "$(cdp_invalid_method)" "false"

echo ""
echo "Phase 3: Recovery - valid command after error"
echo -n "Command 3 (valid): "
send_cdp_command "$(cdp_browser_version)"

echo ""
echo "Phase 4: Completely invalid JSON"
echo -n "Command 4 (invalid JSON): "
send_cdp_command "$(cdp_invalid_json)" "false"

echo ""
echo "Phase 5: Recovery - valid command after malformed JSON"
echo -n "Command 5 (valid): "
send_cdp_command "$(cdp_browser_version)"

echo ""
echo "Phase 6: Series of valid commands to confirm stability"
for i in $(seq 1 10); do
    echo -n "Command $((5 + i)) (valid): "
    send_cdp_command "$(cdp_random_valid)"
done

echo ""
echo "Phase 7: Another error followed by recovery"
echo -n "Command 16 (invalid method): "
send_cdp_command "$(cdp_invalid_method)" "false"

for i in $(seq 1 5); do
    echo -n "Command $((16 + i)) (valid): "
    send_cdp_command "$(cdp_random_valid)"
done

print_summary "Error Recovery Test"
exit_code=$?

mkdir -p test-results/websocat-cdp
cat > test-results/websocat-cdp/error-recovery-$(date +%Y%m%d-%H%M%S).md <<EOF
# Error Recovery Test Results

**Date:** $(date +"%Y-%m-%d %H:%M:%S")
**Pattern:** Valid -> Invalid -> Valid (repeated)
**Purpose:** Test session stability after errors

## Results

- Total Commands: $TOTAL_TESTS
- Passed: $PASSED_TESTS
- Failed: $FAILED_TESTS
- Success Rate: ${SUCCESS_RATE}%
- Duration: ${DURATION_S}s

## Test Phases

1. Valid command (baseline)
2. Invalid CDP method (inject error)
3. Valid command (verify recovery)
4. Malformed JSON (inject error)
5. Valid command (verify recovery)
6. 10 valid commands (confirm stability)
7. Invalid method + 5 valid (final verification)

## Latency

- Min: ${MIN_LAT}ms
- Max: ${MAX_LAT}ms
- Average: ${AVG_LAT}ms
- P99: ${P99_LAT}ms

## Conclusion

$(if [ $exit_code -eq 0 ]; then echo "PASSED - Session handles errors gracefully and recovers"; else echo "FAILED - Error handling issues or session instability detected"; fi)

This test validates that teleport sessions remain stable even when
LLMs send invalid commands, ensuring robustness in production use where
errors are inevitable.
EOF

exit $exit_code
