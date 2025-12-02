#!/bin/bash
# Master test runner for standard websocat+CDP test suite

set -e

SCRIPT_DIR="$(dirname "$0")"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

echo "================================================================"
echo ""
echo "    Teleport Standard Websocat+CDP Test Suite"
echo ""
echo "================================================================"
echo ""

# Results tracking
TOTAL_SUITES=7
PASSED_SUITES=0
FAILED_SUITES=0

declare -a SUITE_RESULTS

# Run each test suite
run_suite() {
    local name="$1"
    local script="$2"

    echo ""
    echo "----------------------------------------------------------------"
    echo "Running: $name"
    echo "----------------------------------------------------------------"

    if bash "$SCRIPT_DIR/$script"; then
        PASSED_SUITES=$((PASSED_SUITES + 1))
        SUITE_RESULTS+=("OK $name: PASSED")
        echo "Result: PASSED"
    else
        FAILED_SUITES=$((FAILED_SUITES + 1))
        SUITE_RESULTS+=("XX $name: FAILED")
        echo "Result: FAILED"
    fi

    echo ""
    sleep 2  # Pause between suites
}

# Run all tests
run_suite "Test 1: Rapid-Fire" "test_rapid_fire.sh"
run_suite "Test 2: Burst Pattern" "test_burst_pattern.sh"
run_suite "Test 3: Mixed Timing" "test_mixed_timing.sh"
run_suite "Test 4: Slow Deliberate" "test_slow_deliberate.sh"
run_suite "Test 5: Command Variety" "test_command_variety.sh"
run_suite "Test 6: Error Recovery" "test_error_recovery.sh"
run_suite "Test 7: Stress Test (100)" "test_stress_100.sh"

# Final summary
echo ""
echo "================================================================"
echo ""
echo "                    FINAL SUMMARY"
echo ""
echo "================================================================"
echo ""

for result in "${SUITE_RESULTS[@]}"; do
    echo "$result"
done

echo ""
echo "Total Suites: $TOTAL_SUITES"
echo "Passed: $PASSED_SUITES"
echo "Failed: $FAILED_SUITES"
echo "Success Rate: $(( PASSED_SUITES * 100 / TOTAL_SUITES ))%"
echo ""

if [ $FAILED_SUITES -eq 0 ]; then
    echo "ALL SUITES PASSED"
    echo ""
    echo "The multiple exec fix is fully validated for production use!"
    exit 0
else
    echo "SOME SUITES FAILED"
    echo ""
    echo "Review individual test results in test-results/websocat-cdp/"
    exit 1
fi
