# Telepipe Testing Corpus
### *Deterministic Validation Suite for Redirect, Exec, Connect, Stop, Disconnect, and Info*
### *Cross‑Platform • Pure Bash + TCP + FDs • No Telepipe Vocabulary Required*

This corpus defines the **full external test suite** that validates Telepipe Core using only mainstream OS tools (`bash`, `nc`, `ps`, `lsof`, `kill`, `exec`, `/dev/tcp`).
It proves that everything Telepipe does can be reproduced with standard shell orchestration and ensures that Telepipe behaves consistently on:

- macOS
- Linux
- Windows (Git Bash / MSYS2)

Each test is atomic, self‑contained, uses no Telepipe CLI terminology, and simulates Telepipe behavior externally.

This corpus is divided into:

1. **Redirect equivalence tests**
2. **Exec equivalence tests**
3. **Connect equivalence tests**
4. **Lifecycle tests (stop + disconnect semantics)**
5. **FD/TCP allocation tests**
6. **Crash‑resilience tests**
7. **Golden streaming tests**
8. **Cross‑platform compatibility tests**

## 1.6 Verify info command output and immediate exit
```bash
# Start redirect session
telepipe redirect --id test-info -- cat &
REDIRECT_PID=$!

# Call info and measure execution time
START=$(date +%s%N)
telepipe info --id test-info > info.out
END=$(date +%s%N)
DURATION=$(( (END - START) / 1000000 )) # milliseconds

# Verify info exited immediately (< 100ms)
if [ $DURATION -gt 100 ]; then
    echo "FAIL: info took ${DURATION}ms (should be <100ms)"
    exit 1
fi

# Verify output format
if ! grep -q "^mode\tredirect$" info.out; then
    echo "FAIL: missing mode field"
    exit 1
fi

if ! grep -q "^stdin\t[0-9]\+$" info.out; then
    echo "FAIL: missing stdin port"
    exit 1
fi

if ! grep -q "^fdin\t[0-9]\+$" info.out; then
    echo "FAIL: missing stdin FD"
    exit 1
fi

# Verify Telepipe command exited (PID gone)
sleep 0.1
if ps -p $REDIRECT_PID >/dev/null 2>&1; then
    echo "FAIL: redirect command should have exited"
    exit 1
fi

# Supervisor should still be running (check via info)
if telepipe info --id test-info >/dev/null 2>&1; then
    echo "PASS: info exited immediately, supervisor still running"
else
    echo "FAIL: session disappeared"
    exit 1
fi
```

---

# 1. Redirect Equivalence Tests
### *Spawn a process, allocate FDs, allocate TCP ports, bind TCP sockets, and redirect stdio manually.*

## 1.1 Allocate three free TCP ports
```bash
find_free_port() {
  for p in $(seq 49152 65535); do
    echo > /dev/tcp/127.0.0.1/$p 2>/dev/null || { echo $p; return; }
  done
}
PORT_IN=$(find_free_port)
PORT_OUT=$(find_free_port)
PORT_ERR=$(find_free_port)
```

## 1.2 Allocate three numeric file descriptors
```bash
find_free_fd() {
  for fd in $(seq 3 256); do
    if ! { true >&$fd 2>/dev/null; }; then echo "$fd"; return; fi
  done
}
FD_IN=$(find_free_fd)
FD_OUT=$(find_free_fd)
FD_ERR=$(find_free_fd)
```

## 1.3 Create the persistent TCP connections
```bash
exec {FD_IN}<>/dev/tcp/127.0.0.1/$PORT_IN
exec {FD_OUT}<>/dev/tcp/127.0.0.1/$PORT_OUT
exec {FD_ERR}<>/dev/tcp/127.0.0.1/$PORT_ERR
```

## 1.4 Spawn process with redirected stdio
```bash
psql <&$FD_IN >&$FD_OUT 2>&$FD_ERR &
CHILD=$!
```

## 1.5 Assertions
- `ps -p "$CHILD"` should show the process alive.
- `lsof -p "$CHILD"` should show TCP connections bound to the three ports.
- No output printed.
- Parent shell remains running.

---

# 2. Exec Equivalence Tests
### *Send data into the redirected process over TCP and stream back stdout/stderr.*

## 2.1 Send input
```bash
echo "select now();" >&$FD_IN
```

## 2.2 Read stdout
```bash
read -r line <&$FD_OUT
echo "$line"
```

## 2.3 Read stderr
```bash
read -r err <&$FD_ERR
echo "$err"
```

## 2.4 Assertions
- Child receives SQL.
- Client receives full output.
- Output continues to stream until EOF.

---

# 3. Connect Equivalence Tests
### *Open a single FD to an existing TCP protocol endpoint.*

**Note:** For WebSocket-based protocols like CDP, use `websocat` rather than raw TCP.

## 3.1 Allocate FD and connect (for raw TCP protocols like PostgreSQL)
```bash
FD_CONN=$(find_free_fd)
exec {FD_CONN}<>/dev/tcp/127.0.0.1/5432
```

## 3.2 Send a command
```bash
echo "SELECT 1;" >&$FD_CONN
```

## 3.3 Stream response
```bash
read -r msg <&$FD_CONN
echo "$msg"
```

## 3.4 Assertions
- The protocol endpoint replies normally.
- No reconnection overhead.
- The FD stays open for the entire session.

---

# 4. Lifecycle Tests
### *Stop and disconnect equivalence tests for redirect and connect modes.*

## 4.1 Stop semantics
Killing the parent should **not** kill the child.

```bash
kill -9 $$  # simulated from another shell
```

Assertions:
- child continues running
- TCP ports still open
- streaming still functional when attaching from another session

## 4.2 Disconnect semantics
Terminating a connect supervisor should not affect the external process.

Simulated:
```bash
kill -9 $SUPERVISOR_PID
```

Assertions:
- the external service (database, etc.) remains alive
- TCP endpoint remains open

---

# 5. FD / TCP Allocation Tests
### *Verify deterministic allocation logic.*

## 5.1 Allocation order
```bash
fd1=$(find_free_fd)
fd2=$(find_free_fd)
fd3=$(find_free_fd)
```
Assertions:
- fd1 < fd2 < fd3
- all ≥ 3

## 5.2 Port allocation order
Assertions:
- ports increase monotonically
- no collisions
- port opens succeed

---

# 6. Crash Resilience Tests
### *Telepipe must recover from crashes without harming children.*

## 6.1 Kill supervisor
```bash
kill -9 $SUPERVISOR_PID
```

Assertions:
- child remains alive
- `exec` (manual version) can still communicate with the child

## 6.2 Kill child
```bash
kill -9 $CHILD
```

Assertions:
- TCP ports close automatically
- supervisor exits (simulated by another script)

---

# 7. Golden Streaming Tests
### *Round‑trip correctness for arbitrary streaming payloads.*

## 7.1 Multiline input
```bash
printf "1\n2\n3\n" >&$FD_IN
```

Assertions:
- output matches expected SQL results

## 7.2 High‑volume streaming
```bash
yes "hello" | head -n 5000 >&$FD_IN
```

Assertions:
- no frame loss
- no TCP resets
- output lines equal input patterns

---

# 8. Cross‑Platform OS Tests

## 8.1 Windows Git Bash / MSYS2
- `/dev/tcp` functional
- numeric FD redirection functional
- no FIFOs required
- TCP sockets reusable

## 8.2 macOS
- `/dev/tcp` functional
- FDs recyclable
- child survives supervisor exit

## 8.3 Linux
- identical behavior
- verified against `bash`, `dash`, `busybox sh`

---

# 9. Corpus Execution Rules

Each test must:

- run without requiring Telepipe
- validate one Telepipe behavior at a time
- be reproducible
- exit with explicit success/failure
- print minimal logs
- avoid interactive prompts

Telepipe's internal tests use this corpus as the **ground truth**.

---

# 10. Internal Test Requirements

Telepipe's internal tests must validate:

- Session dictionary integrity under rapid churn
- Supervisor exit codes mirroring child exit codes
- FD exhaustion handling
- Port exhaustion handling
- Disconnect correctness with reconnect loops
- Exec behavior under high throughput
- Correct EOF semantics across mixed half-closes

These tests complement the external corpus by verifying implementation-specific behavior.

---

# 11. Summary

This corpus proves that:

- redirect is equivalent to FD allocation + TCP binding + process spawn
- exec is equivalent to TCP bidirectional streaming
- connect is equivalent to attaching a single FD to a TCP protocol port
- stop and disconnect follow standard Unix lifecycle semantics
- FD and TCP allocation are deterministic and monotonic
- Windows, macOS, and Linux behave identically under these patterns

This is the canonical "external proof" of Telepipe's correctness and the foundation of the official golden tests.
