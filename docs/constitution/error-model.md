# Telepipe Error Model & Error Codes
### *Deterministic • Machine‑Readable • Human‑Understandable • Cross‑Platform*

This document defines the **official Telepipe Error Model**, including:
- error classes
- error codes
- error messages
- error semantics
- cross‑platform expectations
- exit codes
- structured diagnostics

This model is **strict**, **machine‑parseable**, and **stable across versions**, ensuring reproducible behavior in automated environments (AI agents, CI, automation, etc.).

---

# 1. Overview of the Telepipe Error Model

Telepipe errors are grouped into **eight classes**, each containing specific failure types:

1. **E‑ALLOC** – Allocation failures
2. **E‑FD** – File descriptor failures
3. **E‑TCP** – TCP socket failures
4. **E‑PROC** – Process lifecycle failures
5. **E‑DICT** – Session dictionary failures
6. **E‑CLI** – User input / CLI usage failures
7. **E‑EXEC** – Exec mode failures
8. **E‑INFO** – Info mode failures (rare)

Each Telepipe command must return:
- a **non‑zero exit code**
- a **single‑line machine‑friendly error message**
- and optionally a **multi‑line diagnostic block** when `--debug` is passed

Error messages follow the format:

```
ERROR <CODE>: <short description>
```

Example:

```
ERROR E-ALLOC-PORT: unable to allocate TCP port
```

---

# 2. Error Class: E‑ALLOC
### *Allocation failures (FDs, TCP ports, etc.)*

Telepipe must detect allocation failures **before** spawning the child or connecting to external services.

## E‑ALLOC‑FD
**Description:** No free numeric file descriptors available.
**Trigger:** FD scan (3–255) fails.
**Message:**
```
ERROR E-ALLOC-FD: no available numeric file descriptors
```
**Exit code:** `21`

## E‑ALLOC‑PORT
**Description:** No free TCP ports can be allocated in the ephemeral range.
**Trigger:** Port scan (49152–65535) fails.
**Message:**
```
ERROR E-ALLOC-PORT: unable to allocate TCP port
```
**Exit code:** `22`

## E‑ALLOC‑DICT
**Description:** Session dictionary cannot be created or updated (permissions, corruption).
**Message:**
```
ERROR E-ALLOC-DICT: unable to write session dictionary
```
**Exit code:** `23`

---

# 3. Error Class: E‑FD
### *File Descriptor Binding / Redirection Failures*

## E‑FD‑BIND
**Description:** Failed to bind numeric FD to TCP socket (`exec {FD}<>/dev/tcp/...`).
**Message:**
```
ERROR E-FD-BIND: unable to bind numeric FD to TCP socket
```
**Exit code:** `31`

## E‑FD‑REDIRECT
**Description:** Failed to redirect stdio of child process.
**Message:**
```
ERROR E-FD-REDIRECT: failed to redirect child stdio
```
**Exit code:** `32`

---

# 4. Error Class: E‑TCP
### *TCP Connection & Streaming Failures*

## E‑TCP‑CONNECT
**Description:** Failed to establish TCP connection.
**Message:**
```
ERROR E-TCP-CONNECT: unable to establish TCP connection
```
**Exit code:** `41`

## E‑TCP‑BROKEN
**Description:** TCP connection broke during streaming / forwarding.
**Message:**
```
ERROR E-TCP-BROKEN: tcp connection closed unexpectedly
```
**Exit code:** `42`

## E‑TCP‑RESET
**Description:** The remote endpoint forcibly closed the connection.
**Message:**
```
ERROR E-TCP-RESET: remote endpoint reset connection
```
**Exit code:** `43`

## E‑TCP‑HALF
**Description:** Half‑closed stream detected (stdout or stderr closed prematurely).
**Message:**
```
ERROR E-TCP-HALF: unexpected half-close of tcp stream
```
**Exit code:** `44`

**Note:** E-TCP-HALF is emitted only when half-close occurs in an unexpected order (e.g., stderr closes before stdout when both should remain open). Normal half-close patterns (child closes stdout, stderr remains) are not errors.

---

# 5. Error Class: E‑PROC
### *Child/Supervisor Process Lifecycle Failures*

## E‑PROC‑SPAWN
**Description:** Failed to spawn the child process.
**Message:**
```
ERROR E-PROC-SPAWN: failed to spawn child process
```
**Exit code:** `51`

## E‑PROC‑SUPERVISOR
**Description:** Supervisor encountered an unrecoverable internal error.
**Message:**
```
ERROR E-PROC-SUPERVISOR: supervisor internal failure
```
**Exit code:** `52`

## E‑PROC‑NOCHILD
**Description:** Child PID listed in dictionary but the process does not exist.
**Message:**
```
ERROR E-PROC-NOCHILD: child process not found
```
**Exit code:** `53`

## E‑PROC‑EXITCODE
**Description:** Child returned non‑zero exit code when unexpected.
**Message:**
```
ERROR E-PROC-EXITCODE: unexpected child exit code
```
**Exit code:** `54`

**Note:** This error is rare and reserved for future use. Currently, redirect supervisors always exit with the child's exit code, even if non-zero.

---

# 6. Error Class: E‑DICT
### *Session Dictionary Errors*

## E‑DICT‑MISSING
**Description:** No session found for provided ID.
**Message:**
```
ERROR E-DICT-MISSING: session not found
```
**Exit code:** `61`

## E‑DICT‑CORRUPT
**Description:** Session dictionary entry is malformed or unreadable.
**Message:**
```
ERROR E-DICT-CORRUPT: session dictionary corrupt
```
**Exit code:** `62`

## E‑DICT‑STALE
**Description:** Entry references dead PIDs or closed FDs.
**Message:**
```
ERROR E-DICT-STALE: session dictionary entry is stale
```
**Exit code:** `63`

## E‑DICT‑CONFLICT
**Description:** Session ID already exists in dictionary.
**Message:**
```
ERROR E-DICT-CONFLICT: session id already exists
```
**Exit code:** `64`

**Trigger:** User attempts to create redirect or connect session with an ID that already exists.

---

# 7. Error Class: E‑CLI
### *User Input / Argument Usage Errors*

## E‑CLI‑ARGS
**Description:** Invalid flag, missing required flag, or incorrect argument form.
**Message:**
```
ERROR E-CLI-ARGS: invalid or missing arguments
```
**Exit code:** `71`

## E‑CLI‑MODE
**Description:** Combination of flags did not match any known behavior.
**Message:**
```
ERROR E-CLI-MODE: invalid mode or incompatible flags
```
**Exit code:** `72`

## E‑CLI‑ID
**Description:** Illegal session ID (wrong charset, empty, etc.).
**Message:**
```
ERROR E-CLI-ID: invalid session identifier
```
**Exit code:** `73`

## E‑CLI‑INVALID‑HOST
**Description:** Invalid host or IP address provided.
**Message:**
```
ERROR E-CLI-INVALID-HOST: invalid host or ip address
```
**Exit code:** `74`

## E‑CLI‑INVALID‑FD
**Description:** User-provided FD override is invalid (out of range, already in use).
**Message:**
```
ERROR E-CLI-INVALID-FD: invalid file descriptor
```
**Exit code:** `75`

## E‑CLI‑INVALID‑PORT
**Description:** User-provided port override is invalid (out of range, already in use).
**Message:**
```
ERROR E-CLI-INVALID-PORT: invalid tcp port
```
**Exit code:** `76`

---

# 7.5. Error Class: E‑EXEC
### *Exec mode failures*

## E‑EXEC‑ALREADY‑ACTIVE
**Description:** An exec call was attempted while another exec is currently connected to the same session. Only one exec connection is allowed at a time per session.
**Trigger:** User calls `telepipe exec --id <session>` while another exec is active on that session.
**Message:**
```
ERROR E-EXEC-ALREADY-ACTIVE: another exec is connected to session '<id>'
```
**Exit code:** `81`

**When This Occurs:**
- User calls `telepipe exec --id <session>` while another exec is active on that session
- Multiple concurrent exec attempts from different terminals/processes

**User Action:**
1. Wait for the current exec to complete, then retry
2. Or spawn separate sessions for concurrent workers:
   ```bash
   telepipe redirect --id worker1 -- ...
   telepipe redirect --id worker2 -- ...
   ```
3. Or use external multiplexing tools like `tee`

**Recovery:** Retry after the active exec connection closes. The session remains valid; only the concurrent attempt is rejected.

**Detection Mechanism:**
The supervisor uses a one-byte status protocol on the stdin connection:
- When exec connects to stdin port, supervisor writes `0x01` (BUSY) if another exec is active
- Exec client reads status byte and immediately returns this error
- No user data is transferred when exec is rejected

---

# 7.6. Error Class: E‑INFO
### *Info mode failures (rare)*

## E‑INFO‑READ
**Description:** Info mode could not read session dictionary entry.
**Message:**
```
ERROR E-INFO-READ: unable to read session info
```
**Exit code:** `68`

**Note:** Info failures are rare since info is read-only. This error only occurs if dictionary file is locked, corrupted, or deleted during read.

---

# 8. Exit Code Reference Table

| Code | Error |
|------|-------|
| **21** | E‑ALLOC‑FD |
| **22** | E‑ALLOC‑PORT |
| **23** | E‑ALLOC‑DICT |
| **31** | E‑FD‑BIND |
| **32** | E‑FD‑REDIRECT |
| **41** | E‑TCP‑CONNECT |
| **42** | E‑TCP‑BROKEN |
| **43** | E‑TCP‑RESET |
| **44** | E‑TCP‑HALF |
| **51** | E‑PROC‑SPAWN |
| **52** | E‑PROC‑SUPERVISOR |
| **53** | E‑PROC‑NOCHILD |
| **54** | E‑PROC‑EXITCODE |
| **61** | E‑DICT‑MISSING |
| **62** | E‑DICT‑CORRUPT |
| **63** | E‑DICT‑STALE |
| **64** | E‑DICT‑CONFLICT |
| **68** | E‑INFO‑READ |
| **71** | E‑CLI‑ARGS |
| **72** | E‑CLI‑MODE |
| **73** | E‑CLI‑ID |
| **74** | E‑CLI‑INVALID‑HOST |
| **75** | E‑CLI‑INVALID‑FD |
| **76** | E‑CLI‑INVALID‑PORT |
| **81** | E‑EXEC‑ALREADY‑ACTIVE |

---

# 9. Error Behavior Rules

## 9.1 Redirect mode
Must fail **before spawning** any child. All allocation and validation errors must be detected in the setup phase.

## 9.2 Exec mode
Fails immediately if:
- dictionary lookup fails (E-DICT-MISSING, E-DICT-CORRUPT, E-DICT-STALE)
- TCP endpoint unreachable (E-TCP-CONNECT)
- session invalid

## 9.3 Connect mode
Must retry unless `--no-reconnect` is used.
When reconnecting, only **E‑TCP‑CONNECT** is emitted.

## 9.4 Info mode
Should complete in <100ms. Only E-INFO-READ is possible.

## 9.5 Session dictionary
Any corruption must lead to:
```
ERROR E-DICT-CORRUPT: session dictionary corrupt
```
Telepipe must **avoid partial writes**.

---

# 10. Debug Diagnostics

When `--debug` is passed, Telepipe prints:

```
--- DEBUG START ---
<timestamp>
<current pid>
<session id>
<allocated fds>
<allocated ports>
<tcp connect attempts>
<child/supervisor pids>
<additional context>
--- DEBUG END ---
```

Debug output must:
- appear on stderr
- never be interleaved with stdout
- never alter machine-readable error format

---

# 11. Summary

This error model guarantees:

- deterministic, enumerated failure modes
- stable error codes for automation
- portable behavior
- predictable lifecycle semantics
- strict separation between human and machine outputs
- comprehensive error coverage for all operations

This is the canonical error reference for Telepipe Core.
