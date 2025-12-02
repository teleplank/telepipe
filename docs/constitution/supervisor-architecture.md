# Telepipe Supervisor Architecture
### *I/O Multiplexing • Child Lifecycle • TCP Lifecycle • Self-Healing • Zero-Parsing • Purely Reactive*

This document defines the **complete and canonical architecture** of the Telepipe Supervisor — the long-lived process responsible for:

- managing redirected or connected TCP endpoints
- maintaining persistent streaming
- supervising child processes (redirect mode)
- handling remote endpoints (connect mode)
- reacting to I/O, timeouts, and disconnects
- reconnecting when necessary
- emitting exit status
- terminating cleanly

Supervisors are **the beating heart of Telepipe**, enabling stable, deterministic behavior across CLI calls and long-running sessions.

---

# 1. Core Supervisor Principles

A supervisor MUST follow these principles:

1. **No dictionary writes**
2. **Never hold locks during streaming**
3. **Binary-blind streaming** (no parsing or rewriting of data)
4. **Purely reactive event loop**
5. **Detect and report failures without crashing**
6. **Stay alive after the original CLI command exits**
7. **No global state**
8. **No elevated permissions**
9. **Cross-platform identical semantics**
10. **Never influence or override TCP framing**

These constraints guarantee portability, determinism, and safety.

---

# 2. Supervisor Types

Telepipe implements two distinct supervisor types:

## 2.1 Redirect Supervisor
Responsible for:

- reading from TCP → writing to child stdin
- reading from child stdout → writing to TCP
- reading from child stderr → writing to TCP
- monitoring child PID
- shutting down when child exits
- emitting exit status

Redirect supervisors are **tethered** to a child process.

---

## 2.2 Connect Supervisor
Responsible for:

- reading from inbound TCP → writing to remote TCP
- reading from remote TCP → writing to outbound TCP
- optionally reconnecting remote TCP after resets
- shutting down when session is disconnected

Connect supervisors **do not own a child process**.

---

# 3. Supervisor Lifecycles

## 3.1 Redirect Supervisor Lifecycle

```
spawned → initialize → bind FDs → event loop → child exit detected → cleanup → exit
```

Redirect supervisors terminate automatically when:

- the child process terminates
- child's stdio becomes invalid
- TCP endpoints break
- fatal error occurs

Supervisor lifetime = child lifetime.

---

## 3.2 Connect Supervisor Lifecycle

```
spawned → initialize → connect remote → event loop
```

Connect supervisors run indefinitely until:

- `telepipe disconnect --id X` is invoked
- `--no-reconnect` and remote disconnects
- fatal error occurs

Supervisor lifetime = user-controlled.

---

# 4. Supervisor Responsibilities

Every supervisor MUST implement:

1. **I/O Multiplexing**
2. **TCP Connection Management**
3. **Child PID Monitoring** (redirect only)
4. **Event Loop**
5. **Reconnect Logic** (connect only)
6. **Graceful Shutdown**
7. **Signal-safe behavior**
8. **Cross-platform I/O semantics**

All supervisors must be **single-threaded**, internally asynchronous, and use:

- `poll` / `epoll` / `kqueue` on POSIX
- non-blocking I/O with fallback loops on Git Bash

---

# 5. Internal Supervisor Components

## 5.1 The Supervisor Struct

```rust
pub struct Supervisor {
    pub entry: SessionEntry,
    pub role: SupervisorRole,
    pub inbound_fd: Option<u32>,   // stdin or inbound TCP
    pub outbound_fd: Option<u32>,  // stdout/stderr or outbound TCP
    pub stderr_fd: Option<u32>,    // redirect stderr-only pipe
    pub remote_fd: Option<u32>,    // connect remote TCP
    pub child_pid: Option<u32>,    // redirect only
    pub reconnect: bool,           // connect only
}
```

## 5.2 SupervisorRole
```rust
pub enum SupervisorRole {
    Redirect,
    Connect,
}
```

---

# 6. Event Loop Architecture

The supervisor event loop must be:

- non-blocking
- reactive
- never blocking on a single FD
- never parsing byte content
- never holding locks

### High-Level Loop

```rust
loop {
    poll_all_fds();

    if child_exited { handle_child_exit(); break; }
    if tcp_reset { handle_tcp_reset(); }
    if fatal_error { break; }

    copy_read_ready_fds();
}
```

The loop is **tight, simple, and portable**.

---

# 7. Detailed I/O Flow

## 7.1 Redirect Mode

### TCP → Child STDIN

```
read(tcp_stdin_fd)  →  write(child_stdin_fd)
```

### Child STDOUT → TCP

```
read(child_stdout_fd)  →  write(tcp_stdout_fd)
```

### Child STDERR → TCP

```
read(child_stderr_fd)  →  write(tcp_stderr_fd)
```

All data is copied raw, without framing, buffering heuristics, or modification.

**Note:** The status byte protocol (see Concurrency Model Section 7) is connection setup, not data modification. The raw data stream begins after the status byte is consumed by the exec client.

---

## 7.2 Connect Mode

### Inbound → Remote

```
read(local_inbound_fd)  →  write(remote_fd)
```

### Remote → Outbound

```
read(remote_fd)  →  write(local_outbound_fd)
```

Data flows through supervisor unaltered.

---

# 8. Child Monitoring (Redirect Only)

The supervisor must monitor:

- `waitpid` (POSIX)
- `process::Child::try_wait` (Rust)
- OS signals indicating termination
- broken stdio pipes

Once the child terminates, supervisor must:

1. flush pending buffers
2. close FDs
3. notify via exit status
4. terminate

---

# 9. TCP Reset & Reconnection (Connect Only)

If TCP reset occurs:

```
remote closed → attempt reconnect (unless --no-reconnect)
```

Reconnect logic must:

- wait with exponential backoff
- never hold locks
- not modify dictionary
- avoid storms via max retry intervals
- rebind inbound/outbound fds

If `--no-reconnect` is set:

→ supervisor shuts down immediately.

---

# 10. Error Handling Semantics

Supervisor internal errors must map to:

- `E-TCP-BROKEN`
- `E-TCP-RESET`
- `E-SUPERVISOR-FAIL`

Supervisors must:

- log errors under `--debug`
- never panic
- never crash the parent shell
- always terminate cleanly

---

# 11. Shutdown Semantics

Redirect supervisors shutdown when:

- child terminates
- I/O is unrecoverably closed
- tcp endpoints die
- kill command (`telepipe stop --id`)

Connect supervisors shutdown when:

- remote unreachable w/ `--no-reconnect`
- kill command (`telepipe disconnect`)

Shutdown must free:

- all FDs
- TCP sockets
- OS resources

OS then terminates ephemeral ports automatically.

---

# 12. Cross-Platform Behavior

Supervisors must provide identical behavior on:

- macOS
- Linux
- Windows Git Bash/MSYS2

Platform variations:

| Feature | POSIX | Git Bash |
|---------|--------|-----------|
| poll/epoll | yes | partial |
| non-blocking TCP | yes | yes |
| process signals | native | emulated |
| waitpid | native | limited |
| stdin/out pipes | native | converted to TCP |

Supervisor must abstract over these differences.

---

# 13. Metrics & Diagnostics (Optional)

A future tracing system may expose:

- bytes transferred per direction
- reconnect attempts
- TCP reset counts
- child exit codes
- supervisor uptime
- I/O throughput

Not required for core spec.

---

# 14. Resource Constraints

### 14.1 CPU Footprint

Telepipe supervisors must:
- Consume effectively zero CPU when idle
- Avoid permanent busy loops
- Sleep on I/O readiness (epoll, kqueue, or select)

### 14.2 Memory Footprint

Memory overhead must not exceed:
- 1–2 MB for redirect mode
- <1 MB for connect mode

---

# 15. TCP Behavior

### 15.1 TCP Backpressure

When the child emits data faster than the client reads:
- Sockets may fill
- Child may block
- Telepipe supervisor must handle this properly

**Solution:**
- Non-blocking reads
- select/poll loops
- Configurable backpressure thresholds (future)

### 15.2 TCP Half-Closes

Child may close stdout but not stderr. Telepipe must handle half-closed streams gracefully.

### 15.3 Idle Timeout Handling

- Redirect sessions have no idle timeout
- Connect sessions obey infinite reconnect unless `--no-reconnect` is set

---

# 16. Security Responsibilities

Supervisors MUST:

- never modify session dictionary
- never escalate privileges
- never drop to shell
- never read arbitrary files
- never open arbitrary TCP ports
- never expose non-loopback connections unless configured

Supervisors are **payload-blind transport agents**.

---

# 17. Summary

The Telepipe Supervisor Architecture is:

- **simple** (single-threaded state machine)
- **safe** (no parsing, no privilege)
- **predictable** (identical across platforms)
- **reactive** (purely event-driven)
- **deterministic** (no global state, no race conditions)
- **reliable** (reconnect, child monitoring)
- **modular** (redirect vs connect roles)

It is the keystone of Telepipe's long-lived, multi-session, AI-compatible behavior.
