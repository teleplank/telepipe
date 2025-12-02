# The Memory Buffer Pattern
## Cross-Cycle Memory for AI Agents

A deterministic pattern for preserving insights across sessions without breaking governance.

---

## The Problem

LLM-driven development has a fundamental limitation:

**Each session starts fresh. Previous insights are lost.**

Without a memory mechanism:
- Every prompt is isolated
- Previous diagnostics are forgotten
- Later sessions must rediscover prior reasoning
- Multi-step repairs lose context and drift

---

## The Solution: Architect Memory Buffer (AMB)

The AMB is a **controlled, constitutional, deterministic memory surface**.

It formally captures the insights of one cycle and makes them explicitly available for the next — without rewriting history, mutating prior files, or breaking determinism.

```
Cycle N produces AMB.
Cycle N+1 reads AMB → patches governance.
Cycle N+2 reads AMB + governance → patches runtime.
Cycle N+3 reads AMB + governance + runtime → patches tests.
```

This creates a **constitutional amendment loop with memory**.

---

## How It Works

### What AMB Contains

AMB holds **four classes of insight**:

**1. Runtime Symptoms**
Captured observations from execution:
- Error patterns observed
- Unexpected behaviors
- Edge cases discovered
- Performance issues noted

**2. Diagnostic Reasoning**
Your classification of failures:
- Architecture vs runtime vs test issues
- Constitutional gaps identified
- Root cause analysis
- Byte-perfect path issues

**3. Required Amendments**
Governance changes needed (but not yet applied):
- Spec clarifications required
- New invariants identified
- Missing edge case coverage

**4. Patch Plans**
Detailed changes for future cycles:
- File-level modifications planned
- Implementation approach documented
- Dependencies noted

The key insight: **"Keep this, but do not act upon it yet."**

---

## Properties of AMB

### Deterministic
AMB is append-only and cycle-specific. It cannot be reformatted or rewritten casually.

### Non-Architectural
AMB never defines architecture or law. It only restates what you already discovered.

### Cross-Cycle Bridge
AMB is always read in later prompts:
- Governance Patch Cycle reads AMB
- Runtime Patch Cycle reads AMB
- Test Patch Cycle reads AMB

### Zero-Drift Safe
AMB does not modify source code. It does not contain normative rules. It simply preserves insights that will become input for later cycles.

---

## Why This Matters

Before AMB, LLM-driven development had a fatal flaw:

> LLMs forget previous insights unless you repeat them in every prompt, leading to drift and re-derivation.

AMB solves this by creating a **persistent, append-only architectural memory substrate**.

### Benefits

- Eliminates re-derivation of complex diagnostics
- Makes multi-cycle repair deterministic
- Allows deep architectural reasoning without losing context
- Ensures later cycles incorporate full historical insight
- Makes the AI behave like an engineer who remembers yesterday

---

## Using AMB in Practice

### Creating an AMB File

After a development session, capture insights in a structured format:

```markdown
# AMB: Session 2024-01-15

## Symptoms Observed
- CDP connection drops after 30 seconds idle
- Exit code 81 returned on concurrent exec (expected)
- Port exhaustion at ~200 sessions

## Diagnostic Analysis
- Idle timeout is WebSocket-level, not Telepipe
- Concurrent exec rejection working correctly
- Port exhaustion matches IANA ephemeral range limits

## Required Amendments
- Constitution should specify WebSocket keep-alive recommendation
- Add port exhaustion recovery to exhaustion-fallbacks spec

## Next Cycle Plan
- Implement keep-alive ping in CDP examples
- Add session cleanup before port exhaustion
```

### Reading AMB in Next Session

Start your next session by reading the AMB:

```
Context: Read AMB from previous session.
The following insights were captured:
[paste AMB contents]

Continue from where we left off.
```

### Updating AMB

After each significant session:
1. Append new symptoms observed
2. Update diagnostic reasoning
3. Mark completed items
4. Add new patch plans

---

## AMB and Governance

AMB works within the governance model:

```
                    ┌─────────────────┐
                    │   CONSTITUTION  │
                    │  (Supreme Law)  │
                    └────────┬────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
         ▼                   ▼                   ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│    EXECUTIVE    │ │   LEGISLATIVE   │ │    JUDICIAL     │
└────────┬────────┘ └────────┬────────┘ └────────┬────────┘
         │                   │                   │
         └───────────────────┼───────────────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │       AMB       │
                    │  (Memory Layer) │
                    │                 │
                    │ - Symptoms      │
                    │ - Diagnostics   │
                    │ - Amendments    │
                    │ - Plans         │
                    └─────────────────┘
```

AMB sits below governance — it informs but does not override.

---

## Staged Multi-Cycle Reasoning

The pattern enables sophisticated multi-cycle workflows:

**Cycle 1: Discovery**
- Run tests, observe failures
- Capture symptoms in AMB
- Document diagnostic reasoning

**Cycle 2: Governance**
- Read AMB from Cycle 1
- Propose constitutional amendments
- Wait for human approval

**Cycle 3: Implementation**
- Read AMB + approved amendments
- Implement runtime patches
- Update AMB with results

**Cycle 4: Validation**
- Read full AMB history
- Run comprehensive tests
- Close the loop

Each cycle builds on the previous, with full context preserved.

---

## What AMB Prevents

- **Drift** - Memory is structured and auditable
- **Forgotten insights** - Everything is captured
- **Duplicated reasoning** - No re-derivation needed
- **Ambiguous regeneration** - Clear patch plans
- **Premature patching** - Staged execution
- **Loss of architectural clarity** - Full context preserved

---

## Summary

The Architect Memory Buffer is:

> A deterministic, constitutional, cross-cycle memory mechanism that preserves diagnostics and patch plans while enforcing staged AI reasoning.

It allows AI agents to:
- Remember across sessions
- Build on previous work
- Maintain architectural coherence
- Execute multi-cycle repairs

**AMB makes AI agents behave like senior engineers who remember yesterday.**

---

## Next Steps

- [Mindset](mindset.md) - The verification-first approach
- [Completing the Loop](completing-the-loop.md) - Full verification cycles
- [Governance Model](../about/governance.md) - How AMB fits the system
