# Telepipe Command Reference

Complete reference for all Telepipe commands. Every command is deterministic, cross-platform, and AI-friendly.

---

## Command Overview

### Core Operations
| Command | Purpose |
|---------|---------|
| `telepipe redirect` | Spawn process with redirected I/O to TCP |
| `telepipe exec` | Send input to and stream output from a session |
| `telepipe connect` | Attach to external TCP service |

### Lifecycle Operations
| Command | Purpose |
|---------|---------|
| `telepipe stop` | Terminate redirect session (kills process) |
| `telepipe disconnect` | Terminate connect session (leaves service running) |
| `telepipe info` | Show session state |

---

## telepipe redirect

**Spawn a process and redirect its stdio to persistent TCP sockets.**

### Syntax
```bash
telepipe redirect --id <session-id> [options] -- <command> [args...]
```

### Required Arguments
- `--id <session-id>` - Unique session identifier (alphanumeric, `-`, `_`)

### Optional Arguments
- `--host <host>` - Bind address (default: `127.0.0.1`)
- `--stdin-tcp-port <port>` - Override stdin TCP port
- `--stdout-tcp-port <port>` - Override stdout TCP port
- `--stderr-tcp-port <port>` - Override stderr TCP port
- `--fdin <fd>` - Override stdin file descriptor (3-255)
- `--fdout <fd>` - Override stdout file descriptor (3-255)
- `--fderr <fd>` - Override stderr file descriptor (3-255)

### Examples

**Start a Node.js server:**
```bash
telepipe redirect --id myapp -- node server.js
```

**Start PostgreSQL client:**
```bash
telepipe redirect --id psql -- psql -U postgres
```

**Start Python REPL:**
```bash
telepipe redirect --id python -- python3 -i
```

**Chrome DevTools via websocat bridge:**
```bash
# First get the WebSocket URL
WS_URL=$(curl -s http://127.0.0.1:9222/json/version | grep -o 'ws://[^"]*')

# Then connect via websocat
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

**With port overrides:**
```bash
telepipe redirect --id custom \
  --stdin-tcp-port 50000 \
  --stdout-tcp-port 50001 \
  --stderr-tcp-port 50002 \
  -- python3 -i
```

### Exit Codes
| Code | Error | Description |
|------|-------|-------------|
| 0 | Success | Session created, supervisor running |
| 21 | E-ALLOC-FD | No available file descriptors |
| 22 | E-ALLOC-PORT | No available TCP ports |
| 51 | E-PROC-SPAWN | Failed to spawn child process |
| 64 | E-DICT-CONFLICT | Session ID already exists |
| 71 | E-CLI-ARGS | Invalid arguments |

### Notes
- The `redirect` command exits immediately after setup (exit 0)
- The supervisor continues running in the background
- Use `telepipe info --id <session-id>` to view session state
- Use `telepipe stop --id <session-id>` to terminate

---

## telepipe exec

**Pipe input to and stream output from a session.**

### Syntax
```bash
echo "input" | telepipe exec --id <session-id>
```

### Required Arguments
- `--id <session-id>` - Session to interact with

### Examples

**Send SQL query:**
```bash
echo "SELECT version();" | telepipe exec --id psql
```

**Send CDP command:**
```bash
echo '{"id":1,"method":"Browser.getVersion"}' | telepipe exec --id chrome
```

**Interactive mode (reads from stdin):**
```bash
telepipe exec --id python
>>> print("hello")
hello
```

**Capture output:**
```bash
echo "SELECT * FROM users;" | telepipe exec --id db > results.txt 2> errors.txt
```

**Navigate and screenshot with CDP:**
```bash
# Navigate
echo '{"id":1,"method":"Page.navigate","params":{"url":"https://example.com"}}' | telepipe exec --id chrome

# Screenshot
echo '{"id":2,"method":"Page.captureScreenshot"}' | telepipe exec --id chrome
```

### Exit Codes
| Code | Error | Description |
|------|-------|-------------|
| 0 | Success | Data streamed successfully |
| 41 | E-TCP-CONNECT | Cannot connect to session ports |
| 61 | E-DICT-MISSING | Session not found |
| 81 | E-EXEC-ALREADY-ACTIVE | Another exec is active (expected!) |

### One-at-a-Time Execution

**Important:** Only ONE exec can be active per session at a time.

**Serial exec (supported):**
```bash
echo "cmd1" | telepipe exec --id session  # Completes
echo "cmd2" | telepipe exec --id session  # Completes
echo "cmd3" | telepipe exec --id session  # Completes
# Unlimited serial calls!
```

**Concurrent exec (not supported):**
```bash
echo "cmd1" | telepipe exec --id session &  # Running
echo "cmd2" | telepipe exec --id session    # E-EXEC-ALREADY-ACTIVE (81)
```

**For concurrent workers, use separate sessions:**
```bash
telepipe redirect --id worker1 -- websocat ...
telepipe redirect --id worker2 -- websocat ...

echo "cmd1" | telepipe exec --id worker1 &
echo "cmd2" | telepipe exec --id worker2 &
```

---

## telepipe connect

**Attach to an existing external TCP service.**

### Syntax
```bash
telepipe connect --id <session-id> --port <port> [options]
```

### Required Arguments
- `--id <session-id>` - Unique session identifier
- `--port <port>` - TCP port of the service

### Optional Arguments
- `--host <host>` - Service host (default: `127.0.0.1`)
- `--no-reconnect` - Exit immediately if connection drops

### Examples

**Connect to PostgreSQL:**
```bash
telepipe connect --id db --port 5432
```

**Connect to Redis:**
```bash
telepipe connect --id cache --port 6379
```

**Connect to MySQL:**
```bash
telepipe connect --id mysql --port 3306
```

**Connect with no reconnect:**
```bash
telepipe connect --id db --port 5432 --no-reconnect
```

### NOT for WebSocket Protocols!

Chrome DevTools Protocol (CDP) uses WebSockets, not raw TCP. **Do NOT use:**
```bash
# WRONG - will not work!
telepipe connect --id chrome --port 9222
```

**Instead, use websocat bridge:**
```bash
# CORRECT - use redirect with websocat
WS_URL=$(curl -s http://127.0.0.1:9222/json/version | grep -o 'ws://[^"]*')
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

### Exit Codes
| Code | Error | Description |
|------|-------|-------------|
| 0 | Success | Connected, supervisor running |
| 22 | E-ALLOC-PORT | No available TCP ports |
| 41 | E-TCP-CONNECT | Cannot connect to service |
| 64 | E-DICT-CONFLICT | Session ID already exists |
| 76 | E-CLI-INVALID-PORT | Invalid port number |

### Notes
- The `connect` command exits immediately after setup (exit 0)
- The supervisor continues running in the background
- By default, reconnects automatically if connection drops
- Use `--no-reconnect` to exit on connection loss

---

## telepipe info

**Show complete session state and exit immediately.**

### Syntax
```bash
telepipe info --id <session-id>
```

### Required Arguments
- `--id <session-id>` - Session to inspect

### Output Format (Redirect Mode)
```
mode        redirect
host        127.0.0.1
pid         12345
supervisor  12346
stdin       49152
stdout      49153
stderr      49154
fdin        3
fdout       4
fderr       5
```

### Output Format (Connect Mode)
```
mode        connect
host        127.0.0.1
port        5432
supervisor  12347
connected   yes
reconnect   enabled
fd          3
```

### Examples

**Get session info:**
```bash
telepipe info --id myapp
```

**Parse specific field:**
```bash
PORT=$(telepipe info --id myapp | grep stdin | cut -f2)
echo "STDIN port: $PORT"
```

**Check if connected:**
```bash
if [ "$(telepipe info --id db | grep connected | cut -f2)" = "yes" ]; then
    echo "Database is connected"
fi
```

**Get all ports:**
```bash
telepipe info --id myapp | grep -E '(stdin|stdout|stderr)' | cut -f2
```

### Exit Codes
| Code | Error | Description |
|------|-------|-------------|
| 0 | Success | Info displayed |
| 61 | E-DICT-MISSING | Session not found |
| 68 | E-INFO-READ | Cannot read session info |

### Notes
- Info is read-only - never modifies state
- Exits immediately - never blocks
- Completes in <100ms
- Tab-separated output for easy parsing

---

## telepipe stop

**Terminate a redirect-mode session and its child process.**

### Syntax
```bash
telepipe stop --id <session-id>
```

### Required Arguments
- `--id <session-id>` - Session to stop

### Examples

**Stop a session:**
```bash
telepipe stop --id myapp
```

### Effects
1. Sends SIGTERM to child process
2. If child doesn't exit, sends SIGKILL
3. Supervisor exits with child's exit status
4. Session record removed from dictionary

### Exit Codes
| Code | Error | Description |
|------|-------|-------------|
| 0 | Success | Session stopped |
| 61 | E-DICT-MISSING | Session not found |

### Notes
- Only for **redirect** mode sessions
- For connect mode, use `telepipe disconnect`
- Child process is terminated

---

## telepipe disconnect

**Terminate a connect-mode supervisor.**

### Syntax
```bash
telepipe disconnect --id <session-id>
```

### Required Arguments
- `--id <session-id>` - Session to disconnect

### Examples

**Disconnect from database:**
```bash
telepipe disconnect --id db
```

### Effects
1. Terminates the supervisor
2. External service **remains alive** (unaffected)
3. Session record removed from dictionary

### Exit Codes
| Code | Error | Description |
|------|-------|-------------|
| 0 | Success | Session disconnected |
| 61 | E-DICT-MISSING | Session not found |

### Notes
- Only for **connect** mode sessions
- For redirect mode, use `telepipe stop`
- External service continues running

---

## Session Identifier Rules

Session IDs must follow these rules:

- **Unique** within the dictionary
- **Allowed characters:** Alphanumeric, `-`, `_`
- **Case-sensitive:** `myApp` and `myapp` are different
- **No spaces or special characters**

### Valid Examples
```bash
--id myapp
--id chrome-cdp
--id build_logs
--id worker1
--id psql-sf
```

### Invalid Examples
```bash
--id "my app"      # Contains space
--id my@app        # Contains @
--id               # Empty
```

---

## Environment Variables

Optional environment variables:

| Variable | Purpose | Example |
|----------|---------|---------|
| `TELEPIPE_HOST` | Default host | `127.0.0.1` |
| `TELEPIPE_PORT_BASE` | Starting port for allocation | `49152` |
| `TELEPIPE_FD_BASE` | Starting FD for allocation | `3` |

CLI flags always override environment variables.

---

## Quick Reference

### Start sessions
```bash
# Process I/O redirect
telepipe redirect --id myapp -- node server.js

# Database connection (direct TCP)
telepipe connect --id db --port 5432

# Chrome CDP (WebSocket via websocat)
WS_URL=$(curl -s http://127.0.0.1:9222/json/version | grep -o 'ws://[^"]*')
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

### Interact with sessions
```bash
# Send command
echo "SELECT 1;" | telepipe exec --id db

# Get info
telepipe info --id myapp
```

### End sessions
```bash
# Stop redirect session (kills process)
telepipe stop --id myapp

# Disconnect connect session (leaves service running)
telepipe disconnect --id db
```
