# Telepipe Demos
## See AI Verify Its Own Code

---

## The Full-Stack Agent Enablement Final Boss

Four demos. Four runtimes. One paradigm: **AI that proves, not hopes.**

| Demo | Layer | Runtime | What AI Verifies |
|------|-------|---------|------------------|
| [Sighted Agent Build](./sighted-agent-build/) | Frontend | CDP | React state via `window.__GAME_STATE__` |
| [LLDB Debugging](./lldb-debugging/) | Mobile | LLDB | SwiftUI state via `po viewModel` |
| [SQL Optimization](./sql-optimization/) | Backend | PostgreSQL | Query plans via `EXPLAIN ANALYZE` |
| [API Testing](./api-testing/) | Integration | HTTP | Endpoints via `curl` |

**Complete stack coverage with runtime verification.**

---

## Available Demos

### [Sighted Agent Build](./sighted-agent-build/) - Web/React
**Watch an AI build an app while verifying its own runtime state**

An AI agent builds a Tic-Tac-Toe game in React, but with a twist: it connects to Chrome via CDP and queries actual JavaScript runtime state to prove its code works.

- **Time:** ~10 minutes
- **Runtime:** Chrome DevTools Protocol (CDP)
- **What you'll see:** AI querying `window.__GAME_STATE__` and verifying React closure values
- **The breakthrough:** AI produces proof, not assumptions

```
Traditional: "This code should work..."
With Telepipe: "I queried board[0] via CDP. Value: 'X'. Verified."
```

→ [Start the demo](./sighted-agent-build/QUICK_START.md)

---

### [LLDB Debugging](./lldb-debugging/) - iOS/Swift
**Watch an AI debug an iOS app by inspecting Swift runtime state**

An AI agent builds a TodoApp in SwiftUI, discovers a persistence bug via LLDB inspection, fixes it, and verifies the fix - all through actual runtime queries.

- **Time:** ~10 minutes
- **Runtime:** LLDB (LLVM Debugger)
- **What you'll see:** AI querying `po viewModel.items.count` and finding bugs through inspection
- **The breakthrough:** AI can debug iOS code, not just write it

```
Traditional: "It compiles, so it probably works..."
With Telepipe: "I queried items.count via LLDB after restart. Value: 0. BUG FOUND!"
```

→ [Start the demo](./lldb-debugging/QUICK_START.md)

---

### [SQL Optimization](./sql-optimization/) - Backend/PostgreSQL
**Watch an AI optimize queries with actual execution plan proof**

An AI agent analyzes slow queries via EXPLAIN ANALYZE, identifies bottlenecks, creates indexes, and verifies 100x+ speedups - all with measured proof.

- **Time:** ~10 minutes
- **Runtime:** PostgreSQL EXPLAIN ANALYZE
- **What you'll see:** AI transforming Seq Scan (5234ms) → Index Scan (52ms)
- **The breakthrough:** AI optimizes with metrics, not guesses

```
Traditional: "Try adding an index?"
With Telepipe: "EXPLAIN ANALYZE shows Seq Scan → Index Scan. 100x faster. Verified."
```

→ [Start the demo](./sql-optimization/QUICK_START.md)

---

### [API Testing](./api-testing/) - Integration/REST
**Watch an AI build and test APIs with actual HTTP requests**

An AI agent builds a Todo REST API, starts the server via Telepipe, and tests all endpoints with real HTTP requests - verifying status codes, response bodies, and error handling.

- **Time:** ~10 minutes
- **Runtime:** HTTP (curl)
- **What you'll see:** AI testing POST → 201, GET → 200, DELETE → 204, and error cases
- **The breakthrough:** AI can test APIs, not just write them

```
Traditional: "The endpoint should return 201..."
With Telepipe: "curl POST returned 201, body has id. Verified."
```

→ [Start the demo](./api-testing/QUICK_START.md)

---

## The Pattern

All four demos show the same paradigm:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│   FRONTEND (CDP)   MOBILE (LLDB)   BACKEND (SQL)   INTEGRATION (HTTP)   │
│   ══════════════   ════════════    ═════════════   ═══════════════════  │
│                                                                         │
│   Chrome DevTools  LLVM Debugger   PostgreSQL      REST API             │
│   JavaScript       Swift runtime   Exec. plans     HTTP requests        │
│   React closures   SwiftUI state   Performance     Status codes         │
│   window.__STATE__ po viewModel    EXPLAIN         curl -X POST         │
│                                                                         │
│   SAME PARADIGM:                                                        │
│   AI connects → AI queries → AI verifies → AI proves                    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Coming Soon

More demos showing AI agents verifying their own work:

- **macOS Automation** - AI controls Xcode via AppleScript
- **Full-Stack** - AI debugging frontend + backend + database + API simultaneously

---

## Why Demos Matter

These aren't just examples. They're **proof** that the feedback loop closes.

When you run these demos, you'll see your AI:
1. Write code
2. Connect to real systems
3. Query actual state
4. Verify its own work
5. Produce evidence

**That's the paradigm shift. From hope to proof.**

---

## Choose Your Demo

| Demo | Platform | Runtime | Time |
|------|----------|---------|------|
| [Sighted Agent Build](./sighted-agent-build/) | Web | CDP | ~10 min |
| [LLDB Debugging](./lldb-debugging/) | iOS | LLDB | ~10 min |
| [SQL Optimization](./sql-optimization/) | Backend | PostgreSQL | ~10 min |
| [API Testing](./api-testing/) | Integration | HTTP | ~10 min |

---

*Each demo is self-contained. Pick one and try it. Or try all four for the complete verification experience.*
