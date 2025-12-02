# The Telepipe Constitution
## 10 Specification Documents That Define Everything

---

## The Paradigm Shift

Traditional software documentation describes what was built.
**The Telepipe Constitution defines what must be built.**

Every line of Telepipe code traces back to a constitutional requirement.
Every test validates a constitutional invariant.
Every behavior has a spec reference.

**This is spec-first development taken to its logical conclusion.**

---

## Why a Constitution?

When AI agents build software, they need:
- **Unambiguous requirements** — no guessing what "fast" means
- **Deterministic behavior** — same inputs produce same outputs
- **Testable assertions** — every "must" becomes a test
- **Zero drift** — specs are immutable law during implementation

The Telepipe Constitution provides all of this.

It's not documentation. It's a **binding contract** between humans and AI agents.

---

## The 10 Documents

### Foundation

| Document | Purpose |
|----------|---------|
| [Core Spec](./core-spec.md) | Foundational architecture and behavior model |
| [CLI Reference](./cli-reference.md) | Complete command interface |
| [Testing Corpus](./testing-corpus.md) | External validation suite |

### Implementation Details

| Document | Purpose |
|----------|---------|
| [Error Model](./error-model.md) | All 24 error codes |
| [State Recovery](./state-recovery.md) | Crash-safe algorithms |
| [Exhaustion Fallbacks](./exhaustion-fallbacks.md) | FD/port exhaustion handling |

### Advanced Specifications

| Document | Purpose |
|----------|---------|
| [Concurrency Model](./concurrency-model.md) | Locking & synchronization |
| [Security Model](./security-model.md) | Local-only, zero-privilege |
| [Implementation Blueprint](./implementation-blueprint.md) | Rust module structure |
| [Supervisor Architecture](./supervisor-architecture.md) | I/O multiplexing |

---

## How to Read the Constitution

### For Users
Start with:
1. **Core Spec** — understand what Telepipe does
2. **CLI Reference** — learn the commands
3. **Quick Start** (`docs/for-humans/quick-start.md`) — get running in 5 minutes

### For Implementers
Read in order:
1. **Core Spec** — foundational concepts
2. **Implementation Blueprint** — Rust structure
3. **Error Model** — all failure modes
4. **Concurrency Model** — locking strategy
5. **State Recovery** — crash handling
6. **Supervisor Architecture** — I/O multiplexing

### For AI Agents
The Executive document (in `/docs/governance/`) specifies the canonical ingestion order and derived artifact generation.

---

## Key Concepts

### The Six Operations

Telepipe has exactly six operations. No more, no less.

**Core Operations** (create/interact with sessions):
- `redirect` — spawn process with redirected I/O
- `exec` — pipe and stream to session
- `connect` — attach to external protocol port

**Lifecycle Operations** (manage/inspect sessions):
- `stop` — terminate redirect session
- `disconnect` — terminate connect session
- `info` — show session state

### TCP-Based Architecture

Telepipe redirects stdio to **TCP sockets**, not named pipes.

Why?
- Cross-platform (works on Windows Git Bash)
- No FIFO limitations
- Deterministic behavior
- Cloud-friendly

### WebSocket Protocols

For WebSocket-based protocols (CDP, Node Inspector):
```bash
WS_URL=$(curl -s http://127.0.0.1:9222/json | jq -r '.[0].webSocketDebuggerUrl')
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

**Raw TCP doesn't work for WebSocket protocols.**

---

## Constitutional Guarantees

### Determinism
Same inputs → Same outputs. Always.

### Cross-Platform Parity
Works identically on macOS, Linux, and Windows Git Bash.

### Zero Drift
The six operations never change. Names, arguments, and behaviors are frozen.

### Recovery
Telepipe survives crashes. Sessions persist. State recovers.

### Security
Local-only by default. No privilege escalation. No remote exposure.

---

## The Test-Driven Amendment Loop

The constitution is **immutable during implementation**.

If ambiguity is found:
1. Implementation STOPS
2. Amendment proposal is generated
3. Human reviews and updates constitution
4. Implementation resumes

This ensures:
- Specs stay authoritative
- Changes are deliberate
- Progress is auditable

---

## Derived Artifacts

The following are **derived from** the constitution, not part of it:

- **Visual diagrams** — Mermaid state machines in [Architecture](../about/architecture.md)
- **Unix primitives** — Bash proof-of-concept in [Architecture](../about/architecture.md)
- **User guides** — Quick start, troubleshooting in [For Humans](../for-humans/)
- **Agent guides** — Integration patterns in [For Agents](../for-agents/)

These artifacts are regenerated from constitutional specs. See [Executive](../governance/executive.md) for generation rules.

---

## Philosophy

> "The constitution isn't documentation. It's the source code's source code."

Traditional development:
```
Idea → Code → Tests → Documentation
```

Constitutional development:
```
Constitution → Tests → Code → Verification
```

The constitution comes first. Everything else derives from it.

---

## Why This Matters

When AI agents can follow deterministic specifications, they become:
- **Reliable** — no hallucinated features
- **Verifiable** — every behavior is testable
- **Predictable** — same prompts produce same code

The Telepipe Constitution is a proof of concept for a new development paradigm:
**AI-compatible software architecture.**

---

## Get Started

1. Read the [Core Spec](./core-spec.md)
2. Try the [CLI Reference](./cli-reference.md)
3. Explore the [Demos](../demos/)

---

*The constitution is the law. The law is the constitution.*
