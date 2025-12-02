# Telepipe State Recovery Algorithms
### *Crash-Safe • Deterministic • Self-Healing • Cross-Platform*

This document defines the **Telepipe State Recovery Algorithms** — the logic Telepipe uses to recover session state after unexpected termination, OS crashes, supervisor crashes, stale dictionary entries, or corrupted references.

Telepipe state recovery ensures:

- No orphaned supervisors remain.
- No stale FD/TCP allocations persist.
- Redirect sessions with living children are recovered.
- Connect sessions with reachable endpoints are recovered.
- Broken or corrupted sessions are discarded immediately.
- Recovery is **idempotent**, **atomic**, and **safe to run on startup every time**.

---

# 1. Overview

Telepipe's session state is persisted in a **dictionary** (a directory of files).
When Telepipe starts or when any Telepipe command executes, Telepipe performs:

- **Dictionary Scan**
- **Validation Pass**
- **Repair Pass**
- **Eviction Pass**
- **Optional Reattach Pass** (redirect mode only; optional future enhancement)

This is called the **Telepipe Recovery Pipeline**:

```
SCAN → VALIDATE → REPAIR → EVICT → (OPTIONAL REATTACH)
```

Recovery must run **before** any command except `telepipe info --id`.

---

# 2. Dictionary Scan

Telepipe scans the session dictionary directory:

```
$HOME/.telepipe/sessions/<ID>.json
```

For each file:

- parse JSON safely
- ensure required fields are present
- collect into in-memory working set

Failures at this stage are handled by **E-DICT-CORRUPT**.

---

# 3. Validation Pass

For each session entry, Telepipe performs the following validations:

## 3.1 Required Field Validation
A valid record must include:

```
id
mode
supervisor_pid
child_pid (redirect) or null (connect)
stdin_port
stdout_port
stderr_port
stdin_fd
stdout_fd
stderr_fd
connect_fd
host
```

If fields are missing or malformed:

→ **mark entry as corrupt**
→ will be removed in the eviction pass

---

## 3.2 PID Validation

### 3.2.1 Supervisor PID Check
Telepipe checks:

```
process_exists(supervisor_pid)
```

If false:

- If **mode = redirect** → child may still be alive
- If **mode = connect** → no supervisor means session cannot be recovered

Mark for special handling:

| Mode       | Supervisor missing | Action |
|------------|--------------------|--------|
| redirect   | yes                | candidate for REATTACH |
| connect    | yes                | remove in eviction |

---

### 3.2.2 Child PID Check (redirect mode)

```
process_exists(child_pid)
```

If false:

- redirect session is dead
- remove in eviction pass

If true:

- mark session as recoverable
- child process still running

---

# 4. Socket Validation

## 4.1 TCP Port Availability
Telepipe checks if ports for:

- stdin_port
- stdout_port
- stderr_port
(or connect_port)

…are currently:

- in LISTEN state (for redirect supervisor)
- or in ESTABLISHED (for connect supervisor)

If ports are free:

- supervisor is gone
- child may still be alive
- this is a **recoverable redirect** or a **dead connect session**

## 4.2 FD Validity
FDs stored in the dictionary cannot be reused because they belonged to the supervisor that crashed.

Telepipe **never** trusts FDs stored on disk.

→ All recovered supervisors must allocate fresh FDs.

---

# 5. Repair Pass

## 5.1 Repair of Redirect Sessions with Living Children

A redirect session is recoverable if:

- child process is alive
- supervisor process is dead
- TCP ports are not currently in use

Telepipe must:

1. allocate fresh FDs
2. **attempt to re-bind** new FDs to existing child's TCP ports
3. **spawn a new supervisor**
4. update the dictionary with new supervisor PID and FD values
5. validate the bindings succeeded

If binding fails (race conditions, port reuse):

→ mark session as stale → remove in eviction pass

---

## 5.2 Repair of Connect Sessions

Connect sessions **cannot** be repaired because they require:

- original persistent TCP connection
- original FD state
- original supervisor state

If the supervisor is dead:

→ remove the session.

---

## 5.3 Repair of Corrupted Records

If dictionary entries are missing fields that can be reconstructed (rare), Telepipe may attempt:

- PID rebinding
- port probing
- host inference

If reconstruction fails:

→ remove record.

---

# 6. Eviction Pass

Entries must be removed if:

- JSON invalid
- missing fields
- stale supervisor in connect-mode
- dead child in redirect-mode
- TCP ports already reused by another process
- unsafe to recover
- inconsistent with filesystem
- inconsistent with OS process table

Evicted entries must be:

- deleted from dictionary
- reported only under `--debug` mode
- not printed during normal Telepipe usage

No partial state may remain.

---

# 7. Optional Reattach Pass (Redirect Mode)

This is a **future-protocol** for full robustness.

Telepipe may reattach to a redirect child that remains alive:

```
redirect session crashed → new supervisor binds → redirect session restored
```

Steps:

1. allocate fresh FDs
2. rebind to the known TCP ports
3. spawn a replacement supervisor process
4. update dictionary
5. validate by sending a probe (e.g., newline)

Success:
- redirect session fully restored
- child continues uninterrupted

Failure:
- child is still alive but unbindable
- remove record to avoid ghost state

This reattach logic is optional but recommended.

---

# 8. Recovery Trigger Points

Telepipe must run recovery:

- at startup
- before every Telepipe command except `info`
- after any supervisor exits
- after stop/disconnect operations
- whenever dictionary appears inconsistent
- when requested explicitly (future `telepipe recover`)

**Rationale:** Info is read-only and should be fast. Recovery can be expensive with large dictionaries. Info skips recovery to provide instant feedback on session state.

Recovery must be seamless; user should not notice it.

---

# 9. Recovery Algorithm (Pseudocode)

```
recover():
    entries = scan_dictionary()

    for each entry in entries:
        if corrupt(entry):
            mark_for_eviction(entry)
            continue

        if entry.mode == "redirect":
            recover_redirect(entry)
        else if entry.mode == "connect":
            recover_connect(entry)

    evict_marked_entries()
    save_dictionary()
```

### recover_redirect(entry):
```
child_alive = process_exists(entry.child_pid)
supervisor_alive = process_exists(entry.supervisor_pid)

if supervisor_alive and child_alive:
    # nothing to repair
    return OK

if not child_alive:
    mark_for_eviction(entry)
    return

# supervisor dead, child alive → attempt reattach
fresh_fds = allocate_fresh_fds(3)
if !bind_fds_to_ports(fresh_fds, entry.ports):
    mark_for_eviction(entry)
    return

new_supervisor = spawn_supervisor(fresh_fds, entry.child_pid)
update_entry(entry, new_supervisor_pid=fresh)
return OK
```

### recover_connect(entry):
```
if process_exists(entry.supervisor_pid):
    return OK

mark_for_eviction(entry)
```

---

# 10. Safety & Atomicity Guarantees

- No dictionary writes occur until the **eviction pass** completes.
- Recovery is fully **idempotent**.
- A crash during recovery leads to a fresh recovery pass on next invocation.
- No recovery step may produce partial state.
- State is replaced atomically (write temp → fsync → rename).

---

# 11. Summary

Telepipe State Recovery Algorithms guarantee:

- redirect sessions survive Telepipe crashes
- connect sessions are cleaned when supervisors die
- stale, corrupt, or inconsistent dictionary entries are safely discarded
- redirect-mode repair preserves child processes across disasters
- recovery is automatic, silent, and predictable
- all operations are cross-platform and deterministic

This algorithmic layer gives Telepipe its **fault tolerance**, **crash resilience**, and **long-lived session reliability**.
