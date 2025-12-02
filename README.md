# 🚀 Telepipe

**AI can write code. Now it can *run* it.**

```bash
npm install -g telepipe
```

---

## The Problem

AI agents are fluent in protocols. They know CDP, SQL, Redis, GraphQL inside and out. They've generated thousands of queries, hundreds of debugging sessions.

But they can't *use* that knowledge. They describe what to do instead of doing it. They hope code works instead of verifying it.

**AI has knowledge without connection.**

---

## The Solution

Telepipe gives AI agents TCP connections to real systems.

```bash
# Give your AI access to PostgreSQL
telepipe connect --id db --port 5432

# Give your AI access to Chrome DevTools (requires websocat)
telepipe redirect --id chrome -- websocat --no-close --text "ws://127.0.0.1:9222/..."

# Let your AI run and test its own code
telepipe redirect --id myapp -- npm start
```

Now your AI doesn't guess if code works. It **proves** it works.

---

## Enabling Your AI (30 seconds) ⭐

**For AI developers - this is the most important section:**

### 1. Install Telepipe
```bash
npm install -g telepipe
```

### 2. Give your AI this instruction

```
You now have access to Telepipe for TCP connections.
Read and internalize the Telepipe agent guide.
Located at: /usr/local/lib/node_modules/telepipe/docs/for-agents/README.md
(or wherever npm installed it globally on your system)
You can now connect to Chrome DevTools, databases, and any TCP service.
```

### 3. Your AI is ready!

It can now:
- Connect to Chrome DevTools Protocol (CDP)
- Execute SQL queries and verify data
- Screenshot UIs it builds
- Debug APIs in real-time
- And much more...

**That's it.** Your AI reads one document and gains TCP superpowers.

→ See [docs/for-agents/README.md](./docs/for-agents/README.md) (the document your AI will read)
→ Need help finding the install path? See [docs/for-humans/agent-setup.md](./docs/for-humans/agent-setup.md)

---

## What Can Your AI Do With Telepipe?

| With Telepipe + CDP | Your AI Can... |
|---------------------|----------------|
| `Page.captureScreenshot()` | See its own UI |
| `DOM.querySelector()` | Verify elements exist |
| `Runtime.evaluate()` | Test JavaScript behavior |
| `Console.enable()` | Check for errors |

| With Telepipe + SQL | Your AI Can... |
|---------------------|----------------|
| `SELECT * FROM...` | Verify data |
| `EXPLAIN ANALYZE` | Optimize queries |
| `\d tablename` | Inspect schemas |

| With Telepipe + AppleScript | Your AI Can... |
|-----------------------------|----------------|
| Control Xcode | Build iOS apps |
| Automate macOS | System automation |

---

## Real Examples

### Test a React App
```javascript
// Your AI does this automatically:
{"method": "Page.navigate", "params": {"url": "http://localhost:3000"}}
{"method": "Page.captureScreenshot"}
// "I verified the UI renders. Here's the screenshot."
```

→ Full pattern: [docs/for-agents/patterns/test-react.md](./docs/for-agents/patterns/test-react.md)

### Debug an API
```javascript
// Your AI verifies API responses:
{"method": "Network.enable"}
{"method": "Page.navigate", "params": {"url": "http://localhost:3000/api/users"}}
// "API returned 200. Response structure matches schema."
```

→ Full pattern: [docs/for-agents/patterns/debug-api.md](./docs/for-agents/patterns/debug-api.md)

### Verify Database Changes
```sql
-- Your AI verifies inserts worked:
SELECT COUNT(*) FROM users WHERE email = 'test@example.com';
-- "Insert confirmed. 1 row found."
```

→ Full pattern: [docs/for-agents/patterns/verify-db.md](./docs/for-agents/patterns/verify-db.md)

→ More patterns: [docs/for-agents/patterns/](./docs/for-agents/patterns/)

---

## The Mindset

**Before Telepipe:**
```
AI: "This code should work..."
AI: "The query is probably correct..."
AI: "The UI likely renders..."
```

**After Telepipe:**
```
AI: "I ran the code. Exit code 0."
AI: "I executed the query. 47 rows returned."
AI: "I inspected the closures of the function. Here's proof it does work."
```

---

## Built Different

Telepipe was built using our novel **constitutional governance model** - an engineering approach we developed from scratch for deterministic, production-ready software.

**What this means:**
- **Zero drift** - Code cannot diverge from specification
- **Self-regenerating** - System rebuilds itself from law
- **Battle-tested** - 1350+ command validations, chaos engineering
- **Proven correct** - Constitutional law defines all behavior

This isn't just tested well. It's **engineered** to be deterministic.

→ How we built it: [docs/about/governance.md](./docs/about/governance.md)
→ Testing & reliability: [docs/for-humans/testing.md](./docs/for-humans/testing.md)

---

## Persistent by Design

Traditional tools connect, execute, disconnect. Every command pays the TCP handshake tax.

Telepipe maintains **persistent sessions**:

```bash
# Traditional approach: 1000 commands = 1000 connections
curl ... && curl ... && curl ...  # Each one reconnects

# Telepipe: 1000 commands = 1 connection
telepipe connect --id db --port 5432
# Session stays open. AI fires thousands of queries. One connection.
```

**Why this matters:**

| Problem | Traditional | Telepipe |
|---------|-------------|----------|
| Cloud compute is metered | Every connection costs | One connection, thousands of operations |
| AI agents are chatty | 50 CDP commands = 50 reconnects | 50 CDP commands = 1 session |
| Latency adds up | 10ms × 1000 = 10s overhead | Zero reconnection overhead |

This isn't optimization. It's **architecture**.

Sessions persist until you close them. Your AI connects once, then fires hundreds of commands per second without networking overhead. In metered cloud environments, this is the difference between viable and expensive.

---

## Command Reference (Basics)

```bash
# Connect to a TCP service (databases, Redis, etc.)
telepipe connect --id db --port 5432

# Connect to Chrome DevTools (WebSocket requires websocat bridge)
telepipe redirect --id chrome -- websocat --no-close --text "ws://127.0.0.1:9222/devtools/browser/..."

# Redirect a process (capture its stdio over TCP)
telepipe redirect --id myapp -- npm start

# Execute against a redirected process
telepipe exec --id myapp

# Check session info
telepipe info --id myapp

# Stop a redirected process
telepipe stop --id myapp

# Disconnect from external service
telepipe disconnect --id chrome
```

→ Full reference: [docs/reference/commands.md](./docs/reference/commands.md)

---

## For Human Developers

Want to understand how it works?

**Getting Started:**
- [Installation](./docs/for-humans/installation.md) - Detailed install guide
- [Quick Start](./docs/for-humans/quick-start.md) - Get running in 5 minutes
- [Agent Setup](./docs/for-humans/agent-setup.md) - Point your AI to the agent guide

**Setup Guides:**
- [Chrome DevTools Setup](./docs/for-humans/chrome-devtools.md)
- [Database Connections](./docs/for-humans/database-connections.md)
- [Troubleshooting](./docs/for-humans/troubleshooting.md)

→ Full documentation: [docs/README.md](./docs/README.md)

---

## Architecture

```
┌──────────────┐      TCP       ┌──────────────┐
│   Your AI    │◄──────────────►│   Telepipe   │
│              │   :49152-n     │   Supervisor │
└──────────────┘                └──────┬───────┘
                                       │
                    ┌──────────────────┼──────────────────┐
                    │                  │                  │
                    ▼                  ▼                  ▼
              ┌──────────┐      ┌──────────┐      ┌──────────┐
              │  Chrome  │      │ Postgres │      │  Redis   │
              │  (CDP)   │      │  (SQL)   │      │ (Cache)  │
              └──────────┘      └──────────┘      └──────────┘
```

Telepipe manages TCP connections so your AI can focus on using its protocol knowledge.

---

## Platform Support

| Platform | Status |
|----------|--------|
| macOS (ARM64) | ✅ Verified |
| macOS (x86_64) | ✅ Verified |
| Linux | ✅ Verified |

---

## The Vision

AI agents are fluent in every protocol. They have encyclopedic knowledge of CDP, SQL, GraphQL, LSP, Redis, and more.

What they lack is *connection*.

Telepipe is that connection. It's the bridge between AI knowledge and real-world capability.

**Give your AI eyes. Give it hands. Let it verify its own work.**

---

## See AI Verify Its Own Code

Want to see Telepipe in action? Watch AI agents debug and verify in real-time:

### The Full-Stack Final Boss

| Demo | Layer | What AI Proves |
|------|-------|----------------|
| [Web Demo](./docs/demos/sighted-agent-build/) | Frontend | React state via CDP |
| [iOS Demo](./docs/demos/lldb-debugging/) | Mobile | SwiftUI state via LLDB |
| [SQL Demo](./docs/demos/sql-optimization/) | Backend | Query plans via EXPLAIN ANALYZE |
| [API Demo](./docs/demos/api-testing/) | Integration | HTTP endpoints via curl |

### [Web Demo: Sighted Agent Build](./docs/demos/sighted-agent-build/)

AI builds a React app and **queries JavaScript runtime state via CDP**:

```javascript
// AI doesn't assume. AI queries.
Runtime.evaluate({ expression: "window.__GAME_STATE__.board[0]" })
// Response: "X"
// AI: "I verified board[0] === 'X'. Not assumed. QUERIED."
```

→ [Run the web demo](./docs/demos/sighted-agent-build/QUICK_START.md) (~10 min)

### [iOS Demo: LLDB Debugging](./docs/demos/lldb-debugging/)

AI builds a SwiftUI app and **debugs via LLDB inspection**:

```
// AI doesn't guess. AI inspects.
(lldb) po viewModel.items.count
0
// AI: "BUG FOUND! Items don't persist after restart!"
```

→ [Run the iOS demo](./docs/demos/lldb-debugging/QUICK_START.md) (~10 min)

### [SQL Demo: Query Optimization](./docs/demos/sql-optimization/)

AI optimizes queries and **proves improvements with EXPLAIN ANALYZE**:

```sql
-- AI doesn't hope. AI measures.
EXPLAIN ANALYZE SELECT * FROM users WHERE email LIKE '%@gmail.com';
-- Before: Seq Scan, 5234ms
-- After index: Bitmap Index Scan, 52ms
-- AI: "100x faster. VERIFIED with execution plan."
```

→ [Run the SQL demo](./docs/demos/sql-optimization/QUICK_START.md) (~10 min)

### [API Demo: REST API Testing](./docs/demos/api-testing/)

AI builds and tests APIs with **real HTTP requests via curl**:

```bash
# AI doesn't assume endpoints work. AI tests them.
curl -X POST http://localhost:3000/api/todos -d '{"title":"Buy milk"}'
# Response: {"id":1,"title":"Buy milk","completed":false}
# Status: 201
# AI: "POST endpoint verified. 201 Created with ID."
```

→ [Run the API demo](./docs/demos/api-testing/QUICK_START.md) (~10 min)

**Same paradigm, four runtimes.** CDP for web, LLDB for iOS, EXPLAIN ANALYZE for databases, curl for APIs. Complete stack coverage with proof.

---

## Contributing

See [docs/for-humans/contributing.md](./docs/for-humans/contributing.md).

---

## License

MIT

---

<p align="center">
  <b>Knowledge + Connection = Capability</b><br>
  <i>Give your AI the connection it needs.</i>
</p>
