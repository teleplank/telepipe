# Telepipe Legislative Branch
## Amendment Protocol and Evolutionary Governance

**Purpose:** This document defines how Telepipe evolves when ambiguities or conflicts are discovered. It establishes the **Test-Driven Amendment Loop (TDAL)** and the process for safely updating the constitution.

---

## Governance Hierarchy

1. **CONSTITUTION** (10 spec documents) - What Telepipe is and does
2. **EXECUTIVE** - How the agent behaves
3. **BUILD_DIRECTIVES** - Current implementation strategy
4. **INVARIANTS** - Testable assertions
5. **NORMS** - Code quality and behavior rules
6. **LEGISLATIVE** (this document) - How to evolve the system

**Key Principle:** The Constitution is **immutable during build cycles**. Changes require formal amendments.

---

## The Test-Driven Amendment Loop (TDAL)

### Overview

When a test fails or ambiguity is found, the agent classifies the issue and either:
- **Fix code** (implementation bug)
- **Fix test** (test bug)
- **STOP and propose amendment** (spec ambiguity)

This loop ensures:
1. The spec stays authoritative
2. Changes are deliberate and reviewed
3. No drift occurs
4. Progress is safe and auditable

### TDAL Cycle Diagram

```
┌─── Build Cycle N ────────────────────────┐
│                                          │
│ 1. Read Constitution + Directives        │
│ 2. Implement Module                      │
│ 3. Run Tests                             │
│ 4. Classify Results                      │
│                                          │
│    ┌── PASS ──────────┐                 │
│    │  Mark complete   │                 │
│    │  Move to next    │                 │
│    └──────────────────┘                 │
│                                          │
│    ┌── FAIL ──────────────────────┐     │
│    │ A. Code Bug                  │     │
│    │    → Fix code (max 3 tries)  │     │
│    │                              │     │
│    │ B. Test Bug                  │     │
│    │    → Fix test                │     │
│    │                              │     │
│    │ C. Spec Ambiguity            │     │
│    │    → STOP                    │     │
│    │    → Generate Amendment      │     │
│    │    → Wait for Human          │     │
│    └──────────────────────────────┘     │
│                                          │
└──────────────────────────────────────────┘
                  ↓
         [Human Reviews Amendment]
                  ↓
         [Constitution Updated]
                  ↓
         [Resume Build Cycle]
```

---

## Amendment Classification Rules

When a test fails or code won't compile, classify into one of three categories:

### Category A: Code Bug (Implementation Error)

**Symptoms:**
- Spec is clear and unambiguous
- Test correctly reflects spec
- Code does not match spec

**Action:**
1. Fix the code
2. Retry test
3. If still fails after 3 attempts → reclassify or STOP

### Category B: Test Bug (Test Error)

**Symptoms:**
- Spec is clear and unambiguous
- Code correctly implements spec
- Test does not match spec

**Action:**
1. Fix the test
2. Re-run test
3. Document why test was wrong

### Category C: Spec Ambiguity (Constitution Issue)

**Symptoms:**
- Multiple valid interpretations exist
- Spec conflicts with itself
- Spec is silent on required behavior
- Unclear what "correct" means

**Action:**
1. **STOP immediately**
2. Generate amendment proposal
3. Wait for human to update constitution
4. Resume after amendment is applied

---

## Amendment Types

### Type 1: Clarification (No Behavior Change)

Making existing intent explicit without changing behavior.

**Example:**
```
BEFORE: "Info mode should be fast"
AFTER: "Info mode must complete in <100ms"
```

**Authority:** Human can approve quickly
**Risk:** Low

### Type 2: Specification Gap (Adding Missing Info)

Specifying behavior that was previously undefined.

**Example:**
```
BEFORE: [Silent on what happens if session file is corrupt]
AFTER: "If session file is corrupt, return E-DICT-CORRUPT (exit 62)"
```

**Authority:** Human approval required
**Risk:** Medium - might affect existing assumptions

### Type 3: Breaking Change (Modifying Existing Behavior)

Changing how something works.

**Authority:** Requires careful review and justification
**Risk:** High - breaks existing implementations

**Note:** Avoid Type 3 amendments if possible.

---

## Summary

The Legislative Branch ensures:
1. **Safe evolution** - Changes are deliberate, not accidental
2. **Clear authority** - Human approves all spec changes
3. **Audit trail** - Every change is documented
4. **Zero drift** - Agent can't modify spec on its own
5. **Forward progress** - Ambiguities don't block forever

**Key Principle:** When in doubt, STOP and ask. Never guess at spec intent.
