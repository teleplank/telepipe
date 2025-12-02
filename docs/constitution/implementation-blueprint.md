# Telepipe Implementation Blueprint (Rust)
### *Module-by-Module • Deterministic • Memory-Safe • Cross-Platform*

This is the **official Rust implementation blueprint** for Telepipe Core.
It provides a detailed, production-ready architectural breakdown for implementing Telepipe's:

- redirect behavior
- exec behavior
- connect behavior
- stop/disconnect
- session dictionary
- supervisor architecture
- recovery pipeline
- concurrency model
- locks
- error model
- FD/TCP allocation
- process spawning
- streaming

This blueprint is language-specific and assumes Rust 2021 or newer.

---

# 1. High-Level Architecture

Telepipe should be implemented as the following Rust modules:

```
src/
  main.rs
  cli.rs
  config.rs
  errors.rs
  fds.rs
  tcp_alloc.rs
  process_spawn.rs
  supervisor.rs
  session_dict/
      mod.rs
      entry.rs
      io.rs
      recovery.rs
      locks.rs
  exec_mode.rs
  connect_mode.rs
  redirect_mode.rs
  stop.rs
  disconnect.rs
  info.rs
  platform/
      mod.rs
      posix.rs
      windows.rs
  util/
      json.rs
      atomic_write.rs
      logging.rs
      time.rs
```

Each module has a precise responsibility.
No module may access another module's internals except through explicit interfaces.

---

# 2. Core Data Structures

## 2.1 `SessionEntry`
```rust
pub struct SessionEntry {
    pub id: String,
    pub mode: SessionMode,
    pub supervisor_pid: u32,
    pub child_pid: Option<u32>,
    pub host: String,
    pub stdin_port: Option<u16>,
    pub stdout_port: Option<u16>,
    pub stderr_port: Option<u16>,
    pub connect_port: Option<u16>,
    pub stdin_fd: Option<u32>,
    pub stdout_fd: Option<u32>,
    pub stderr_fd: Option<u32>,
    pub connect_fd: Option<u32>,
}
```

## 2.2 `SessionMode`
```rust
pub enum SessionMode {
    Redirect,
    Connect,
}
```

## 2.3 `SupervisorConfig`
```rust
pub struct SupervisorConfig {
    pub entry: SessionEntry,
}
```

---

# 3. Error Handling

All errors must use a central enum:

```rust
#[derive(Debug, thiserror::Error)]
pub enum TelepipeError {
    #[error("no available numeric file descriptors")]
    AllocFd,
    #[error("unable to allocate TCP port")]
    AllocPort,
    #[error("unable to write session dictionary")]
    DictWrite,
    #[error("session not found")]
    DictMissing,
    #[error("invalid or missing arguments")]
    InvalidArgs,
    #[error("failed to spawn child process")]
    SpawnFailed,
    #[error("tcp connection closed unexpectedly")]
    TcpBroken,
    #[error("remote endpoint reset connection")]
    TcpReset,
    #[error("session dictionary corrupt")]
    DictCorrupt,
    #[error("session dictionary stale")]
    DictStale,
    #[error("supervisor internal failure")]
    SupervisorFailure,
}
```

All errors must map 1:1 to the error model.

---

# 4. Module Breakdown

# 4.1 `cli.rs`
- Parse CLI options.
- Identify behavior: redirect / exec / connect / stop / disconnect / info.
- Never perform logic directly — dispatch to modules.

---

# 4.2 `tcp_alloc.rs`
### Responsibility:
- scan ephemeral port range
- ensure bind-ability
- allocate three ports or fail

### Key Functions:
```rust
pub fn allocate_port() -> Result<u16, TelepipeError>;
pub fn allocate_three_ports() -> Result<(u16, u16, u16), TelepipeError>;
```

---

# 4.3 `fds.rs`
### Responsibility:
- probe FDs 3–256
- find available
- never reuse in same process
- error on exhaustion

### Functions:
```rust
pub fn allocate_fd() -> Result<u32, TelepipeError>;
pub fn allocate_three_fds() -> Result<(u32, u32, u32), TelepipeError>;
```

---

# 4.4 `process_spawn.rs`
### Responsibility:
- spawn child processes
- redirect stdio to TCP sockets
- POSIX: use `Command` + `StdIO::from_raw_fd`
- Windows (Git Bash): use process spawning with pipes → TCP fallback

### Functions:
```rust
pub fn spawn_redirected_child(
    cmd: &str,
    args: &[String],
    fds: (u32, u32, u32)
) -> Result<u32, TelepipeError>;
```

---

# 4.5 `supervisor.rs`
### Responsibility:
- run per-session supervisor
- handle I/O multiplexing with `mio` or `tokio`
- monitor child death
- detect TCP resets
- emit exit status
- replace broken connections (connect-mode)
- no dictionary writes

---

# 4.6 `session_dict/mod.rs`
Handles:

- reading session files
- writing session files
- JSON encoding/decoding
- session locking
- atomic write + fsync
- validation logic

### Key Functions:
```rust
pub fn load_all() -> Result<Vec<SessionEntry>, TelepipeError>;
pub fn write(entry: &SessionEntry) -> Result<(), TelepipeError>;
pub fn delete(id: &str) -> Result<(), TelepipeError>;
pub fn load(id: &str) -> Result<SessionEntry, TelepipeError>;
```

All I/O must be atomic.

---

# 4.7 `session_dict/locks.rs`
Implements:

- GLOBAL lock
- PER-SESSION lock
- PORTALLOC lock

Using:
- `flock` on POSIX
- atomic-create lockfile fallback

---

# 4.8 `session_dict/recovery.rs`
Implements Telepipe's recovery pipeline:

```
SCAN → VALIDATE → REPAIR → EVICT
```

### Functions:
```rust
pub fn recover() -> Result<(), TelepipeError>;
```

- Reconstruct supervisor for redirect-mode
- Delete connect-mode entries with dead supervisors
- Remove stale entries
- No user-visible effects outside `--debug`

---

# 4.9 `redirect_mode.rs`
Entry point for:

```rust
telepipe redirect --id X -- cmd args...
```

Steps:

1. acquire GLOBAL_LOCK
2. recovery pass
3. acquire SESSION_LOCK
4. acquire PORTALLOC_LOCK
5. allocate FDs
6. allocate ports
7. create TCP sockets
8. spawn child
9. spawn supervisor
10. write dictionary
11. release locks
12. exit (supervisor stays alive)

---

# 4.10 `exec_mode.rs`
Entry point for:

```rust
telepipe exec --id X
```

Steps:

1. load session (no global lock needed)
2. attempt TCP connections
3. stream stdin → remote stdin
4. stream stdout/stderr → local stdout/stderr
5. exit when EOF

Zero dictionary writes.

---

# 4.11 `connect_mode.rs`
Entry point for:

```rust
telepipe connect --id X --port 5432
```

Steps:

1. acquire GLOBAL_LOCK
2. recovery pass
3. acquire SESSION_LOCK
4. allocate FD
5. create TCP connection
6. spawn supervisor
7. write dictionary
8. exit

Supervisor will reconnect unless `--no-reconnect`.

**Note:** For WebSocket-based protocols like CDP, use redirect mode with websocat:
```bash
WS_URL=$(curl -s http://127.0.0.1:9222/json | jq -r '.[0].webSocketDebuggerUrl')
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

---

# 4.12 `stop.rs`
Entry point for:

```rust
telepipe stop --id X
```

Steps:

1. acquire GLOBAL_LOCK
2. recovery
3. SESSION_LOCK
4. load entry
5. kill child
6. kill supervisor
7. delete dictionary entry

---

# 4.13 `disconnect.rs`
Entry point:

```rust
telepipe disconnect --id X
```

Steps:

1. acquire GLOBAL_LOCK
2. recovery
3. SESSION_LOCK
4. kill supervisor
5. delete session entry

---

# 4.14 `info.rs`
Entry point for:

```rust
telepipe info --id X
```

Steps:

1. load entry (read-only, no global lock needed)
2. format fields as tab-separated output
3. print to stdout
4. exit immediately with code 0

**Performance requirement:** Must complete in <100ms.

**Locking:** Uses only brief shared read-lock on dictionary entry file.

**Critical guarantee:** Info mode exits immediately after printing. Never blocks.

---

# 5. Streaming Architecture

Telepipe streaming relies on:

- `mio`, `poll`, `epoll`, or `kqueue`
- non-blocking TCP sockets
- separate task for each direction (stdin → remote, remote → stdout, remote → stderr)

Important invariants:

- no parsing
- no transformation
- raw byte copy (`copy_bidirectional`)
- shut down connection on error

---

# 6. Cross-Platform Abstraction (`platform/mod.rs`)

Implemented via:

### posix.rs
- raw FDs
- execve redirection
- signal handling
- process table scanning
- TCP binding via libc

### windows.rs
- Git Bash compatibility layer
- named pipes converted to TCP fallback
- process spawning via `Command`
- no reliance on Win32 APIs except `kill` equivalent

---

# 7. Utilities (`util/`)

## 7.1 `atomic_write.rs`
Implements:

```
write tempfile
fsync tempfile
rename over real file
```

Atomic on all supported platforms.

## 7.2 `json.rs`
Stable JSON encode/decode using `serde_json` with strict validation.

## 7.3 `logging.rs`
Handles:
- debug mode
- trace mode
- startup diagnostics

## 7.4 `time.rs`
Provides:
- timestamps
- exponential backoff for lock contention
- retry helpers for connect-mode

---

# 8. Supervisor Implementation Details

Supervisor must:

- run event loop
- detect TCP hangups
- detect child death
- emit exit status
- exit cleanly
- never panic
- never write dictionary

### Rust structure:
```rust
pub struct Supervisor {
    pub config: SupervisorConfig,
}
```

### Entrypoint:
```rust
impl Supervisor {
    pub fn run(&mut self) -> Result<(), TelepipeError> {
        // event loop here
    }
}
```

Event loop must use:

- `poll` on POSIX
- fallback to blocking reads on Git Bash
- no locks held during streaming

---

# 9. Testing Hooks

Telepipe must expose internal hooks for golden tests:

- override port allocator
- override fd allocator
- override child spawn
- override supervisor spawn
- deterministic reproduction of all failures

Hooks placed behind:

```rust
#[cfg(test)]
```

And never included in release builds.

---

# 10. Summary

This blueprint defines:

- the full Rust module structure
- data models
- error architecture
- concurrency architecture
- supervisor architecture
- redirect / exec / connect flows
- session dictionary and locking
- cross-platform abstractions
- streaming model
- testing hooks

This is the canonical reference for implementing Telepipe Core in Rust.
