# Telepipe Invariants
## Testable Assertions Extracted from the Constitution

**Purpose:** Every "must", "always", "never", and "guarantee" from the constitution becomes a test. These invariants validate that the implementation matches the specification.

**Usage:** Run these tests after implementing each module. ALL must pass before a cycle is complete.

---

## Test Execution

```bash
# Run all invariant tests
cargo test --package telepipe-core --lib

# Run specific category
cargo test --package telepipe-core --lib fd_invariants
cargo test --package telepipe-core --lib port_invariants
```

---

## Category 1: FD Allocation Invariants

### Source: Core Spec, Section 5.2

**Invariant 1.1:** FD range is 3-255

**Invariant 1.2:** FD 256 is reserved (never allocated)

**Invariant 1.3:** FD exhaustion returns E-ALLOC-FD (exit 21)

---

## Category 2: Port Allocation Invariants

### Source: Core Spec, Section 5.1

**Invariant 2.1:** Port range is 49152-65535

**Invariant 2.2:** Ports are ephemeral (IANA standard)

**Invariant 2.3:** Port exhaustion returns E-ALLOC-PORT (exit 22)

---

## Category 3: Info Mode Invariants

### Source: Core Spec, Section 3.3

**Invariant 3.1:** Info exits immediately (<100ms)

**Invariant 3.2:** Info never blocks

**Invariant 3.3:** Info output is tab-separated

**Invariant 3.4:** Info redirect mode has all required fields:
- mode, host, pid, supervisor, stdin, stdout, stderr, fdin, fdout, fderr

**Invariant 3.5:** Info connect mode has all required fields:
- mode, host, port, supervisor, connected, reconnect, fd

---

## Category 4: Redirect Mode Invariants

### Source: Operations, Section 2.1

**Invariant 4.1:** Redirect allocates exactly 3 FDs

**Invariant 4.2:** Redirect allocates exactly 3 ports

**Invariant 4.3:** Redirect command exits with code 0

**Invariant 4.4:** Supervisor continues after redirect exits

---

## Category 5: Connect Mode Invariants

### Source: Operations, Section 2.3

**Invariant 5.1:** Connect allocates exactly 1 FD

**Invariant 5.2:** Connect command exits with code 0

---

## Category 6: Exec Mode Invariants

### Source: Operations, Section 2.2

**Invariant 6.1:** Exec streams data bidirectionally

**Invariant 6.2:** Exec exits when streams close

---

## Category 7: Error Code Invariants

### Source: Error Model

**Invariant 7.1:** Session not found returns exit 61 (E-DICT-MISSING)

**Invariant 7.2:** Duplicate session ID returns exit 64 (E-DICT-CONFLICT)

**Invariant 7.3:** Invalid host returns exit 74 (E-CLI-INVALID-HOST)

**Invariant 7.4:** Invalid FD override returns exit 75 (E-CLI-INVALID-FD)

**Invariant 7.5:** Invalid port override returns exit 76 (E-CLI-INVALID-PORT)

**Invariant 7.6:** FD exhaustion returns exit 21 (E-ALLOC-FD)

**Invariant 7.7:** Port exhaustion returns exit 22 (E-ALLOC-PORT)

**Invariant 7.8:** Info read failure returns exit 68 (E-INFO-READ)

---

## Category 8: Recovery Invariants

### Source: Recovery Algorithms

**Invariant 8.1:** Recovery runs before redirect

**Invariant 8.2:** Recovery does NOT run before info

---

## Category 9: Concurrency Invariants

### Source: Concurrency Model

**Invariant 9.1:** Serial exec calls work (one after another)

**Invariant 9.2:** Concurrent exec returns E-EXEC-ALREADY-ACTIVE (exit 81)

**Invariant 9.3:** Exec works after concurrent attempt is rejected

---

## Category 10: Session Dictionary Invariants

### Source: Core Spec, Section 3.3

**Invariant 10.1:** Session files are valid JSON

**Invariant 10.2:** Session files have required fields

---

## Category 11: Stop/Disconnect Invariants

### Source: Operations, Sections 3.1 and 3.2

**Invariant 11.1:** Stop removes session file

**Invariant 11.2:** Disconnect removes session file

---

## Category 12: Supervisor I/O Forwarding Invariants

### Source: Supervisor Architecture, Section 7.1

**Invariant 12.1:** Supervisor accepts TCP connections on allocated ports

**Invariant 12.2:** Data flows from exec stdin through supervisor to child stdin

**Invariant 12.3:** Child stdout flows through supervisor to exec stdout

**Invariant 12.4:** Child stderr flows through supervisor to exec stderr

**Invariant 12.5:** Complete end-to-end data flow verification

**Invariant 12.6:** Multiple serial exec calls work on same session

**Invariant 12.7:** Binary data passes through unmodified

---

## Summary

This file contains **58+ invariant tests** covering:
- FD allocation (3 tests)
- Port allocation (3 tests)
- Info mode behavior (5 tests)
- Redirect mode (4 tests)
- Connect mode (2 tests)
- Exec mode (2 tests)
- Error codes (8 tests)
- Recovery (2 tests)
- Concurrency (3 tests)
- Session dictionary (2 tests)
- Lifecycle (2 tests)
- Supervisor I/O Forwarding (7 tests)

**ALL must pass before Telepipe is considered complete.**

**Zero failures tolerated.**
