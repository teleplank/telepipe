# Telepipe Core Specification
### *Foundational Architecture • Cross‑Platform Guarantees • Deterministic Behaviors*

This document defines the **Telepipe Core**, the minimal, stable, cross‑platform behavior model implemented by the Telepipe runtime.
It is written in **neutral engineering vocabulary**, using only mainstream OS/process terms.
It contains **no metaphors**, **no legacy modes**, and **no ambiguous verbs**.

Telepipe Core consists of **three core operations** and **three lifecycle operations**.

---

# 0. Goals and Guarantees

Telepipe Core enables any user or automated agent to:

- start a process with its file descriptors redirected into TCP connections
- interact with that process via stdin/stdout/stderr over durable TCP streams
- attach to an existing TCP service (e.g., databases, Redis)
- bridge WebSocket protocols (e.g., CDP, Node Inspector) via redirect mode
- maintain persistent connections without reconnect overhead
- send an arbitrary number of commands
- stream an arbitrary amount of output
- stop spawned processes or disconnect attached supervisors
- run identically on Linux, macOS, and Windows Git Bash/MSYS2

Telepipe Core is built on:

- numeric file descriptors
- persistent TCP sockets
- POSIX redirection
- netcat (`nc`)
- deterministic ephemeral TCP allocation
- deterministic FD allocation
- a session dictionary keyed by mnemonic IDs

Telepipe Core does **not** require:

- named pipes (FIFOs)
- `/dev/tcp` support in spawned processes
- PTYs
- OS‑specific APIs
- JSON output
- process groups
- privileges beyond those needed to run the underlying process

---

# 1. Definitions

### **Session**
A record containing:
- three TCP ports (stdin, stdout, stderr) *or one for connect-mode*
- three numeric file descriptors (stdin, stdout, stderr) *or one for connect-mode*
- the child process PID (in redirect-mode)
- the supervising process PID (in connect-mode)
- the mnemonic ID assigned by the user
  Stored in a persistent dictionary.

### **Numeric File Descriptor (FD)**
An integer ≥3 allocated by scanning the current process' FD table.

### **Ephemeral TCP Port**
A port between 49152–65535 inclusive, selected by probing availability.

### **Supervisor**
A Telepipe instance that manages:
- the redirected file descriptors of a spawned child process, or
- the persistent TCP connection to an external port (connect-mode).

### **Client**
Any Telepipe instance executed later with the same mnemonic ID.

---

# 2. Core Operations

Telepipe Core has **three core operations** that create or interact with sessions.
They are mutually independent and combine to form all Telepipe functionality.

---

# 2.1 Operation A: `redirect`
### *Spawn a process and redirect its stdio to persistent TCP sockets.*

**Note:** "Redirect" is the canonical name because Telepipe's primary function is redirecting file descriptors to TCP, not process management. The spawn operation is merely a step toward achieving redirection.

This behavior:

1. **finds three available numeric file descriptors**
2. **finds three available ephemeral TCP ports**
3. **opens three persistent TCP sockets bound to those FDs**
4. **spawns the target process with redirected stdio**
5. **stores the session under a user-provided mnemonic ID**
6. **runs as a supervisor until the child exits or is stopped**

### Responsibilities

#### 2.1.1 Allocate FDs
Sequential scan from 3–256 until three unused FDs are found.

#### 2.1.2 Allocate ports
Sequential scan from 49152–65535 until three unused ports are found.

#### 2.1.3 Bind FDs to TCP
```
exec {FD_IN}<>/dev/tcp/HOST/PORT_IN
exec {FD_OUT}<>/dev/tcp/HOST/PORT_OUT
exec {FD_ERR}<>/dev/tcp/HOST/PORT_ERR
```

#### 2.1.4 Spawn process
```
child <&FD_IN >&FD_OUT 2>&FD_ERR &
```

**Why spawn?** Telepipe must spawn the child itself (not attach to existing PIDs) to ensure:
- Cross-platform compatibility (attaching requires platform-specific APIs)
- No privilege escalation (ptrace/debugging privileges not needed)
- Clean FD inheritance from the start

#### 2.1.5 Persist session record
The session dictionary records:
- mnemonic ID
- child PID
- FDs
- ports
- supervisor PID

#### 2.1.6 Runtime lifecycle
- supervisor remains running while the child is alive
- when child exits, supervisor exits with *the same exit code*
- if supervisor is killed, child continues running (standard Unix semantics)

### Redirect mode output
By default: **no output**.
Complementary info mode prints session information (see §3.3).

---

# 2.2 Operation B: `exec`
### *Pipe and stream to a previously created redirect session.*

This behavior:

1. locates an existing session by mnemonic
2. connects the client's stdin to the session's stdin TCP socket
3. connects the session's stdout/stderr TCP sockets to the client's stdout/stderr
4. forwards data until stdin closes and/or output streams end
5. exits when all connected streams close

### Responsibilities

#### 2.2.1 Session lookup
Fail if session does not exist.

#### 2.2.2 Establish TCP connections
- stdin → PORT_IN
- stdout ← PORT_OUT
- stderr ← PORT_ERR

#### 2.2.3 Stream forwarding
Identical to:
```
nc ... | nc ...
```
but stateful and mnemonic‑based.

#### 2.2.4 Return values
`exec` exits when:
- stdin closes, and
- remote stdout/stderr reach EOF.

---

# 2.3 Operation C: `connect`
### *Attach a supervisor to an existing external TCP service.*

Used for:
- Database connections (PostgreSQL, MySQL, Redis)
- Raw TCP protocol endpoints
- Custom debugging ports with TCP interfaces

**Important:** For WebSocket-based protocols (Chrome DevTools Protocol, Node Inspector, etc.), use `redirect` mode with a WebSocket bridge like `websocat`. See section 2.4.

This behavior:

1. **finds one free FD** (connect uses a single bidirectional connection)
2. **opens a persistent TCP connection to the external service**
3. **records a connect‑mode session**
4. **streams until disconnected**

### Responsibilities

#### 2.3.1 Allocate FD
Find free numeric FD ≥3.

#### 2.3.2 Bind FD to external TCP service
```
exec {FD_CONN}<>/dev/tcp/HOST/PORT
```

#### 2.3.3 Persist session record
Contains:
- mnemonic
- host:port
- FD
- supervisor PID

#### 2.3.4 Lifecycle
Runs indefinitely until:
- user disconnects, or
- supervisor is killed.

---

# 2.4 WebSocket Protocols (CDP, Node Inspector)

### The WebSocket Requirement

Chrome DevTools Protocol (CDP) and similar protocols use **WebSocket**, not raw TCP. While these services expose a TCP port (e.g., 9222), the actual communication requires WebSocket framing.

### Correct Pattern: redirect + websocat

```bash
# Get WebSocket URL from Chrome's JSON endpoint
WS_URL=$(curl -s http://127.0.0.1:9222/json | jq -r '.[0].webSocketDebuggerUrl')

# Use redirect mode with websocat as the child process
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"

# Send CDP commands
echo '{"id":1,"method":"Runtime.evaluate","params":{"expression":"1+1"}}' | \
  telepipe exec --id chrome
```

### Why This Works

1. **websocat** handles WebSocket protocol framing
2. **redirect mode** provides persistent stdin/stdout/stderr over TCP
3. **Telepipe** manages the session lifecycle
4. Commands flow: stdin → websocat → WebSocket → Chrome → WebSocket → websocat → stdout

### Summary

| Protocol Type | Telepipe Mode | Example |
|---------------|---------------|---------|
| Raw TCP | `connect` | PostgreSQL, MySQL, Redis |
| WebSocket | `redirect` + websocat | CDP, Node Inspector, DAP |

---

# 2.5 Why Different FD Counts for Redirect vs Connect?

### Redirect Mode: 3 FDs (stdin, stdout, stderr)

**Requirement:** Child processes have THREE independent streams that must remain separate:
- **stdin** — application reads commands/input
- **stdout** — application writes normal output
- **stderr** — application writes error messages/logs

**Why not multiplex?** Multiplexing would require Telepipe to parse/frame the data, which violates the binary-transparency principle. Many applications send binary data, control characters, or protocol-specific framing that Telepipe must not interpret.

**Example:** A PostgreSQL client streams binary protocol on stdout while sending error messages to stderr. These cannot be combined.

### Connect Mode: 1 FD (bidirectional protocol channel)

**Requirement:** External protocol endpoints already provide their own multiplexing.

**Why only 1 FD?** Services like PostgreSQL, MySQL, and Redis expose a single TCP port that handles bidirectional communication with protocol-level framing.

Telepipe simply attaches to this existing channel without interpretation.

### Summary

| Mode | FD Count | Reason |
|------|----------|--------|
| redirect | 3 | Child has independent stdin/stdout/stderr |
| connect | 1 | External protocol already multiplexed |

This design maintains binary transparency while supporting both process redirection and protocol attachment.

---

# 3. Lifecycle Operations

Three lifecycle operations manage or inspect existing sessions.

---

# 3.1 Operation D: `stop`
### *Stop the child process of a redirect session.*

```
telepipe stop --id NAME
```

Effects:
- sends SIGTERM then SIGKILL to child
- supervisor exits with same exit code
- session record is removed

---

# 3.2 Operation E: `disconnect`
### *Terminate a supervisor associated with a connect session.*

```
telepipe disconnect --id NAME
```

Effects:
- stops supervisor
- underlying external service remains alive
- session record is removed

---

# 3.3 Operation F: `info`
### *Print the session's complete state in tab‑separated format and exit immediately.*

```
telepipe info --id NAME
```

**Critical guarantee:** Info mode is read-only and **exits immediately** after printing. It never blocks, never modifies state, and always returns exit code 0 on success.

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

**Rules:**
1. Fields are separated by single TAB character (`\t`)
2. All fields are required (no optional fields)
3. Field order is stable and must not change
4. Output is suitable for parsing by scripts (`cut`, `awk`, etc.)
5. Info outputs ALL state relevant to the session's mode

### Exit Guarantee

**INVARIANT:** Info mode exits immediately after printing output.

Specifically:
1. Reads session dictionary entry
2. Formats output as tab-separated fields
3. Prints to stdout
4. Exits with code 0

**Timing:** Info should complete in <100ms even with large dictionaries (it reads only one entry, not the entire dictionary).

**Blocking:** Info never waits for:
- Supervisor response
- Child process state
- Network I/O
- Lock contention (uses brief shared read-lock only)

**Purpose:** Provides instant session state feedback without blocking on long-running operations.

### Why Tab-Separated Output?

Telepipe uses TSV (tab-separated values) instead of JSON because:

1. **Unix tradition** — Tools like `cut`, `awk`, `grep` expect tab-delimited fields
2. **Simplicity** — No escaping, no nested structures, no parsing ambiguity
3. **Readability** — Columns align naturally in terminal output
4. **Stability** — Field order never changes; safe for scripting
5. **Performance** — No JSON parser needed; instant field extraction

**Example parsing:**
```bash
# Get stdin port
PORT=$(telepipe info --id foo | grep stdin | cut -f2)

# Get all ports
telepipe info --id foo | grep -E '(stdin|stdout|stderr)' | cut -f2

# Check if connected
if [ "$(telepipe info --id cdp | grep connected | cut -f2)" = "yes" ]; then
    echo "Connected"
fi
```

This follows the Unix philosophy: "Write programs that do one thing well and work together."

---

# 4. Cross‑Platform Process Lifecycle Guarantees

### 4.1 Child survival on parent death
On standard Unix systems:
- when the supervisor dies, the child continues running
- child is adopted by PID 1

This is required behavior.

### 4.2 TCP cleanup
When:
- an FD closes
- OR a process exits

…the OS automatically releases:
- the file descriptor
- the TCP socket
- associated kernel resources

No manual cleanup needed.

### 4.3 No named pipes
Telepipe Core uses **no FIFOs**, ensuring full operability on Windows Git Bash/MSYS2.

### 4.4 Platform-Specific Considerations

**Git Bash / MSYS2 Limitations:**
- `/dev/tcp` is implemented purely in Bash, not by the OS
- `lsof` is not always available
- Job control semantics differ slightly
- PID reuse happens faster than on Linux/macOS

**Mitigation:**
- Use TCP connection tests instead of `lsof` for existence checks
- Avoid assumptions about process group behavior

**macOS older shells (bash 3.x):**
- Limited brace expansion
- No modern `exec {fd}>...` syntax

Telepipe Core requires a modern shell; this is documented explicitly.

---

# 5. Deterministic Allocation Rules

### 5.1 TCP ports
Scan from **49152–65535**, sequentially.

**Why 49152-65535?**

This is the IANA-defined ephemeral port range used industry-wide by:
- Chrome DevTools Protocol (CDP)
- Node.js inspector
- Docker internal ports
- VSCode debug adapters
- Kubernetes port-forwards
- Webpack dev servers
- All major development tools

This is the "don't conflict with system services" zone. Telepipe follows industry standard practice.

### 5.2 Numeric file descriptors
Scan from **3–255**, sequentially.

FD 256 is reserved for future use.

### 5.3 Failure rules
If:
- 3 ports cannot be allocated → redirect fails
- 3 FDs cannot be allocated → redirect fails
- 1 port/FD cannot be allocated for connect → connect fails

Errors must be:
- clear
- synchronous
- non‑interactive

### 5.4 Port Determinism Across Runs

If multiple Telepipe instances run simultaneously, port allocation must be:
- Monotonic
- Collision-safe
- Race-free

Telepipe must avoid TOCTOU race conditions when two processes pick ports at the same time.

**Solution:** Bind immediately after selecting. If bind fails → retry next port.

### 5.5 FD Determinism

FD allocation must never:
- Skip numbers
- Reuse FDs still in use
- Attach to FDs inadvertently inherited from parents

All spawns must use `close-on-exec` where appropriate to prevent data leakage across sessions.

---

# 6. Session Dictionary

A session record must contain:

```
id: string
mode: redirect|connect
supervisor_pid: int
child_pid: int|null
host: string
stdin_port: int|null
stdout_port: int|null
stderr_port: int|null
stdin_fd: int|null
stdout_fd: int|null
stderr_fd: int|null
connect_fd: int|null
connect_port: int|null
```

The dictionary must be writable and recoverable.

### 6.1 Corruption Avoidance

The session dictionary must be:
- Atomic on writes
- Locked per update
- fsync-safe
- Validated on read

If malformed entries exist:
- Telepipe must refuse to run
- Telepipe must print a diagnostic with repair guidance

### 6.2 Crash Resilience

When Telepipe dies abruptly:
- Dictionary may contain stale PIDs
- Stale sessions must be automatically cleaned by a recovery pass

**Recovery rules:**
- If supervisor PID does not exist → remove entry
- If redirect child PID exists but TCP ports are closed → remove entry
- If connect FD cannot reconnect → remove entry

---

# 7. Security & Locality

- Telepipe Core assumes **local‑only** semantics unless `--host` is explicitly provided.
- By default, host = `127.0.0.1`
- No external connections are allowed unless explicitly configured.

---

# 8. Command Summary Table

| Command | Purpose | Type |
|---------|---------|------|
| `redirect` | Spawn process with redirected I/O | Core |
| `exec` | Pipe and stream to session | Core |
| `connect` | Attach to external protocol port | Core |
| `stop` | Terminate redirect session | Lifecycle |
| `disconnect` | Terminate connect session | Lifecycle |
| `info` | Show session state | Lifecycle |

**Core operations** create or interact with sessions.
**Lifecycle operations** manage or inspect existing sessions.

---

# 9. Summary

Telepipe Core is defined strictly by:

- **redirect** — spawn + redirect stdio into TCP
- **exec** — send and stream data from a redirect session
- **connect** — attach to an existing TCP port
- **stop** — stop spawned processes
- **disconnect** — stop supervisors for connect sessions
- **info** — print session information

All functionality derives from:
- numeric file descriptors
- persistent TCP sockets
- stable allocation rules
- deterministic lifecycle
- strict cross-platform behavior
