# Telepipe Port Exhaustion + FD Exhaustion Fallback Strategies
### *Deterministic • Zero-Ambiguity • OS-Safe • No Race Conditions*

This document defines the **official fallback and mitigation strategies** Telepipe must follow when the operating system is unable to provide:

- available ephemeral TCP ports, or
- available numeric file descriptors (FDs)

These scenarios are rare but **must** be handled gracefully, deterministically, and without corrupting Telepipe's session dictionary.

---

# 1. Core Requirements

Telepipe must:

- never skip port numbers or FD numbers silently
- never select ports or FDs randomly
- never allocate outside the permitted ranges
- fail early and fail cleanly
- return stable error codes
- avoid partial session creation
- never spawn a child process when allocation failed
- maintain atomicity and integrity of session dictionary

---

# 2. Ephemeral TCP Port Exhaustion
### *Exhaustion of ports in 49152–65535*

Telepipe allocates ephemeral ports by scanning monotonically:

```
49152 → 49153 → … → 65535
```

This is exactly aligned with IANA definitions and the behavior of mainstream tools (Chrome, Node, Docker, Kubernetes port-forward, etc.).

## 2.1 Detection

Telepipe detects exhaustion when:

- the port scan reaches 65535
- AND all attempts to bind fail

Error emitted:

```
ERROR E-ALLOC-PORT: unable to allocate TCP port
```

Exit code: **22**

## 2.2 Fallback Strategy

**No fallback exists.**
Telepipe must terminate immediately.

### Why?

- Using non-ephemeral ports risks conflicts with system services
- Reusing already-used ephemeral ports races with OS state
- Looping endlessly wastes CPU and blocks the user
- Falling back to UNIX named pipes breaks Windows parity
- Falling back to Unix domain sockets changes the abstraction model

Telepipe must stay deterministic and portable.

### Allowed behavior:
- Surface an optional hint under `--debug`:
```
hint: consider closing long-lived redirect sessions to free ports
```

---

# 3. Numeric FD Exhaustion
### *No free FDs in the range 3–256*

Telepipe allocates FDs by scanning monotonically:

```
3 → 4 → 5 → … → 256
```

FDs below 3 are reserved.

## 3.1 Detection

Telepipe detects exhaustion when:

- every FD in 3–256 fails the probe
- OR the OS returns EMFILE ("too many open files") during binding

Error emitted:

```
ERROR E-ALLOC-FD: no available numeric file descriptors
```

Exit code: **21**

## 3.2 Fallback Strategy

### 3.2.1 Immediate Effects
Telepipe **must not**:

- spawn the child process
- update the session dictionary
- create partial state
- retry indefinitely

### 3.2.2 Safe Fallback: NO ATTEMPT TO RECOVER

FD exhaustion is fatal at allocation time.
Telepipe must terminate.

### 3.2.3 Optional Debug Hint

Under `--debug`:

```
hint: fd exhaustion may indicate
- runaway redirect sessions
- another program leaking fds
- shell-level soft limits (ulimit -n)
```

---

# 4. Combined Exhaustion Cases
### *Simultaneous short supply of TCP ports and numeric FDs*

If either allocation fails:

- do **not** proceed
- do **not** attempt partial allocation
- do **not** modify session dictionary
- do **not** spawn child
- do **not** attempt fallback via other resource types
- do **not** adjust port scanning order
- do **not** attempt "best effort" behavior

Error precedence rule:

1. FD exhaustion → E-ALLOC-FD
2. Port exhaustion → E-ALLOC-PORT

Reason:
FD failure is terminal at shell-level; no need to scan ports afterward.

---

# 5. Safety Requirements

## 5.1 Atomicity
Telepipe must perform port + FD discovery in a temporary structure.
Only after *all* pass successfully:

→ commit to dictionary
→ spawn supervisor / child

## 5.2 No Side Effects
If exhaustion is detected:
- all temporary bindings must be rolled back
- no lingering sockets
- no orphaned FDs
- no partial supervisors spawned

## 5.3 Deterministic Logs
When `--debug` is enabled:

```
[debug] port_scan_start=49152
[debug] port_scan_end=65535
[debug] attempted_ports=<list>
[debug] fd_scan_start=3
[debug] fd_scan_end=256
[debug] attempted_fds=<list>
```

Never printed in non-debug mode.

---

# 6. Long-Lived Process Considerations

Telepipe supports arbitrarily long redirect sessions.
These consume:

- 3 ephemeral ports
- 3 file descriptors
- 1 supervisor process

If many such sessions exist, exhaustion can occur.

Best practice suggestions (not part of fallback):

- users may run:
```
telepipe info --id <session>
```
to view active sessions

- supervisors may be stopped manually:
```
telepipe stop --id <session>
```

Telepipe encourages **explicit management**, not implicit recycling.

---

# 7. Exhaustion Under Load

## 7.1 High-Churn Workloads
When creating thousands of redirect sessions per minute:

- port reuse may lag (TIME_WAIT)
- ephemeral space may saturate temporarily

Telepipe must never attempt:

- `SO_REUSEADDR`
- `SO_REUSEPORT`
- port randomization

This would break determinism.

## 7.2 Bulk Reaping Behavior

Telepipe recovery pipeline automatically removes:

- stale
- corrupt
- dead
- non-recoverable

sessions before every command invocation, which frees resources.

This is the **only allowed automatic mitigation**.

---

# 8. Testing Requirements

Unit tests + golden tests **must simulate exhaustion** explicitly:

### 8.1 Port exhaustion test
Simulate by:
- binding every port in the ephemeral range
- running redirect-mode spawn
- expecting **E-ALLOC-PORT**

### 8.2 FD exhaustion test
Simulate via:
```
ulimit -n 3
telepipe redirect …
```
Expect **E-ALLOC-FD**.

### 8.3 Combined exhaustion test
Simulate via:
- `ulimit -n 3`
- binding large amounts of ephemeral ports

Expect **E-ALLOC-FD** (FD exhaustion has precedence).

---

# 9. Summary

Telepipe's exhaustion handling is:

- deterministic
- strict
- portable
- race-free
- safe
- preview-friendly for AI agents

Specifically:

- **No fallback exists** for port exhaustion
- **No fallback exists** for FD exhaustion
- both are fatal and must terminate
- recovery pipeline is the only mitigation mechanism
- debug hints may suggest remedies but NOT alter behavior

This guarantees that Telepipe behaves identically across macOS, Linux, and Windows environments, even under extreme stress.
