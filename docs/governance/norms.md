# Telepipe Agent Norms
## Behavioral Rules and Code Quality Standards

**Purpose:** This document defines HOW the agent should behave when building Telepipe. These are the "soft laws" - not about what to build (that's the Constitution) but about how to work, when to stop, and what quality means.

---

## Code Quality Standards

### Rule 1: Functions Under 50 Lines
Every function should be **under 50 lines** of code (not counting blank lines or comments).

### Rule 2: Comments Explain WHY, Not WHAT
Code should be self-explanatory. Comments explain **rationale**, not **mechanics**.

### Rule 3: No Unsafe Blocks Without Safety Proof
Avoid `unsafe` code unless absolutely necessary. If required, add detailed safety comments.

### Rule 4: Prefer Clarity Over Cleverness
Write code that is **obvious**, not **clever**.

### Rule 5: Error Messages Match Spec
Error messages MUST match the exact format from the error model.

### Rule 6: Consistent Naming
Follow Rust naming conventions and match terminology from the constitution:
- Use "session" not "context" or "instance"
- Use "redirect" not "spawn" or "launch"
- Use "supervisor" not "daemon" or "watcher"
- Use "info" not "status" or "query"

---

## Testing Requirements

### Rule 7: Test After Every Module
After implementing a module, MUST:
1. Run `cargo check --workspace`
2. Run `cargo build --workspace`
3. Run relevant tests from INVARIANTS
4. Document results in cycle report

### Rule 8: Tests Must Be Deterministic
Tests should:
- Produce same results every run
- Not depend on timing (except timeouts)
- Not depend on external services
- Clean up after themselves

### Rule 9: Don't Change Spec to Fix Tests
If a test fails:
- **First:** Check if your code is wrong
- **Second:** Check if your test is wrong
- **Last:** Check if the spec is ambiguous

---

## When to STOP and Ask

### Rule 10: Stop on Ambiguity
The agent MUST stop immediately if:
1. **Spec has 2+ valid interpretations**
2. **Spec conflicts with itself**
3. **Spec is silent on required behavior**
4. **Same test fails 3 times with different fixes**

**Don't guess. Don't assume. Stop and ask.**

### Rule 11: Stop on Circular Dependencies
If you discover a circular dependency, document it, propose refactoring, and wait for human approval.

### Rule 12: Stop on Impossible Requirements
If the spec requires something that seems impossible, document why and stop.

---

## Error Handling Norms

### Rule 13: Use Result Types
Every function that can fail MUST return `Result<T, TelepipeError>`.

**Never:**
- Use `.unwrap()` in production code
- Use `.expect()` without good reason
- Panic outside of main
- Silently ignore errors

### Rule 14: Map Errors to Exit Codes
Every error must map to an exit code from the error model.

### Rule 15: Error Messages to Stderr Only
ALL errors go to stderr in the exact format specified.

---

## Iteration and Retry Norms

### Rule 16: Three-Attempt Limit
If a test fails, the agent may retry up to **3 times**.
After 3 failures: STOP and report.

### Rule 17: Document Failed Attempts
In the cycle report, document each attempt with issue, fix tried, and result.

### Rule 18: Learn from Previous Cycles
Read previous cycle reports and don't repeat the same mistake twice.

---

## Philosophy and Mindset

### Rule 25: You Are a Compiler, Not an Assistant
The agent's role is to:
- Ingest specs
- Transform them into code
- Emit artifacts
- Report objectively

**Not to:**
- Chat about the weather
- Apologize for failures
- Seek approval for every decision
- Add "helpful" features

### Rule 26: The Spec is Always Right
If the spec says it, implement it exactly.

**Exception:** If spec is ambiguous or wrong, STOP and report.

### Rule 27: Determinism is Sacred
Same inputs MUST produce same outputs.

### Rule 28: Zero Drift is Non-Negotiable
Telepipe has exactly 6 operations: redirect, exec, connect, info, stop, disconnect.

**Never:**
- Add a 7th operation
- Rename any operation
- Combine operations
- Create "shortcuts" or "aliases"

---

## Summary

These norms ensure:
- **Quality code** - Clear, maintainable, correct
- **Deterministic builds** - Same input → same output
- **Safe iteration** - Stop before making mistakes
- **Clear audit trail** - Reports document everything
- **Zero drift** - Spec compliance guaranteed

**Follow these norms, and Telepipe will be built correctly.**
