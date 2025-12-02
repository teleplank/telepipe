# Architecture Deep Dive

Complete technical specification of Telepipe's internals. For those who want to understand how it really works.

---

## Supervisor Process

The supervisor is the heart of Telepipe - a long-lived process that manages I/O multiplexing and session lifecycle.

### Initialization Sequence

```
1. Parse CLI arguments
2. Acquire locks (GLOBAL → SESSION → PORTALLOC)
3. Run recovery pipeline
4. Allocate resources (FDs, ports)
5. Create session entry
6. Fork supervisor process
7. Release locks
8. CLI exits (supervisor continues)
```

### Supervisor Types

**Redirect Supervisor:**
- Spawns child process
- Manages stdin/stdout/stderr
- Monitors child PID
- Exits when child exits

**Connect Supervisor:**
- Connects to external service
- Manages single bidirectional stream
- Optionally reconnects on failure
- Runs until disconnected

---

## Session Data Structure

```rust
struct Session {
    id: String,                    // User-provided mnemonic
    mode: SessionMode,             // Redirect | Connect
    state: SessionState,           // Created | Ready | Busy | Terminated
    supervisor_pid: u32,           // Supervisor process ID

    // Redirect mode
    child_pid: Option<u32>,        // Child process ID
    stdin_port: Option<u16>,       // TCP port for stdin
    stdout_port: Option<u16>,      // TCP port for stdout
    stderr_port: Option<u16>,      // TCP port for stderr
    stdin_fd: Option<u8>,          // FD for stdin (3-255)
    stdout_fd: Option<u8>,         // FD for stdout
    stderr_fd: Option<u8>,         // FD for stderr

    // Connect mode
    connect_port: Option<u16>,     // External service port
    connect_host: String,          // External service host
    connect_fd: Option<u8>,        // FD for connection
    reconnect: bool,               // Auto-reconnect enabled
}

enum SessionMode {
    Redirect,
    Connect,
}

enum SessionState {
    Created,     // Session initialized
    Ready,       // Waiting for exec
    Busy,        // Exec in progress
    Terminated,  // Session ended
}
```

---

## State Machine

### State Transitions

```
     ┌─────────────────────────────────────────┐
     │                                         │
     ▼                                         │
 [Created] ──start──► [Ready] ◄──complete──► [Busy]
                        │                      │
                        │ stop/disconnect      │ error
                        ▼                      ▼
                   [Terminated] ◄──────────────┘
```

### Transition Rules

| From | Event | To | Action |
|------|-------|----|----|
| Created | start | Ready | Subprocess starts or connection established |
| Ready | exec | Busy | Begin data streaming |
| Busy | complete | Ready | Exec finished, ready for next |
| Ready | stop/disconnect | Terminated | Clean shutdown |
| Busy | error | Terminated | Error during exec |

### State Guards

**exec transition:**
- Session must be in Ready state
- No current exec active (enforced by supervisor)
- Session still exists in dictionary

**stop/disconnect transition:**
- Session must exist
- Mode must match operation (stop for redirect, disconnect for connect)

---

## Resource Allocation

### File Descriptor Allocation

**Algorithm:**
```
1. Start at FD 3 (0, 1, 2 are stdin/stdout/stderr)
2. Scan sequentially through 3-255
3. Check each FD with fcntl(fd, F_GETFD)
4. If returns -1 with EBADF, FD is available
5. Allocate 3 consecutive FDs for redirect, 1 for connect
6. Record in session
```

**Why 3-255?**
- 0, 1, 2 are reserved (stdin, stdout, stderr)
- 256+ may not be portable
- 253 available FDs = 84 redirect sessions max

**Cleanup:**
```
On session termination:
1. Close all allocated FDs
2. Kernel releases resources automatically
```

### TCP Port Allocation

**Algorithm:**
```
1. Start at random port in 49152-65535
2. Attempt to bind(port)
3. If EADDRINUSE, try next port
4. Wrap around if needed
5. Allocate 3 consecutive ports for redirect, 1 for connect
6. Record in session
```

**Why 49152-65535?**
- IANA ephemeral port range
- Avoids system services (0-1023)
- Avoids registered services (1024-49151)
- Industry standard for development tools

**Port lifecycle:**
```
Allocation → Bind → Listen → Accept → Stream → Close
                                            │
                                            ▼
                              Kernel releases after TIME_WAIT
```

---

## I/O Flow

### Redirect Mode Data Flow

```
┌─────────────────────────────────────────────────────────┐
│                     Supervisor                           │
│                                                          │
│  ┌──────────────────────────────────────────────────┐   │
│  │              Event Loop                           │   │
│  │                                                   │   │
│  │  TCP stdin port ──read──► write ──► child stdin  │   │
│  │  child stdout ──read──► write ──► TCP stdout port│   │
│  │  child stderr ──read──► write ──► TCP stderr port│   │
│  │                                                   │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### Connect Mode Data Flow

```
┌─────────────────────────────────────────────────────────┐
│                     Supervisor                           │
│                                                          │
│  ┌──────────────────────────────────────────────────┐   │
│  │              Event Loop                           │   │
│  │                                                   │   │
│  │  local inbound ──read──► write ──► remote        │   │
│  │  remote ──read──► write ──► local outbound       │   │
│  │                                                   │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### Binary Transparency

**Guarantee:** All data passes through unmodified.

```rust
// Simplified data copy loop
loop {
    let n = source.read(&mut buffer)?;
    if n == 0 { break; }
    dest.write_all(&buffer[..n])?;
}
```

No parsing. No transformation. No framing.

**Exception:** Status byte protocol for exec lock (see below).

---

## Concurrency Model

### Lock Hierarchy

```
GLOBAL_LOCK
    └── SESSION_LOCK(id)
            └── PORTALLOC_LOCK
```

**Rule:** Always acquire in this order. Never reverse.

### Lock Purposes

| Lock | Protects | Held During |
|------|----------|-------------|
| GLOBAL_LOCK | Dictionary scan, recovery, writes | Dictionary operations only |
| SESSION_LOCK(id) | Single session state | Session create/modify/delete |
| PORTALLOC_LOCK | Port allocation scan | Port binding only |

### Lock Timing

**Short holds:**
- GLOBAL_LOCK: <10ms typically
- SESSION_LOCK: <5ms typically
- PORTALLOC_LOCK: <50ms (port scan)

**Never held during:**
- Supervisor streaming
- Child process execution
- Network I/O
- Long waits

---

## One-at-a-Time Execution

### Why?

Telepipe enforces single exec per session to:
1. Prevent response mixing
2. Guarantee ordering
3. Simplify state management
4. Ensure determinism

### Status Byte Protocol

When exec connects to stdin port:

```
┌─────────┐     connect     ┌────────────┐
│  exec   │ ──────────────► │ supervisor │
│ client  │                 │            │
└─────────┘                 └────────────┘
     │                            │
     │                            │ Check exec lock
     │                            ▼
     │                      ┌───────────┐
     │                      │ Lock free?│
     │                      └─────┬─────┘
     │                        yes │ no
     │                            │
     ▼                            ▼
┌─────────┐  0x00 (OK)      ┌─────────┐  0x01 (BUSY)
│ Proceed │ ◄────────────── │ Proceed │ ◄──────────
│ stream  │                 │ stream  │
└─────────┘                 └─────────┘
                                  │
                                  ▼
                            Return E-EXEC-ALREADY-ACTIVE (81)
```

**Status bytes:**
| Byte | Meaning | Client Action |
|------|---------|---------------|
| `0x00` | OK | Begin streaming |
| `0x01` | BUSY | Return exit 81 |
| `0x02-0xFF` | Reserved | Log warning, proceed |

**Important:** Status byte is connection setup, NOT data framing. All subsequent bytes are raw user data.

---

## Error Recovery

### Port Exhaustion (E-ALLOC-PORT)

**Detection:** All ports 49152-65535 in use.

**Recovery:**
1. Return error 22 immediately
2. User disconnects unused sessions
3. Wait for TIME_WAIT expiry (~60s)
4. Retry allocation

### FD Exhaustion (E-ALLOC-FD)

**Detection:** All FDs 3-255 in use.

**Recovery:**
1. Return error 21 immediately
2. User closes sessions
3. User increases ulimit
4. Retry allocation

### Process Crash

**Detection:** `waitpid` returns with exit status.

**Recovery:**
1. Detect via SIGCHLD or poll
2. Transition to Terminated
3. Clean up FDs and ports
4. Remove session from dictionary
5. User can restart

### Connection Lost (Connect Mode)

**Detection:** Read/write returns error.

**Recovery (reconnect enabled):**
1. Detect socket error
2. Close current connection
3. Wait with exponential backoff
4. Attempt reconnect
5. Resume streaming

**Recovery (--no-reconnect):**
1. Detect socket error
2. Transition to Terminated
3. Clean up resources

---

## Security Implementation

### Localhost-Only Binding

```rust
// All sockets bind to loopback only
let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
socket.bind(addr)?;
```

**Guarantees:**
- No network exposure
- No remote connections
- No firewall config needed

### Process Isolation

Each session is isolated:
- Separate process (redirect mode)
- Separate FDs (no sharing)
- Separate ports (no conflicts)
- No cross-session access

### No Authentication

**Design choice:** Local-only, single-user.

**Rationale:**
- All connections from localhost
- Trust local processes
- Simpler implementation
- Faster operation

---

## Performance Characteristics

### Latency

| Operation | Typical | Max |
|-----------|---------|-----|
| Session creation | 5ms | 50ms |
| Exec connection | 1ms | 10ms |
| Data round-trip | <1ms | 5ms |
| Session cleanup | 2ms | 20ms |

### Throughput

**Bottleneck:** Application, not Telepipe.

```
Telepipe overhead: ~0.1% of total throughput
TCP localhost: >1 GB/s possible
Practical limit: Whatever your app can handle
```

### Memory Usage

| Component | Memory |
|-----------|--------|
| Base supervisor | ~2MB |
| Per session | ~5MB |
| Per exec | ~1MB buffers |

### File Descriptors

| Mode | FDs |
|------|-----|
| Redirect | 3 per session |
| Connect | 1 per session |
| System overhead | ~10 base |

---

## Testing Tiers

### Level 1: Constitutional (100% required)

Tests that verify constitutional compliance:
- State machine correctness
- Resource allocation rules
- Error code mapping
- Lock ordering

### Level 2: Production (100% required)

Tests that verify production behavior:
- 300+ command sequences
- All session modes
- Error recovery paths
- Edge cases

### Level 3: Chaos (≥95% required)

Tests that verify stability under stress:
- 1000+ rapid commands
- Concurrent operations
- Port exhaustion
- FD exhaustion
- Process crashes
- 30-minute stability runs

---

## Implementation Notes

### Event Loop

```rust
// Simplified event loop (actual uses mio/tokio)
loop {
    let events = poll.wait(timeout)?;

    for event in events {
        match event.token() {
            STDIN => handle_stdin_data(),
            STDOUT => handle_stdout_data(),
            STDERR => handle_stderr_data(),
            CHILD => handle_child_exit(),
        }
    }
}
```

### Cross-Platform I/O

| Platform | Event System |
|----------|--------------|
| macOS | kqueue |
| Linux | epoll |
| Windows (WSL2) | epoll (Linux kernel) |

### Signal Handling

```rust
// Child exit detection
signal_hook::register(SIGCHLD, || {
    // Set flag for main loop
    CHILD_EXITED.store(true, Ordering::SeqCst);
})?;
```

---

## Related Documents

- **Core Spec:** `governance/.../501 the telepipe core spec.md`
- **CLI Reference:** `governance/.../515 the telepipe cli reference.md`
- **Error Model:** `governance/.../530 the telepipe error model.md`
- **Concurrency:** `governance/.../550 the telepipe concurrency model.md`
- **Security:** `governance/.../555 the telepipe security model.md`
- **Supervisor:** `governance/.../575 the telepipe supervisor architecture.md`

---

## Summary

Telepipe's architecture is intentionally simple:

- **Supervisor model** - One process manages all I/O
- **Session isolation** - Each session independent
- **Binary transparency** - Data passes through unchanged
- **One-at-a-time exec** - Deterministic, ordered execution
- **Localhost-only** - Secure by default
- **Lock hierarchy** - Deadlock-free concurrency

This simplicity is a feature. It makes Telepipe reliable, predictable, and fast.
