# Telepipe Judicial Branch
## Testing Standards and Validation Rules

**Purpose:** This document defines the testing methodology, golden snapshot system, and deterministic validation rules for Telepipe. It ensures tests are reliable, reproducible, and cross-platform.

**Relationship:** Complements INVARIANTS (what to test) by defining HOW to test.

---

## Testing Philosophy

### Core Principles

1. **Deterministic** - Same input → Same output, every time
2. **Fast** - Tests complete quickly (<30 seconds total)
3. **Isolated** - Tests don't interfere with each other
4. **Cross-platform** - Work on macOS, Linux, Windows/Git Bash
5. **Self-contained** - No external dependencies (except optional)

---

## Test Categories

### Level 1: Invariant Tests
**What:** Unit tests for individual functions and modules
**When:** After implementing each module
**Runtime:** <1 second per test
**Framework:** Rust `#[test]` functions

### Level 2: Integration Tests
**What:** Tests of complete operations (redirect→exec→stop)
**When:** After completing related modules
**Runtime:** <5 seconds per test
**Framework:** Rust integration tests + shell scripts

### Level 3: Golden Tests
**What:** External validation using shell scripts
**When:** Before releases and major milestones
**Runtime:** <30 seconds total
**Framework:** Bash scripts with byte-perfect comparisons

---

## Output Standards

### Stdout Contract
**Rule:** Stdout contains ONLY process output, never Telepipe metadata.

### Stderr Contract
**Rule:** Stderr contains ONLY Telepipe diagnostics, in exact format:
```
ERROR E-<CODE>: <message>
```

**Format Rules:**
- Start with `ERROR `
- Followed by error code
- One line per message
- No timestamps
- No ANSI codes
- Terminate with `\n`

---

## Supervisor Data Flow Verification

### Required Verification Pattern

All redirect+exec tests MUST verify actual data flow through the supervisor:

```rust
// 1. Create redirect session
telepipe redirect --id test-X -- cat

// 2. Send SPECIFIC known input via exec
echo "test input" | telepipe exec --id test-X

// 3. Verify EXACT output matches expectation
// assert_eq!(output, "test input\n")  // NOT assert!(!output.is_empty())

// 4. Cleanup
telepipe stop --id test-X
```

### What This Pattern Proves

When a test sends "hello\n" and receives "hello\n" back from `cat`:

1. **exec connected** to TCP stdin port
2. **Supervisor accepted** the TCP connection
3. **Supervisor forwarded** data to child's stdin pipe
4. **Child (cat) read** from its stdin
5. **Child wrote** to its stdout
6. **Supervisor read** from child's stdout pipe
7. **Supervisor forwarded** to TCP stdout port
8. **exec received** the data

This is the complete data path per the Supervisor Architecture.

### Forbidden Patterns

**NEVER use these weak assertions:**
```rust
// ❌ BAD - Only checks command didn't crash
assert_eq!(result.exit_code, 0);

// ❌ BAD - Only checks output exists, not correctness
assert!(!output.stdout.is_empty());

// ❌ BAD - Partial match doesn't prove complete flow
assert!(output.stdout.contains("test"));
```

**ALWAYS use exact match assertions:**
```rust
// ✅ GOOD - Proves complete data flow
assert_eq!(output.stdout, expected_output);

// ✅ GOOD - Verifies exact bytes for binary transparency
assert_eq!(output.stdout_bytes, input_bytes);
```

---

## Test Quality Checklist

Before considering a test "complete":

- [ ] Test is deterministic (same result every run)
- [ ] Test cleans up after itself
- [ ] Test uses unique session ID
- [ ] Test validates exit code
- [ ] Test validates stdout format (if applicable)
- [ ] Test validates stderr format (if applicable)
- [ ] Test completes in <5 seconds
- [ ] Test works on all supported platforms
- [ ] Test has clear assertion messages
- [ ] Test is documented (what it validates)

---

## Summary

The Judicial Branch ensures:
- **Deterministic testing** - Reliable, reproducible results
- **Clear standards** - Exact format requirements
- **Cross-platform** - Works everywhere
- **Fast feedback** - Tests complete quickly
- **Comprehensive** - All code paths tested
- **Data flow verification** - Supervisor I/O forwarding proven

**Follow these standards, and tests will catch bugs before they ship.**
