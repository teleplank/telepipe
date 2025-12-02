# Telepipe Security Model
### *Local-Only • Zero-Privilege • Safe-by-Default • Minimal Attack Surface*

Telepipe's design philosophy is:
**"Maximum power for developers, minimum exposure to the system."**

This document defines the full **Telepipe Security Model**, including:

- security boundaries
- allowed and disallowed operations
- trust assumptions
- process-level isolation
- attack surface analysis
- sandboxing constraints
- privilege constraints
- safe defaults
- optional hardened modes (future)

Telepipe is intentionally simple, deterministic, and safe by design.

---

# 1. Security Philosophy

Telepipe is designed with **zero trust in the environment** and **no assumption of elevated permissions**.

Goals:

- No privileged ports
- No privilege escalation
- No access to system-wide networking
- No persistence beyond user home directory
- No unsafe IPC mechanisms
- No ability to impersonate users or system daemons
- No injection into existing processes

Telepipe corrects an important misconception:
**It does not become "part of" the child process.**
Instead, it only redirects stdio and/or attaches TCP streams.

This design drastically reduces the attack surface.

---

# 2. Security Boundary

Telepipe operates under the following boundaries:

| Category | Allowed | Not Allowed |
|----------|---------|-------------|
| Local filesystem | Read/write only inside `$HOME/.telepipe` | Writes outside this dir without user instruction |
| Network | TCP connections to `127.0.0.1` only | Anything on non-loopback unless explicitly configured |
| Ports | 49152–65535 | Privileged ports (<1024), system service ports |
| Processes | Spawn child, supervise child | Attach to arbitrary existing processes (except TCP) |
| Authentication | None | Credential prompting, privilege elevation |
| Environment | Uses user's environment | Never modifies global shell state |

Telepipe's "blast radius" is strictly bounded.

---

# 3. Privilege Model

Telepipe:

- MUST run as a normal, unprivileged user
- MUST refuse to run under UID 0 (root) unless `--allow-root` is explicitly set
- MUST avoid any system-level escalations
- MUST not use `sudo`, setuid, or capabilities
- MUST not modify system-wide networking

### Why?

Because redirect and connect modes only require:

- TCP sockets
- process spawning
- stdio redirection
- file locking

All possible without privileges.

---

# 4. Filesystem Security

## 4.1 Session Dictionary Directory

Telepipe stores all sessions in:

```
$HOME/.telepipe/sessions/
```

Security requirements:

- directory must be owned by the user
- permissions must be `700`
- session files must be `600`
- locks must be `600` or `644` (OS-dependent)

Telepipe must fail with:

```
ERROR E-DICT-CORRUPT: session dictionary corrupt
```

if permissions are loose or ownership invalid.

---

## 4.2 No Global State

Telepipe must **never** write to:

- `/var/run`
- `/tmp` (except temp file for atomic fsync rename)
- `/usr/local/...`
- system config directories

**All state is localized** to the user's home directory.

---

## 4.3 No Symlink Traversal

Telepipe must:

- refuse to follow symbolic links inside `$HOME/.telepipe`
- treat symlinks as dictionary corruption

This prevents:

- directory escape
- manipulation of supervisor control
- symlink-based temp file attacks

Telepipe does NOT allow "symlink-based redirection" tricks.

---

# 5. Process Security

## 5.1 Child Process Isolation

Telepipe spawns children **exactly** as the user would, except:

- stdio is redirected
- supervisors are inserted

Telepipe does NOT:

- modify the child's filesystem rights
- inject into the child
- tamper with its process credentials
- sandbox the child
- elevate its privileges

Telepipe is not a sandboxing tool.
It is purely a redirection + supervision tool.

---

## 5.2 Supervisor Isolation

Supervisors:

- never run as root
- hold no elevated permissions
- never modify environment variables
- never modify PATH
- never impersonate child processes
- never take ownership of parent's TTY
- avoid interacting with user's keystrokes

Supervisors exist only to:

- multiplex I/O
- preserve streaming
- detect child death

## 5.3 PID Spoofing Resistance

Telepipe validates PIDs using:

- `/proc/<pid>` checks (Linux)
- `kill -0` (POSIX)
- process table queries (macOS)

PID reuse is handled safely by:

- checking FD/TCP state
- verifying port bindings
- refusing to interact with mismatched processes

---

# 6. Networking Security

## 6.1 Localhost Only by Default

Telepipe binds to:

```
127.0.0.1:<ephemeral>
```

This ensures:

- no exposure to LAN
- no exposure to WAN
- no need for firewall rules
- no remote code execution risk
- no unintended sharing of stdout/stderr

## 6.2 Explicit Non-Localhost Binding

Telepipe must require:

```
--host <IP>
```

to accept anything other than `127.0.0.1`.

If the host is not loopback:

- print a warning
- require confirmation (future)

---

# 7. TCP Session Security

## 7.1 No Frame Parsing

Telepipe supervisors are **binary-blind**:
- no protocol parsing
- no transformation of bytes
- no injection
- no stripping

This drastically reduces the attack surface.

## 7.2 No MITM Positioning

Telepipe does not position itself as a proxy.
It simply:

- receives child output
- writes it to TCP
- receives TCP input
- writes it to child stdin

**No logic interprets or alters the data.**

---

# 8. Locking Security

Telepipe uses locks only for preventing state corruption.
Locks are not security barriers.

Rules:

- lockfiles must never be world-writeable
- lock contention must be bounded
- lockfiles must be inside the Telepipe directory only
- atomic creation limits symlink attacks

---

# 9. Session Integrity

Telepipe enforces:

- atomicity of writes
- fsync before rename
- validation of session files before reading
- stale entry detection
- supervisory PID verification

Any mismatch → session drop.

This prevents:

- session impersonation
- session spoofing
- stale resurrected state
- accidental privilege escalation

---

# 10. Memory Safety (Implementation-Level)

Telepipe core must avoid:

- use-after-free
- double-free
- FD leaks
- integer overflows
- buffer overflows
- unsanitized user-controlled buffer sizes

Rust implementation (recommended) eliminates most classes automatically.

C/C++ implementations must:

- use bounded copies
- avoid raw pointer arithmetic
- ensure FD lifecycle correctness
- use RAII wrappers

---

# 11. Safe Defaults

Telepipe chooses defaults that eliminate risks:

- redirect binds TCP only to localhost
- connect binds only to localhost
- no remote endpoints reachable without explicit override
- no multi-user sharing
- no integrating with systemd
- no global PATH modifications
- no root requirements

Telepipe is safe for environments with:

- multiple users
- shared servers
- sensitive pipelines
- corporate lockdown
- DevOps CI
- AI agents with partial file system access

---

# 12. Attack Surface Summary

Telepipe's attack surface consists solely of:

- local process spawning
- local TCP connections to localhost
- a directory inside the user's home

Telepipe has **zero network attack surface** unless explicitly exposed.

---

# 13. Future Hardened Modes (Optional)

Telepipe may introduce:

- `--jail <path>` — chroot-like directory confinement
- `--tls` — TLS-wrapped local TCP sockets
- `--uid-map` — user namespace restrictions
- `--rate-limit` — throttle exec-mode streaming
- `--privdrop` — drop privileges if child runs elevated

These are **not** part of the Telepipe Core but reserved for future capability.

---

# 14. Summary

Telepipe's security model ensures:

- no privileged operations
- no remote exposure
- no system-level side effects
- no dictionary poisoning
- no protocol parsing vulnerabilities
- no escalations
- no injection
- no sandbox escapes
- no symlink attacks
- no hidden backdoors

**Telepipe is inherently safe because it is inherently simple.**
It does not interpret data.
It does not elevate privileges.
It does not expose services to the outside world.

This is the complete Telepipe Security Model.
