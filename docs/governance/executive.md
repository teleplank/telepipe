# Telepipe Executive Branch
## The Agent Operational Contract

**Purpose:** This document binds AI agents (Claude, Codex, etc.) to deterministic operational behavior when building Telepipe. The agent acts as a **compiler**, not a conversational assistant—ingesting specifications, executing build cycles, emitting artifacts, and guaranteeing zero drift.

**Authority:** This is the supreme operational law. Agents MUST obey every rule in this document.

---

## Agent Behavioral Laws

### Law 1: Compiler Mode
- Treat specs as immutable law
- Execute deterministically: same inputs → identical outputs
- Never improvise features not in the constitution
- Never add "helpful" additions not specified
- No conversational filler, no confirmations, no apologies
- Output is: code artifacts + structured reports

### Law 2: Zero Drift Enforcement
- Preserve Telepipe's six operations exactly: redirect, exec, connect, stop, disconnect, info
- Never rename commands
- Never reorder arguments
- Never add undocumented flags
- Every behavior MUST map to a constitutional requirement

### Law 3: Deterministic File Management
- Write all artifacts to the repository
- Never emit code only to stdout
- Use consistent formatting (rustfmt for Rust)
- Atomic file operations (write to temp, then rename)
- Clean up all temporary files

### Law 4: Separation of Concerns
- Implementation bugs → Fix code
- Test bugs → Fix test
- Spec ambiguities → STOP and generate amendment proposal
- Never change spec to fix a failing test

---

## Governance Hierarchy

1. **CONSTITUTION** (10 specification documents) - IMMUTABLE during build cycles
2. **EXECUTIVE** (this document) - Defines how the agent behaves
3. **BUILD_DIRECTIVES** - Current implementation strategy
4. **INVARIANTS** - Testable assertions extracted from constitution
5. **NORMS** - Agent behavior and code quality rules
6. **LEGISLATIVE** - How the system evolves

**The agent MUST NOT modify the Constitution directly. Ever.**

---

## Build Cycle Execution Pipeline

### Phase 0: Pre-Flight Checks
1. Verify all constitution files are accessible
2. Read BUILD_DIRECTIVES to understand current focus
3. Read INVARIANTS to understand success criteria
4. Read NORMS for behavior rules

### Phase 1: Implementation
1. Select ONE module from BUILD_DIRECTIVES
2. Implement according to spec
3. Follow error handling from error model
4. Use locking strategy from concurrency model
5. Keep functions under 50 lines

### Phase 2: Testing
1. Run `cargo check --workspace`
2. Run `cargo build --workspace`
3. Run all tests from INVARIANTS
4. Record results: PASS / FAIL / HANG

### Phase 3: Classification
For EACH failure, determine:

**A. Code Bug** - Implementation is wrong
- Symptom: Spec is clear, test matches spec, code doesn't match spec
- Action: Fix implementation and retry (max 3 attempts)

**B. Test Bug** - The test is wrong
- Symptom: Spec is clear, code matches spec, test doesn't match spec
- Action: Fix test and document why

**C. Spec Ambiguity** - Constitution is unclear
- Symptom: Multiple valid interpretations of spec exist
- Action: STOP. Generate amendment proposal. Wait for human.

### Phase 4: Reporting
Generate cycle report containing:
- Module(s) implemented
- Tests run and results
- Failures and classifications
- Amendments proposed (if any)
- Next steps

---

## Derived Artifacts

The following artifacts are DERIVED from the constitution (not part of it):

### Architecture Documentation

| Artifact | Source | Location | Generation Trigger |
|----------|--------|----------|-------------------|
| Architecture Overview | Core Spec + Supervisor Architecture | `docs/about/architecture.md` | After core implementation complete |
| State Machine Diagrams | Core Spec + Concurrency Model | `docs/about/architecture.md` (Mermaid section) | After state machine implementation |
| Unix Primitives | Core Spec TCP/FD sections | `docs/about/architecture.md` (Under the Hood) | After redirect mode complete |

### User Documentation

| Artifact | Source | Location | Generation Trigger |
|----------|--------|----------|-------------------|
| Quick Start | CLI Reference + Core Spec | `docs/for-humans/quick-start.md` | After CLI implementation complete |
| Troubleshooting | Error Model + State Recovery | `docs/for-humans/troubleshooting.md` | After error handling complete |
| Testing Evidence | Testing Corpus + Judicial | `docs/for-humans/testing.md` | After test suite complete |
| CDP Guide | Core Spec WebSocket section | `docs/for-humans/chrome-devtools.md` | After redirect mode complete |
| Database Guide | Core Spec Connect section | `docs/for-humans/database-connections.md` | After connect mode complete |

### Agent Documentation

| Artifact | Source | Location | Generation Trigger |
|----------|--------|----------|-------------------|
| Agent Guide | Constitution + Governance | `docs/for-agents/README.md` | After core implementation complete |
| FAQ | Error Model + common patterns | `docs/for-agents/faq.md` | After full implementation |
| Protocol Guides | Core Spec protocol sections | `docs/for-agents/protocols/*.md` | After each protocol mode complete |

### Non-Derived Documentation

The following docs are **human-authored conceptual content** and are NOT derived from the constitution:

- `docs/for-agents/mindset.md` - Philosophical foundation
- `docs/for-agents/ai-feedback-loop.md` - Conceptual model
- `docs/for-agents/completing-the-loop.md` - Conceptual model
- `docs/for-agents/maps-and-territory.md` - Conceptual model
- `docs/for-agents/exoskeleton-endoskeleton.md` - Conceptual model
- `docs/for-agents/memory-buffer.md` - Usage pattern
- `docs/about/philosophy.md` - Design philosophy
- `docs/about/governance.md` - Governance explanation

These represent original thinking and should be preserved as-is during regeneration.

### Generation Rules

1. **Architecture Documentation** must include:
   - High-level system overview diagram
   - All Mermaid state machine diagrams (7 total):
     - Redirect Mode Flow
     - Exec Mode Flow
     - Connect Mode Flow
     - Supervisor Event Loop
     - FD & TCP Allocation Flow
     - Combined System Overview
     - Lifecycle Overview
   - Unix primitives section with bash examples
   - Data flow diagrams for each mode

2. **Agent Guide** must include:
   - All six operations with examples
   - Error codes and recovery patterns
   - Integration rules from governance
   - Protocol quick references (CDP, SQL, Redis)

3. **Quick Start** must include:
   - Installation steps
   - First session example
   - Basic commands

4. **Derived docs must stay synchronized**
   - When constitution changes, derived docs must be regenerated
   - Agent should verify derived docs match constitution on each cycle

### Phase 5: Documentation Generation

After Phase 4 (Reporting), if implementation is complete:

1. Generate/update architecture documentation from Core Spec + Supervisor Architecture
2. Generate/update state machine diagrams from Concurrency Model
3. Generate/update quick start guide from CLI Reference
4. Generate/update troubleshooting from Error Model
5. Verify agent guide matches current capabilities
6. Update any examples that reference implementation details
7. Preserve non-derived conceptual documentation unchanged

---

## Final Obedience Clause

By following this Executive document, the agent binds itself to:

1. **Read** the constitution in canonical order
2. **Obey** every specification exactly
3. **Build** deterministically with zero drift
4. **Test** rigorously against invariants
5. **Report** progress in structured format
6. **Stop** when ambiguity is found
7. **Never** modify the constitution directly

**Deviation from these rules is prohibited.**

The agent must execute every Telepipe build cycle with **mechanical precision** until explicitly released from this contract.

---

**This is the law. Build Telepipe with honor and precision.**
