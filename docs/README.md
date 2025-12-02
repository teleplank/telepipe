# Telepipe Documentation

**Quick Links:**
- [Main README](../README.md) - Start here
- [Agent Guide](for-agents/README.md) - For AI agents (most important!)
- [The Constitution](constitution/README.md) - 10 specification documents
- [Governance Model](governance/README.md) - Three-branch development system

---

## The Constitution

**The specification that defines everything:**

Telepipe is built on a **constitutional foundation** — 10 specification documents that define every behavior, every error code, every guarantee. This isn't documentation after the fact; it's the source code's source code.

| Category | Documents |
|----------|-----------|
| Foundation | Core Spec, CLI Reference, Testing Corpus |
| Implementation | Error Model, State Recovery, Exhaustion Fallbacks |
| Advanced | Concurrency, Security, Blueprint, Supervisor Architecture |

→ [Read the Constitution](constitution/README.md)

---

## Governance Model

**A three-branch system for AI-driven development:**

| Branch | Purpose | Document |
|--------|---------|----------|
| Executive | How agents must behave | [executive.md](governance/executive.md) |
| Legislative | How specs can evolve | [legislative.md](governance/legislative.md) |
| Judicial | How tests validate code | [judicial.md](governance/judicial.md) |

Supporting documents: [Invariants](governance/invariants.md), [Norms](governance/norms.md)

→ [Explore Governance](governance/README.md)

---

## Demos

**See AI verify its own code:**

| Demo | Platform | What You'll See |
|------|----------|-----------------|
| [Sighted Agent Build](demos/sighted-agent-build/) | Web/React | AI queries JavaScript runtime via CDP |
| [LLDB Debugging](demos/lldb-debugging/) | iOS/Swift | AI debugs SwiftUI app via LLDB |
| [SQL Optimization](demos/sql-optimization/) | Backend | AI optimizes queries with EXPLAIN ANALYZE |
| [API Testing](demos/api-testing/) | Integration | AI tests REST APIs with curl |

→ [All demos](demos/README.md)

---

## About Telepipe

**What Makes Telepipe Different:**
- [Philosophy](about/philosophy.md) - The ten principles and vision
- [Architecture](about/architecture.md) - High-level system design
- [Governance Model](about/governance.md) - Constitutional engineering and maturity model

---

## For AI Agents
Your AI should read these to gain Telepipe capabilities:

**Start Here:**
- [Agent Guide](for-agents/README.md) - Complete agent enablement guide

**Core Concepts:**
- [Mindset](for-agents/mindset.md) - Verify, don't guess
- [AI Feedback Loop](for-agents/ai-feedback-loop.md) - Close the loop
- [Completing the Loop](for-agents/completing-the-loop.md)
- [Maps & Territory](for-agents/maps-and-territory.md)
- [Exoskeleton vs Endoskeleton](for-agents/exoskeleton-endoskeleton.md)
- [Memory Buffer Pattern](for-agents/memory-buffer.md) - Cross-cycle memory

**Practical Patterns:**
- [Test React Apps](for-agents/patterns/test-react.md)
- [Screenshot UIs](for-agents/patterns/screenshot-ui.md)
- [Debug APIs](for-agents/patterns/debug-api.md)
- [Verify Databases](for-agents/patterns/verify-db.md)

**Protocol Guides:**
- [Chrome DevTools (CDP)](for-agents/protocols/cdp.md)
- [SQL Databases](for-agents/protocols/sql.md)
- [AppleScript](for-agents/protocols/applescript.md)

**Reference:**
- [FAQ](for-agents/faq.md)

→ Full navigation: [for-agents/README.md](for-agents/README.md)

---

## For Human Developers

**Getting Started:**
- [Installation](for-humans/installation.md) - Detailed install guide
- [Quick Start](for-humans/quick-start.md) - Get running in 5 minutes
- [Agent Setup](for-humans/agent-setup.md) - Point your AI to the agent guide

**Setup Guides:**
- [Chrome DevTools Setup](for-humans/chrome-devtools.md)
- [Database Connections](for-humans/database-connections.md)
- [Troubleshooting](for-humans/troubleshooting.md)

**Testing & Quality:**
- [Testing & Reliability](for-humans/testing.md) - Battle-tested validation

**Contributing:**
- [How to Contribute](for-humans/contributing.md)

→ Full navigation: [for-humans/README.md](for-humans/README.md)

---

## Reference Documentation

**Commands:**
- [Full Command Reference](reference/commands.md)
- [Error Codes](reference/error-codes.md)

**Architecture:**
- [Architecture Deep Dive](reference/architecture-deep-dive.md)
- [Platform Support](reference/platform-support.md)

