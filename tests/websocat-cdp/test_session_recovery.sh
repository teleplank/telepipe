#!/bin/bash
# Test 5: Session Recovery Test
# Kills and restarts session mid-test
# Validates recovery from catastrophic failures

set -e

source "$(dirname "$0")/test_common.sh"
source "$(dirname "$0")/cdp_commands.sh"
source "$(dirname "$0")/chaos_common.sh"

init_test "Session Recovery Test"

trap cleanup_test EXIT
setup_chrome_session || exit 1

echo "Testing session recovery after catastrophic failures..."
echo ""

# Phase 1: Normal operation
echo "Phase 1: Normal operation (10 commands)"
for i in $(seq 1 10); do
    CMD=$(cdp_random_valid)
    echo -n "Command $i: "
    send_cdp_command "$CMD"
done

echo ""
echo -e "${YELLOW}Phase 2: CATASTROPHIC FAILURE - Killing session${NC}"
chaos_kill_and_restart_session

echo ""
echo "Phase 3: Post-recovery validation (10 commands)"
for i in $(seq 11 20); do
    CMD=$(cdp_random_valid)
    echo -n "Command $i: "
    send_cdp_command "$CMD"
    sleep 0.1
done

echo ""
echo -e "${YELLOW}Phase 4: Second failure - Killing session again${NC}"
chaos_kill_and_restart_session

echo ""
echo "Phase 5: Final validation (10 commands)"
for i in $(seq 21 30); do
    CMD=$(cdp_random_valid)
    echo -n "Command $i: "
    send_cdp_command "$CMD"
    sleep 0.1
done

print_summary "Session Recovery Test"
exit_code=$?

mkdir -p test-results/websocat-cdp
cat > test-results/websocat-cdp/session-recovery-$(date +%Y%m%d-%H%M%S).md <<EOF
# Session Recovery Test Results

**Date:** $(date +"%Y-%m-%d %H:%M:%S")
**Pattern:** Normal -> Kill -> Restart -> Validate (repeated)
**Purpose:** Validate recovery from catastrophic failures

## Results

- Total Commands: $TOTAL_TESTS
- Passed: $PASSED_TESTS
- Failed: $FAILED_TESTS
- Success Rate: ${SUCCESS_RATE}%

## Recovery Events

- Phase 1: Normal (10 commands)
- **Phase 2: KILL SESSION**
- Phase 3: Recovery validation (10 commands)
- **Phase 4: KILL SESSION AGAIN**
- Phase 5: Final validation (10 commands)

## Latency

- Min: ${MIN_LAT}ms
- Max: ${MAX_LAT}ms
- Average: ${AVG_LAT}ms
- P99: ${P99_LAT}ms

## Conclusion

$(if [ $exit_code -eq 0 ]; then echo "OK PASSED - Full recovery after catastrophic failures"; else echo "X FAILED - Recovery issues detected"; fi)

This validates that teleport can recover from complete session failures,
proving robustness for production environments where failures are inevitable.
EOF

exit $exit_code
