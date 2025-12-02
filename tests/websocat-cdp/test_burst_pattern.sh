#!/bin/bash
# Test 2: Burst Pattern
# Simulates realistic agentic workflow: burst of actions, pause to think, repeat
# Tests connection reuse and session persistence across thinking pauses

set -e

source "$(dirname "$0")/test_common.sh"
source "$(dirname "$0")/cdp_commands.sh"

init_test "Burst Pattern (Realistic Agentic Workflow)"

trap cleanup_test EXIT
setup_chrome_session || exit 1

echo "Sending commands in burst pattern: 5 rapid, pause 1s, repeat..."
echo "This simulates an LLM executing actions, then thinking, then acting again."
echo ""

BURST_SIZE=5
NUM_BURSTS=10

for burst in $(seq 1 $NUM_BURSTS); do
    echo "Burst $burst/$NUM_BURSTS:"

    for i in $(seq 1 $BURST_SIZE); do
        CMD=$(cdp_random_valid)
        echo -n "  Command $i: "
        send_cdp_command "$CMD"
    done

    # Pause between bursts (simulating LLM "thinking")
    if [ $burst -lt $NUM_BURSTS ]; then
        echo "  [Thinking pause: 1s]"
        sleep 1
    fi
    echo ""
done

print_summary "Burst Pattern Test"
exit_code=$?

mkdir -p test-results/websocat-cdp
cat > test-results/websocat-cdp/burst-pattern-$(date +%Y%m%d-%H%M%S).md <<EOF
# Burst Pattern Test Results

**Date:** $(date +"%Y-%m-%d %H:%M:%S")
**Pattern:** 5 commands rapid-fire, 1s pause, repeat (10 bursts)
**Purpose:** Test realistic agentic workflow with thinking pauses

## Results

- Total Commands: $TOTAL_TESTS
- Passed: $PASSED_TESTS
- Failed: $FAILED_TESTS
- Success Rate: ${SUCCESS_RATE}%
- Duration: ${DURATION_S}s
- Bursts: $NUM_BURSTS
- Commands per burst: $BURST_SIZE

## Latency

- Min: ${MIN_LAT}ms
- Max: ${MAX_LAT}ms
- Average: ${AVG_LAT}ms
- P99: ${P99_LAT}ms

## Conclusion

$(if [ $exit_code -eq 0 ]; then echo "PASSED - Session handles burst patterns correctly"; else echo "FAILED - Connection reuse issues detected"; fi)

This test validates that teleport maintains session state across LLM thinking
pauses, as would happen in realistic agentic workflows where the LLM alternates
between executing actions and planning next steps.
EOF

exit $exit_code
