# Telepipe CLI Reference
### *Complete Command-Line Interface Specification for Telepipe Core*
### *Core Operations • Lifecycle Operations • Options • Output Rules*

This document defines the **full, authoritative Telepipe CLI**.
It mirrors the Telepipe Core Spec and the Core Operations document, but presents the interface in the form expected by end‑users and automated agents.

Telepipe has **three core operations** and **three lifecycle operations**.
All commands operate on a **session dictionary** indexed by mnemonic IDs.

---

# 1. Command Summary

## Core Operations
```
telepipe redirect --id <ID> [--host <HOST>] [--stdin-tcp-port <P>] [--stdout-tcp-port <P>] [--stderr-tcp-port <P>] [--fdin <FD>] [--fdout <FD>] [--fderr <FD>] -- <CMD> [ARGS...]

telepipe exec --id <ID>

telepipe connect --id <ID> --port <PORT> [--host <HOST>] [--no-reconnect]
```

## Lifecycle Operations
```
telepipe stop --id <ID>

telepipe disconnect --id <ID>

telepipe info --id <ID>
```

**Note:** Commands are organized by function. Core operations create or interact with sessions. Lifecycle operations manage or inspect sessions.

---

# 2. Core Operations

## 2.1 `telepipe redirect`
### *Spawn a process and redirect stdio into persistent TCP sockets.*

```
telepipe redirect --id <n> [options] -- <COMMAND> [ARGS...]
```

#### Required:
- `--id <n>`
  The mnemonic session identifier.

#### Optional FD/TCP overrides:
- `--host <HOST>`
  Defaults to `127.0.0.1`.

- `--stdin-tcp-port <PORT>`
  Override for stdin TCP port (skip automatic allocation).

- `--stdout-tcp-port <PORT>`
  Override for stdout TCP port (skip automatic allocation).

- `--stderr-tcp-port <PORT>`
  Override for stderr TCP port (skip automatic allocation).

- `--fdin <FD>`
- `--fdout <FD>`
- `--fderr <FD>`
  Manual numeric file descriptor overrides (skip automatic allocation).

**Note on overrides:** If provided, Telepipe uses them directly without scanning. If invalid (already in use), emits `E-CLI-INVALID-FD` or `E-CLI-INVALID-PORT`.

#### Command:
`-- <COMMAND> [ARGS...]`

Telepipe will:
1. Acquire locks (GLOBAL → SESSION → PORTALLOC)
2. Run recovery pipeline
3. Allocate missing FDs (or use provided overrides)
4. Allocate missing TCP ports (or use provided overrides)
5. Open persistent sockets
6. Spawn `<COMMAND>` with its standard streams redirected
7. Store the session in the dictionary
8. Release locks
9. Exit with code 0 (redirect command terminates; supervisor continues as background process)

#### Output:
**No output** by default.
Use `telepipe info --id <n>` to view session state.

---

## 2.2 `telepipe exec`
### *Pipe input to, and stream output from, a redirect or connect session.*

```
echo "message" | telepipe exec --id <n> 1>out 2>err
```

#### Required:
- `--id <n>`

Telepipe will:
1. Look up session by ID (brief shared read-lock)
2. Connect to its TCP ports
  - For redirect: stdin/stdout/stderr ports
  - For connect: single bidirectional protocol port
3. Forward user stdin → remote stdin
4. Forward remote stdout/stderr → user stdout/stderr
5. Exit when all streams close

`exec` does **not** spawn a process or modify dictionary.

**Note:** Telepipe supports unlimited serial exec calls (one completing before the next begins). Concurrent exec calls (simultaneous connections) to the same session return `E-EXEC-ALREADY-ACTIVE` error (exit 81). For multiplexing multiple consumers, use external tools like `tee` or spawn separate sessions per worker.

---

## 2.3 `telepipe connect`
### *Attach Telepipe to an existing external TCP service.*

```
telepipe connect --id <n> --port 5432 [--host 127.0.0.1]
```

#### Required:
- `--id <n>`
- `--port <PORT>`

#### Optional:
- `--host <HOST>` (default: `127.0.0.1`)
- `--no-reconnect`
  If provided, Telepipe exits immediately when the connection drops. Otherwise, attempts reconnect indefinitely.

Telepipe will:
1. Acquire locks (GLOBAL → SESSION)
2. Run recovery pipeline
3. Allocate one FD
4. Open a persistent TCP connection to the external service
5. Store a connect-mode session
6. Spawn supervisor
7. Release locks
8. Exit with code 0 (connect command terminates; supervisor continues as background process)

By itself, `connect` does not stream I/O;
`exec` is used to interact with the session.

**Note:** For WebSocket-based protocols like CDP, use `redirect` with `websocat`:
```bash
WS_URL=$(curl -s http://127.0.0.1:9222/json | jq -r '.[0].webSocketDebuggerUrl')
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

---

# 3. Lifecycle Operations

## 3.1 `telepipe stop`
### *Terminate a redirect-mode child process.*

```
telepipe stop --id <n>
```

Effects:
- Sends SIGTERM/SIGKILL to child
- Supervisor exits with child's exit status
- Session record removed from dictionary

Applies only to **redirect** sessions.

---

## 3.2 `telepipe disconnect`
### *Terminate a connect-mode supervisor.*

```
telepipe disconnect --id <n>
```

Effects:
- Terminates the supervisor
- External service remains alive (unaffected)
- Session record removed from dictionary

Applies only to **connect** sessions.

---

## 3.3 `telepipe info`
### *Print complete session state and exit immediately.*

```
telepipe info --id <n>
```

This operation:
1. Performs a session lookup (brief shared read-lock)
2. Prints all session state (tab-separated)
3. **Exits immediately** (never blocks)
4. Returns exit code 0 on success

### Output format for REDIRECT mode:
```
mode        redirect
host        127.0.0.1
pid         <child-pid>
supervisor  <supervisor-pid>
stdin       <port>
stdout      <port>
stderr      <port>
fdin        <fd-number>
fdout       <fd-number>
fderr       <fd-number>
```

### Output format for CONNECT mode:
```
mode        connect
host        <host>
port        <protocol-port>
supervisor  <supervisor-pid>
connected   yes|no
reconnect   enabled|disabled
fd          <fd-number>
```

**Parsing rules:**
- Fields separated by single TAB character (`\t`)
- Labels are lowercase
- Order is stable across versions
- Suitable for shell scripting: `telepipe info --id foo | grep stdin | cut -f2`

**Performance:** Info completes in <100ms even with large dictionaries (reads only one entry).

---

# 4. Session Identifier Rules

- IDs must be unique within the dictionary
- Alphanumeric characters plus `-` and `_` are allowed
- IDs are case-sensitive
- IDs map 1:1 to entries in the session dictionary
- Creating a session with existing ID emits `E-DICT-CONFLICT`

Examples:

```
--id psql-sf
--id chrome-cdp
--id build-logs
```

---

# 5. Return Codes

Telepipe uses standard exit semantics:

- `redirect` → exits with code 0 after setup (supervisor continues in background)
- `exec` → exit 0 on success, nonzero on connection/session errors
- `connect` → exits with code 0 after setup (supervisor continues in background)
- `stop` → 0 on success
- `disconnect` → 0 on success
- `info` → 0 on success

**Critical note:** The redirect/connect *commands* exit after setup. The *supervisor* (background process) continues running and exits with child's exit code (redirect mode only).

---

# 6. Environment Variables

Optional:

- `TELEPIPE_HOST` — Default host (overridden by `--host`)
- `TELEPIPE_PORT_BASE` — Starting port for allocation
- `TELEPIPE_FD_BASE` — Starting FD for allocation
- `TELEPIPE_HOME` — Overrides default `$HOME/.telepipe` directory
- `TELEPIPE_DEBUG` — Enables verbose internal logging
- `TELEPIPE_TRACE_LOCKS` — Shows lock acquisition/release timing

Environment variables have lower priority than explicit CLI flags.

---

# 6.1 Files

Telepipe stores all session state in:

```
$HOME/.telepipe/sessions/
```

This directory contains session dictionary files and lockfiles. It is created automatically on first use.

---

# 7. Examples

## Redirect + Exec
```bash
# Create redirect session
telepipe redirect --id psql-sf -- psql

# Pipe command to session
echo "select now();" | telepipe exec --id psql-sf >out 2>err

# View session state
telepipe info --id psql-sf

# Clean up
telepipe stop --id psql-sf
```

## Connect + Exec (Database)
```bash
# Attach to PostgreSQL
telepipe connect --id db --port 5432

# Send commands
echo "SELECT 1;" | telepipe exec --id db

# View connection state
telepipe info --id db

# Disconnect
telepipe disconnect --id db
```

## CDP via Redirect + websocat
```bash
# Get WebSocket URL and create session
WS_URL=$(curl -s http://127.0.0.1:9222/json | jq -r '.[0].webSocketDebuggerUrl')
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"

# Send CDP command
echo '{"id":1,"method":"Runtime.enable"}' | telepipe exec --id chrome

# View session state
telepipe info --id chrome

# Stop the session
telepipe stop --id chrome
```

## Checking Session State
```bash
# Get session info
telepipe info --id psql-sf

# Parse specific field
PORT=$(telepipe info --id psql-sf | grep stdin | cut -f2)
echo "STDIN port: $PORT"

# Check if connected (connect mode)
if [ "$(telepipe info --id db | grep connected | cut -f2)" = "yes" ]; then
    echo "Database is connected"
fi
```

## Using Port Overrides
```bash
# Force specific ports
telepipe redirect --id custom \
  --stdin-tcp-port 50000 \
  --stdout-tcp-port 50001 \
  --stderr-tcp-port 50002 \
  -- python3 -i
```

---

# 8. Reserved Command Names

The following command names are explicitly reserved for future use and must NOT be implemented without a specification update:

- `status` (reserved — use `info` instead)
- `attach` (reserved — use `connect` instead)
- `spawn` (reserved — use `redirect` instead)
- `inspect`
- `trace`
- `forward`
- `mirror`
- `record`

These names were reserved to:
- Match mainstream Unix vocabulary
- Avoid tool-specific metaphors
- Align with historic process model semantics
- Allow Telepipe to scale into future modes without collisions

---

# 9. Debug Flags (Future)

Telepipe may provide optional diagnostic flags for developers:

- `telepipe --debug` — Verbose diagnostic output
- `telepipe --trace-fd` — Trace file descriptor operations
- `telepipe --trace-tcp` — Trace TCP connection operations

These are disabled by default and not required for normal operation.

---

# 10. Summary

The Telepipe CLI provides:

**Core Operations:**
- **redirect** — spawn process with redirected I/O to TCP
- **exec** — pipe and stream to existing session
- **connect** — attach to external protocol port

**Lifecycle Operations:**
- **stop** — terminate redirect session
- **disconnect** — terminate connect session
- **info** — show complete session state and exit

This is the complete, deterministic, cross‑platform command interface for Telepipe Core.
