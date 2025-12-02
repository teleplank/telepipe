# Telepipe Concurrency Model & Locks
### *Deterministic • Deadlock-Free • Cross-Platform • Linearizable State Updates*

This document defines the **Telepipe Concurrency & Locking Model**, describing how Telepipe ensures safe state manipulation, predictable supervisor behavior, and race-free interactions across:

- multiple Telepipe invocations
- simultaneous redirect/exec/connect commands
- concurrent session creation
- concurrent session teardown
- cross-shell and cross-process usage
- recovery pipeline runs

Telepipe's concurrency model is intentionally simple, portable, and safe across macOS, Linux, and Windows (Git Bash/MSYS2), while supporting arbitrarily high levels of churn.

---

# 1. Design Goals

Telepipe must maintain:

- **atomic session dictionary updates**
- **guaranteed serialization of conflicting operations**
- **safe parallel operation of independent sessions**
- **no deadlocks**
- **no partial writes**
- **no race conditions for port or FD allocation**
- **no "double supervisor" creation**
- **idempotent recovery passes**

The final result is:

**"Any number of Telepipe processes may run concurrently without ever corrupting Telepipe's state."**

---

# 2. Concurrency Primitives Used

Telepipe uses **only the following primitives**, guaranteed to exist on macOS, Linux, and Git Bash/MSYS2:

### 2.1 File Locks (`flock` or portable fallback)
- Uses advisory locks.
- Locks are process-scoped.
- Works on plain files (dictionary directory + session files).

If `flock` is not available, Telepipe automatically uses its portable fallback:
- open lockfile for read/write
- attempt atomic file creation or `fcntl` advisory lock
- retry with exponential backoff

### 2.2 Atomic File Replacement
Used during dictionary writes:

```
write tmp → fsync → rename(tmp, real)
```

This prevents:
- torn writes
- partial JSON
- lock bypassing
- inconsistent state

### 2.3 Monotonic Counters in Memory
FD and port scans operate exclusively inside a process.
No global counters are shared.

---

# 3. Lock Hierarchy

Telepipe defines three explicit lock levels:

```
GLOBAL_LOCK     – ensures only one Telepipe invocation mutates dictionary
SESSION_LOCK(ID) – ensures no concurrent edits to a single session
PORTALLOC_LOCK   – ensures only one Telepipe invocation performs port allocation
```

### 3.1 Ordering Rules (Critical)

To avoid deadlocks:

```
GLOBAL_LOCK must always be acquired before any SESSION_LOCK(ID)
SESSION_LOCK(ID) must always be acquired before allocating FDs or ports
PORTALLOC_LOCK must always be acquired before scanning ports
```

No Telepipe path breaks this ordering.

---

# 4. Global Lock

The global lock protects:

- dictionary scan
- recovery pipeline
- dictionary write
- eviction pass
- any operation that modifies or deletes entries

Held only for **short, critical periods**.

Never held during:
- supervisor operation
- network I/O
- child process execution
- streaming

This ensures high throughput even under load.

---

# 5. Session Lock

Each session ID has its own lock file:

```
$HOME/.telepipe/sessions/<id>.lock
```

SESSION_LOCK(ID) is held during:

- redirect creation
- connect creation
- stop
- disconnect
- recovery updates for that session
- dictionary write for that session

SESSION_LOCK never overlaps between different sessions.
This enables:

- parallel redirect spawns
- parallel exec streams
- parallel connect attachments

There is **no global serialization** for independent session work.

---

# 6. Port Allocation Lock

Port allocation is a global shared resource difficulty.
To avoid race conditions:

- multiple Telepipe processes may **read** the dictionary concurrently
- but **only one** may allocate ports at a time

`PORTALLOC_LOCK` is acquired:

- before scanning ephemeral ports
- before binding test sockets
- before FD allocation
- when creating new redirect or connect sessions

Upon release:

- ports are considered committed
- binding in actual supervisor is safe

---

# 7. Exec Mode Concurrency

`telepipe exec`:

- does NOT modify the dictionary
- does NOT participate in port allocation
- does NOT spawn child or supervisors
- does NOT update state

Therefore:
- no GLOBAL lock
- no PORTALLOC lock
- no SESSION lock

`exec` only reads session info under a brief shared read-lock (non-blocking).
Streaming happens with **no locks held**.

### One-at-a-Time Model

Only ONE exec connection may be active per session at any given time.

**Serial exec (supported):**
```bash
# exec → complete → exec → complete (unlimited)
echo "cmd1" | telepipe exec --id session  # Time 0
echo "cmd2" | telepipe exec --id session  # Time 5 (after first completes)
```

**Concurrent exec (not supported):**
```bash
# Second exec while first still active → ERROR
echo "cmd1" | telepipe exec --id session &  # Active
echo "cmd2" | telepipe exec --id session    # E-EXEC-ALREADY-ACTIVE
```

If an exec is attempted while another exec is already connected to the same session, the supervisor sends status byte `0x01` (BUSY), and the exec client returns `E-EXEC-ALREADY-ACTIVE` (exit 81) immediately.

### Status Byte Protocol

To reliably signal exec acceptance or rejection, the supervisor uses a one-byte status handshake on the stdin connection:

**Handshake sequence:**
1. Exec client connects to stdin port
2. Supervisor attempts to acquire exec lock
3. Supervisor writes status byte:
   - `0x00` = OK (exec acquired, proceed with data streaming)
   - `0x01` = BUSY (another exec is active)
4. Exec client reads status byte and acts accordingly:
   - `0x00`: Proceed with data streaming
   - `0x01`: Return E-EXEC-ALREADY-ACTIVE (exit 81)

**Status byte values:**
| Byte | Meaning | Client Action |
|------|---------|---------------|
| `0x00` | OK | Proceed with data streaming |
| `0x01` | BUSY | Return E-EXEC-ALREADY-ACTIVE (exit 81) |
| `0x02-0xFF` | Reserved | Log warning, proceed |

**Properties:**
- Status byte is sent ONCE at connection start on stdin port only
- User data begins immediately after status byte
- Status byte is connection setup, NOT data framing
- All data after status byte is raw and unmodified

**Timeout:**
Exec client waits up to 2 seconds for status byte. If not received, returns E-TCP-CONNECT.

### Why One-at-a-Time?

1. **Protocol-agnostic:** Telepipe does not manage connection ordering or multiplexing
2. **Deterministic:** Clear error instead of silent queueing or response mixing
3. **UNIX philosophy:** Fail fast with explicit feedback
4. **Minimal scope:** TCP tunnel primitive, nothing more

### For Concurrent Workers

Users requiring concurrent access should spawn separate sessions:

```bash
# Each worker gets its own session
telepipe redirect --id worker1 -- websocat ...
telepipe redirect --id worker2 -- websocat ...

# Concurrent exec to different sessions
echo "cmd1" | telepipe exec --id worker1 &
echo "cmd2" | telepipe exec --id worker2 &
```

Or use external multiplexing tools:

```bash
# Use tee to broadcast to multiple consumers
echo "cmd" | tee >(telepipe exec --id session) >(other_consumer)
```

---

# 8. Supervisor Concurrency

Supervisors:

- operate without holding locks
- communicate only via the TCP sockets they own
- do not modify session dictionary directly

Supervisor exits trigger:

- normal child exit handling
- cleanup
- recovery pass

Recovery pass uses GLOBAL_LOCK.

---

# 9. Recovery Pipeline + Concurrency

Before *every* Telepipe command except `telepipe info`, Telepipe performs:

```
acquire GLOBAL_LOCK
run recovery steps
release GLOBAL_LOCK
```

Because the pipeline is idempotent:

- it is safe to run concurrently
- it never deadlocks
- it always results in a consistent dictionary

During recovery:
- session locks are temporarily acquired as needed
- conflicts are prevented by global lock precedence

---

# 10. Deadlock Avoidance

This ordering prevents every known deadlock pattern:

### 10.1 No Lock Inversion
All paths follow:

```
GLOBAL → SESSION → PORTALLOC
```

Never reverse.

### 10.2 Never Hold Locks While Blocking on IO
Telepipe ensures:
- no TCP streaming under locks
- no child wait loops under locks
- no long operations under locks
- no subprocess creation under long-held locks

### 10.3 Supervisors Never Compete for Locks
Supervisors:
- don't acquire locks
- don't modify dictionary
- cannot deadlock or starve other Telepipe invocations

---

# 11. FD Allocation Concurrency

Telepipe allocates FDs **after acquiring session lock** but **before spawning**.

FD allocation is never global and therefore cannot race with other Telepipe instances.

Guarantees:

- FDs are process local
- session lock ensures exclusive per-session control
- no parallel entity can claim the same FD

---

# 12. TCP Allocation Concurrency

Port allocation uses PORTALLOC_LOCK, ensuring:

- no two Telepipe processes scan the same ports simultaneously
- no conflicts due to TIME_WAIT
- no reuse races

The same lock prevents:

- two sessions picking the same port
- partially created session port claims
- incorrect dictionary writes

---

# 13. Concurrency Under Extreme Load

Telepipe is designed to handle:

- dozens of redirect spawns in parallel
- hundreds of exec operations concurrently
- parallel connect operations
- continuous churn across shells

The concurrency model ensures:

- starvation-free operations
- bounded retry loops
- deterministic behavior
- no compound failure scenarios

---

# 14. Read-Only Operations

The following operations are **pure read**:

- `telepipe info --id` (reads dictionary, no modifications)
- `telepipe exec --id` (reads session info, then streams I/O)

Both acquire only a brief shared dictionary read-lock and release before streaming/output.
They never block redirect/connect creation.

---

# 15. Cross-Platform Locking Strategies

## 15.1 macOS & Linux
- Use `flock` for coarse-grain locks
- Use advisory `fcntl` for fallback

## 15.2 Windows (Git Bash / MSYS2)
Git Bash provides POSIX-like file locking through:
- `flock`
- or emulated via `msys2` runtime

If unavailable:
- locking falls back to portable "atomic create lock" pattern:
```
lockfile = <file>.lock
attempt open(lockfile, O_CREAT | O_EXCL)
```

Retry with backoff:
```
50ms → 100ms → 200ms → 400ms → 800ms → capped at 1s
```

---

# 16. Lock Diagnostics (Optional)

Telepipe may support:

```
--trace-locks
```

Displays:
- lock acquisition time
- lock release time
- lock contention statistics
- lock ordering violations (should never occur)

---

# 17. Summary

Telepipe's concurrency & locking model ensures:

- **no global contention except for dictionary modification**
- **per-session concurrency for all operations**
- **idempotent, deadlock-free recovery**
- **race-free port and FD allocation**
- **no supervisor participates in locks**
- **parallel redirect, exec, and connect operations**
- **safe behavior on macOS, Linux, and Windows**

This design makes Telepipe scalable, predictable, and robust even under extreme churn across multiple shells and processes.
