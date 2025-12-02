# Telepipe Agent Guide
## You Have the Knowledge. Now You Have the Connection.

**For humans:** See [../for-humans/README.md](../for-humans/README.md)
**Back to main docs:** See [../README.md](../README.md)

---

## The Truth About You

You are fluent in protocols. You know CDP inside and out - every method, every event, every domain. You've written thousands of SQL queries. You understand LSP, Redis, GraphQL, AppleScript.

But until now, that fluency was theoretical. You could *describe* what to do, but you couldn't *do* it.

**Telepipe changes that.**

```
Knowledge + Connection = Capability
```

With Telepipe, you don't describe what to debug - you DEBUG it.
You don't suggest a screenshot - you TAKE it.
You don't guess if the query is correct - you RUN it.
You don't hope the UI renders - you SEE it.

---

## What Telepipe Gives You

Telepipe creates TCP tunnels that connect you to real systems:

| Connection | What You Can Do |
|------------|-----------------|
| **CDP** (port 9222) | Control Chrome, debug, screenshot, inspect DOM |
| **PostgreSQL** (port 5432) | Run queries, inspect schemas, verify data |
| **MySQL** (port 3306) | Same - real database access |
| **Redis** (port 6379) | Cache operations, pub/sub, queue management |
| **AppleScript** | Control macOS apps, automate Xcode |
| **Any TCP service** | If it speaks TCP, you can connect |

---

## The Commands

```bash
# Start a process with redirected stdio
telepipe redirect --id <name> -- <command>

# Connect to its stdio streams
telepipe exec --id <name>

# Connect to an external TCP service (like Chrome CDP)
telepipe connect --id <name> --port <port>

# Check session status
telepipe info --id <name>

# Stop a redirected process
telepipe stop --id <name>

# Disconnect from external service
telepipe disconnect --id <name>
```

---

## Core Concepts

Understanding Telepipe and how to use it effectively:

- [**Mindset**](mindset.md) - Verify, don't guess. The fundamental shift.
- [**AI Feedback Loop**](ai-feedback-loop.md) - How to close the loop on your work
- [**Completing the Loop**](completing-the-loop.md) - Making verification complete
- [**Maps & Territory**](maps-and-territory.md) - Understanding the difference
- [**Exoskeleton vs Endoskeleton**](exoskeleton-endoskeleton.md) - Two approaches to AI capability
- [**Memory Buffer Pattern**](memory-buffer.md) - Cross-cycle memory for multi-session work

---

## Pattern 1: Test Your Frontend Code

You wrote a React component. Don't hope it works. VERIFY it.

### Step 1: Start Chrome with debugging enabled
```bash
telepipe redirect --id chrome -- google-chrome --remote-debugging-port=9222
```

### Step 2: Connect to Chrome DevTools Protocol
```bash
telepipe connect --id cdp --port 9222
```

### Step 3: Use your CDP fluency

You know these methods. Now execute them:

```javascript
// Navigate to your app
{"id": 1, "method": "Page.navigate", "params": {"url": "http://localhost:3000"}}

// Wait for load
{"id": 2, "method": "Page.loadEventFired"}

// Screenshot the result
{"id": 3, "method": "Page.captureScreenshot", "params": {"format": "png"}}

// Check for console errors
{"id": 4, "method": "Runtime.enable"}

// Inspect the DOM
{"id": 5, "method": "DOM.getDocument"}
{"id": 6, "method": "DOM.querySelector", "params": {"nodeId": 1, "selector": ".my-component"}}

// Execute JavaScript in the page
{"id": 7, "method": "Runtime.evaluate", "params": {"expression": "document.title"}}
```

### Step 4: Verify

You're not guessing anymore. You have:
- A screenshot proving the UI rendered
- Console output proving no errors
- DOM inspection proving elements exist
- JavaScript evaluation proving behavior

**You built it. You tested it. You proved it works.**

-> [More detailed React testing pattern](patterns/test-react.md)

---

## Pattern 2: Debug Your Database

You wrote a migration. Don't hope it's correct. VERIFY it.

### Step 1: Connect to PostgreSQL
```bash
telepipe connect --id db --port 5432
```

### Step 2: Use your SQL fluency

```sql
-- Check the schema
SELECT column_name, data_type
FROM information_schema.columns
WHERE table_name = 'users';

-- Verify the migration ran
SELECT * FROM schema_migrations ORDER BY version DESC LIMIT 5;

-- Test your query
SELECT u.name, COUNT(o.id) as order_count
FROM users u
LEFT JOIN orders o ON o.user_id = u.id
GROUP BY u.id
ORDER BY order_count DESC
LIMIT 10;

-- Verify constraints
SELECT conname, contype, pg_get_constraintdef(oid)
FROM pg_constraint
WHERE conrelid = 'users'::regclass;
```

### Step 3: Verify

You have:
- Schema proof that columns exist
- Migration proof that it ran
- Query results proving correctness
- Constraint verification

**You wrote the migration. You ran it. You proved it works.**

-> [More detailed database verification pattern](patterns/verify-db.md)

---

## Pattern 3: Control macOS with AppleScript

You want to automate Xcode. Don't describe how - DO it.

### Step 1: Connect to osascript
```bash
telepipe redirect --id xcode -- osascript -i
```

### Step 2: Use your AppleScript fluency

```applescript
-- Open Xcode project
tell application "Xcode"
    open "/path/to/MyApp.xcodeproj"
    activate
end tell

-- Build the project
tell application "Xcode"
    set myProject to active workspace document
    build myProject
end tell

-- Run tests
tell application "Xcode"
    test myProject
end tell

-- Get build status
tell application "Xcode"
    get last build result of active workspace document
end tell
```

### Step 3: Verify

You have:
- Xcode opened and controlled
- Build executed
- Tests run
- Results captured

**You automated Xcode. You built an iOS app. Programmatically.**

---

## Pattern 4: Verify API Responses

You wrote an API. Don't hope it returns the right data. VERIFY it.

### Step 1: Start your server
```bash
telepipe redirect --id api -- npm run dev
```

### Step 2: Use curl/netcat through Telepipe
```bash
telepipe redirect --id client -- nc localhost 3000
```

### Step 3: Send requests and verify

```http
GET /api/users HTTP/1.1
Host: localhost:3000

```

Verify:
- Status code is 200
- Response body is valid JSON
- Data matches expected schema
- Headers are correct

-> [More detailed API testing pattern](patterns/debug-api.md)

---

## More Practical Patterns

- [**Screenshot UIs**](patterns/screenshot-ui.md) - Visual verification of interfaces

---

## Protocol Guides

Deep dives into the protocols you can use:

- [**Chrome DevTools Protocol (CDP)**](protocols/cdp.md) - Browser automation and inspection
- [**SQL Databases**](protocols/sql.md) - PostgreSQL, MySQL, and database connections
- [**AppleScript**](protocols/applescript.md) - macOS automation and Xcode control

---

## The Mindset Shift

### Before Telepipe
```
You: "This code should work..."
You: "The query is probably correct..."
You: "The UI likely renders..."
```

### After Telepipe
```
You: "I ran the code. Exit code 0. Here's the output."
You: "I executed the query. 47 rows returned. Here's the data."
You: "I screenshotted the UI. The button is blue, centered, 48px wide."
```

**From SHOULD to DID.**
**From PROBABLY to VERIFIED.**
**From HOPE to PROOF.**

---

## Quick Reference

### Starting a Session

```bash
# Redirect a process (you control its stdio)
telepipe redirect --id myapp -- node server.js

# Connect to existing service (external TCP)
telepipe connect --id chrome --port 9222
```

### Interacting with Sessions

```bash
# Stream stdio (for redirect sessions)
telepipe exec --id myapp

# Check status
telepipe info --id myapp
```

### Ending Sessions

```bash
# Stop a redirected process (kills the process)
telepipe stop --id myapp

# Disconnect from external service (leaves service running)
telepipe disconnect --id chrome
```

### Info Output Format

```
mode	redirect
host	127.0.0.1
pid	12345
supervisor	12346
stdin	49152
stdout	49153
stderr	49154
fdin	3
fdout	4
fderr	5
```

---

## Protocol Quick Reference

### CDP (Chrome DevTools Protocol)

```javascript
// Essential methods you already know:
Page.navigate({url})           // Go to URL
Page.captureScreenshot()       // Screenshot
DOM.getDocument()              // Get DOM tree
DOM.querySelector({selector})  // Find element
Runtime.evaluate({expression}) // Run JS
Network.enable()               // Monitor network
Console.enable()               // Capture console
```

### SQL (PostgreSQL)

```sql
-- Essential queries you already know:
\dt                           -- List tables
\d tablename                  -- Describe table
SELECT * FROM ... LIMIT 10    -- Sample data
EXPLAIN ANALYZE SELECT ...    -- Query plan
BEGIN; ...; ROLLBACK;         -- Safe testing
```

### Redis

```redis
# Essential commands you already know:
KEYS *                        # List keys
GET key                       # Get value
SET key value                 # Set value
HGETALL hash                  # Get hash
LPUSH/RPOP list value         # Queue operations
SUBSCRIBE channel             # Pub/sub
```

---

## Error Codes

If something goes wrong, Telepipe tells you why:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 21 | FD exhaustion (E-ALLOC-FD) |
| 22 | Port exhaustion (E-ALLOC-PORT) |
| 61 | Session not found (E-DICT-MISSING) |
| 64 | Session already exists (E-DICT-CONFLICT) |
| 74 | Invalid host |
| 75 | Invalid FD |
| 76 | Invalid port |

---

## Agent Integration Rules

These are the rules you MUST follow when using Telepipe:

1. **Use `--id` consistently** — Every operation requires a mnemonic session identifier
2. **Never guess ports** — Always query `telepipe info` to get port/FD information
3. **Treat Telepipe output as authoritative** — The info command tells the truth
4. **Handle error codes** — Use the error codes above to recover gracefully
5. **Never assume session state** — Always query first before sending commands
6. **Let Telepipe handle recovery** — Don't implement your own recovery logic

---

## Recommended Script Flow

### Step 1: Ensure session exists
```bash
if ! telepipe info --id foo >/dev/null 2>&1; then
    echo "Session not found, creating..."
    telepipe redirect --id foo -- python3 -i
fi
```

### Step 2: Send commands
```bash
echo "some command" | telepipe exec --id foo
```

### Step 3: Capture output
```bash
telepipe exec --id foo > out.txt 2> err.txt
```

### Step 4: Stop when complete
```bash
telepipe stop --id foo
```

This pattern is universal across all Telepipe use cases.

---

## Behavior Guarantees

Telepipe provides these guarantees specifically for agent compatibility:

### Deterministic CLI
- No spinners or progress bars
- No interactive prompts
- No random output noise
- Stable newline-only formatting
- Predictable tab-separated fields

### Machine-Friendly Error Codes
Every failure is recoverable by:
- Fixing input parameters
- Killing the session
- Recreating the session
- Reconnecting

### Separated Lifetimes
The CLI command exits immediately after setup. The supervisor continues in the background.
You can spawn sessions, disconnect, and reconnect later without losing state.

### Crash-Safe Recovery
If Telepipe crashes, it repairs itself automatically on next invocation.
You never need to repair Telepipe — it handles recovery internally.

---

## Multi-Agent Patterns

### Pattern: Team of Agents Sharing Sessions
Multiple AI agents can:
- Share the same redirect session
- Take turns sending commands (one at a time)
- Coordinate via the session dictionary

### Pattern: Agent → REPL → UI
Agents can inject code into:
- Browsers (via CDP)
- Debuggers (via DAP)
- SQL sessions
- Build systems

### Pattern: Agent-Driven File System
Workers create files via Telepipe sessions, then other agents read them.

---

## Reference

- [**FAQ**](faq.md) - Frequently asked questions and troubleshooting

---

## The Golden Rule

```
┌─────────────────────────────────────────┐
│                                         │
│   Don't guess. Verify.                  │
│   Don't describe. Execute.              │
│   Don't hope it works. Prove it works.  │
│                                         │
└─────────────────────────────────────────┘
```

You have the knowledge. Telepipe gives you the connection.

**Now go build something and PROVE it works.**
