#!/bin/bash
# Master runner for Level 3: Chaos + Concurrency Suite
#
# NOTE: Most Level 3 tests were removed in 1.0.0 because they had incorrect
# expectations (expected multiple concurrent execs to succeed, but per 550
# Section 7, the correct model is one-at-a-time with exit 81 for rejections).
#
# Remaining tests:
# - test_session_recovery.sh - Validates recovery from catastrophic failures
#
# Concurrent validation is done via:
# - test_concurrent_quick.sh - Serial + concurrent rejection
# - test_concurrent_quick2.sh - Concurrent with latency measurement
#
# Level 3 chaos concurrent tests will be rewritten in 1.0.1 with correct
# expectations (1 success, N-1 rejections per round).

set -e

SCRIPT_DIR="$(dirname "$0")"

echo "+================================================================+"
echo "|                                                                |"
echo "|    Level 3: CHAOS + RECOVERY TEST SUITE                       |"
echo "|    (Reduced for 1.0.0 - see header comments)                  |"
echo "|                                                                |"
echo "+================================================================+"
echo ""
echo "This suite tests:"
echo "  - Session recovery after catastrophic failures"
echo ""
echo "For concurrent validation, run from project root:"
echo "  - ./test_concurrent_quick.sh"
echo "  - ./test_concurrent_quick2.sh"
echo ""
read -p "Press Enter to begin Level 3 testing..."

# Results tracking
TOTAL_SUITES=1
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
        SUITE_RESULTS+=("X $name: FAILED")
        echo "Result: FAILED"
    fi

    echo ""
    sleep 3
}

# Run remaining tests
echo "======================================="
echo "CHAOS ENGINEERING TESTS"
echo "======================================="
run_suite "Test 1: Session Recovery" "test_session_recovery.sh"

# Final summary
echo ""
echo "+================================================================+"
echo "|                                                                |"
echo "|         LEVEL 3 CHAOS + RECOVERY FINAL SUMMARY                |"
echo "|         (Reduced for 1.0.0)                                   |"
echo "|                                                                |"
echo "+================================================================+"
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

# Generate report
REPORT_FILE="test-results/websocat-cdp/CHAOS_RECOVERY_SUMMARY-$(date +%Y-%m-%d).md"

mkdir -p test-results/websocat-cdp
cat > "$REPORT_FILE" <<EOF
# Level 3: Chaos + Recovery Test Suite Summary (1.0.0)

**Date:** $(date +"%Y-%m-%d %H:%M:%S")
**Purpose:** Session recovery validation

---

## Note on 1.0.0 Test Suite

Most Level 3 concurrent tests were removed in 1.0.0 because they had incorrect
expectations. They expected multiple concurrent execs to succeed, but per 550
Section 7, the correct model is one-at-a-time with exit 81 for rejections.

**For concurrent validation, run separately:**
- test_concurrent_quick.sh - Serial + concurrent rejection
- test_concurrent_quick2.sh - Concurrent with latency measurement

Level 3 chaos concurrent tests will be rewritten in 1.0.1 with correct
expectations (1 success, N-1 rejections per round).

---

## Overall Results

- Total Test Suites: $TOTAL_SUITES
- Passed: $PASSED_SUITES
- Failed: $FAILED_SUITES
- Overall Success Rate: $(( PASSED_SUITES * 100 / TOTAL_SUITES ))%

---

## Test Results

$(for result in "${SUITE_RESULTS[@]}"; do echo "- $result"; done)

---

## Test Coverage for 1.0.0

**Level 1:** Constitutional Validation (INVARIANTS.md - cargo test)
**Level 2:** Production Validation (~300 commands)
**Level 3:** Session Recovery (30 commands with kill/restart cycles)

**Concurrent Validation:**
- test_concurrent_quick.sh - Serial + concurrent rejection
- test_concurrent_quick2.sh - Concurrent with latency measurement

---

## Production Deployment Confidence

$(if [ $FAILED_SUITES -eq 0 ]; then
echo "Session recovery validated - teleport recovers from catastrophic failures."
else
echo "Session recovery test failed - investigate before production deployment."
fi)
EOF

echo "Report saved to: $REPORT_FILE"
echo ""

if [ $FAILED_SUITES -eq 0 ]; then
    echo "*** LEVEL 3 TESTS PASSED ***"
    echo ""
    echo "Session recovery validation complete."
    echo ""
    echo "For full concurrent validation, also run:"
    echo "  ./test_concurrent_quick.sh"
    echo "  ./test_concurrent_quick2.sh"
    exit 0
else
    echo "*** SOME TESTS FAILED ***"
    echo ""
    echo "Success rate: $(( PASSED_SUITES * 100 / TOTAL_SUITES ))%"
    echo ""
    echo "Review results before production deployment."
    exit 1
fi
