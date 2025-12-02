# Telepipe Error Codes

Complete reference for all Telepipe error codes. Every error is deterministic, machine-readable, and actionable.

---

## Success

### 0 - E-OK
Operation completed successfully.

---

## Allocation Errors (20-29)

### 21 - E-ALLOC-FD
**Description:** No free numeric file descriptors available.

**Cause:** FD scan (3-255) failed to find available descriptors.

**Solution:**
```bash
# Close unused sessions
telepipe disconnect --id <session-id>
telepipe stop --id <session-id>

# Check active sessions
telepipe info

# Increase system limit
ulimit -n 4096
```

**Example:**
```bash
telepipe redirect --id myapp -- node server.js
# Error: E-ALLOC-FD (21)
```

---

### 22 - E-ALLOC-PORT
**Description:** No free TCP ports available in ephemeral range.

**Cause:** Port scan (49152-65535) failed - all ports in use.

**Solution:**
```bash
# Close unused sessions
telepipe disconnect --id <session-id>

# Check active sessions
telepipe info

# Wait for TIME_WAIT ports to release
```

**Example:**
```bash
telepipe connect --id db --port 5432
# Error: E-ALLOC-PORT (22)
```

---

### 23 - E-ALLOC-DICT
**Description:** Session dictionary cannot be created or updated.

**Cause:** Permissions issue or filesystem corruption in `~/.telepipe/sessions/`.

**Solution:**
```bash
# Check directory permissions
ls -la ~/.telepipe/sessions/

# Fix permissions
chmod 700 ~/.telepipe/sessions/
```

---

## File Descriptor Errors (30-39)

### 31 - E-FD-BIND
**Description:** Failed to bind numeric FD to TCP socket.

**Cause:** System resource exhaustion or invalid FD state.

**Solution:**
```bash
# Check system resources
ulimit -a

# Restart session
telepipe stop --id <session-id>
telepipe redirect --id <session-id> -- <command>
```

---

### 32 - E-FD-REDIRECT
**Description:** Failed to redirect stdio of child process.

**Cause:** Child process spawn failed during FD setup.

**Solution:**
```bash
# Verify command exists
which <command>

# Check command permissions
ls -la <command-path>
```

---

## TCP Errors (40-49)

### 41 - E-TCP-CONNECT
**Description:** Failed to establish TCP connection.

**Cause:** Service not running, wrong port, or connection refused.

**Solution:**
```bash
# Verify service is running
curl http://127.0.0.1:<port>

# Check what's on the port
lsof -i :<port>

# Start the service if needed
```

**Example:**
```bash
telepipe connect --id db --port 5432
# Error: E-TCP-CONNECT (41)
# PostgreSQL is not running
```

---

### 42 - E-TCP-BROKEN
**Description:** TCP connection broke during streaming.

**Cause:** Remote service crashed or network interruption.

**Solution:**
```bash
# Reconnect
telepipe disconnect --id <session-id>
telepipe connect --id <session-id> --port <port>
```

---

### 43 - E-TCP-RESET
**Description:** Remote endpoint forcibly closed connection.

**Cause:** Service reset or firewall intervention.

**Solution:**
```bash
# Check service status
# Reconnect when service is stable
```

---

### 44 - E-TCP-HALF
**Description:** Unexpected half-close of TCP stream.

**Cause:** Stdout or stderr closed prematurely.

**Note:** Normal half-close patterns are not errors. This only occurs for unexpected ordering.

---

## Process Errors (50-59)

### 51 - E-PROC-SPAWN
**Description:** Failed to spawn child process.

**Cause:** Command not found, permission denied, or resource exhaustion.

**Solution:**
```bash
# Verify command exists
which <command>

# Check permissions
ls -la <command-path>

# Try running directly
<command>
```

**Example:**
```bash
telepipe redirect --id myapp -- nonexistent-command
# Error: E-PROC-SPAWN (51)
```

---

### 52 - E-PROC-SUPERVISOR
**Description:** Supervisor encountered unrecoverable internal error.

**Cause:** Critical supervisor failure.

**Solution:**
```bash
# Stop and restart the session
telepipe stop --id <session-id>
telepipe redirect --id <session-id> -- <command>
```

---

### 53 - E-PROC-NOCHILD
**Description:** Child PID in dictionary but process doesn't exist.

**Cause:** Child process terminated unexpectedly.

**Solution:**
```bash
# Clean up stale session
telepipe stop --id <session-id>
```

---

### 54 - E-PROC-EXITCODE
**Description:** Child returned non-zero exit code unexpectedly.

**Note:** Reserved for future use. Currently, supervisors exit with child's exit code.

---

## Dictionary Errors (60-69)

### 61 - E-DICT-MISSING
**Description:** Session not found.

**Cause:** Session ID doesn't exist in dictionary.

**Solution:**
```bash
# List all sessions
telepipe info

# Use correct session ID
telepipe exec --id <correct-id>
```

**Example:**
```bash
telepipe exec --id nonexistent
# Error: E-DICT-MISSING (61)
```

---

### 62 - E-DICT-CORRUPT
**Description:** Session dictionary entry is malformed.

**Cause:** Dictionary file corruption.

**Solution:**
```bash
# Remove corrupt entry manually
rm ~/.telepipe/sessions/<session-id>.json

# Or recreate the session
```

---

### 63 - E-DICT-STALE
**Description:** Entry references dead PIDs or closed FDs.

**Cause:** Supervisor crashed without cleanup.

**Solution:**
```bash
# Recovery happens automatically on next command
# Or manually clean up
telepipe stop --id <session-id>
```

---

### 64 - E-DICT-CONFLICT
**Description:** Session ID already exists.

**Cause:** Attempting to create session with duplicate ID.

**Solution:**
```bash
# Use different ID
telepipe redirect --id myapp2 -- node server.js

# Or disconnect existing
telepipe disconnect --id myapp
telepipe redirect --id myapp -- node server.js
```

**Example:**
```bash
telepipe redirect --id myapp -- node server.js
telepipe redirect --id myapp -- python app.py  # Conflict!
# Error: E-DICT-CONFLICT (64)
```

---

### 68 - E-INFO-READ
**Description:** Info mode could not read session dictionary entry.

**Cause:** Dictionary file locked, corrupted, or deleted during read.

**Note:** Rare error - info is read-only.

---

## CLI Errors (70-79)

### 71 - E-CLI-ARGS
**Description:** Invalid flag, missing required flag, or incorrect argument.

**Solution:**
```bash
# Check command syntax
telepipe --help
telepipe <command> --help
```

**Example:**
```bash
telepipe redirect  # Missing --id
# Error: E-CLI-ARGS (71)
```

---

### 72 - E-CLI-MODE
**Description:** Combination of flags doesn't match any known behavior.

**Solution:**
```bash
# Use valid flag combinations
telepipe --help
```

---

### 73 - E-CLI-ID
**Description:** Invalid session identifier.

**Cause:** Session ID contains illegal characters or is empty.

**Valid characters:** Alphanumeric, `-`, `_`

**Example:**
```bash
telepipe redirect --id "my app" -- node server.js  # Space not allowed
# Error: E-CLI-ID (73)
```

---

### 74 - E-CLI-INVALID-HOST
**Description:** Invalid host or IP address provided.

**Solution:**
```bash
# Use valid IP address
telepipe connect --id db --port 5432 --host 127.0.0.1
```

---

### 75 - E-CLI-INVALID-FD
**Description:** User-provided FD override is invalid.

**Cause:** FD out of range (3-255) or already in use.

**Solution:**
```bash
# Let Telepipe auto-allocate FDs
telepipe redirect --id myapp -- node server.js
```

---

### 76 - E-CLI-INVALID-PORT
**Description:** User-provided port override is invalid.

**Cause:** Port out of range (1-65535) or already in use.

**Solution:**
```bash
# Use valid port number
telepipe connect --id db --port 5432
```

---

## Exec Errors (80-89)

### 81 - E-EXEC-ALREADY-ACTIVE
**Description:** Another exec is connected to this session.

**Cause:** Telepipe uses one-at-a-time execution model. Only ONE exec can be active per session at any time.

**This is expected behavior!** Not a bug.

**Solution:**
```bash
# Wait for current exec to complete
# Then retry
echo "command" | telepipe exec --id session

# Or use separate sessions for concurrent workers
telepipe redirect --id worker1 -- websocat ...
telepipe redirect --id worker2 -- websocat ...
```

**Example:**
```bash
# Terminal 1
echo "long running query" | telepipe exec --id db

# Terminal 2 (while Terminal 1 is still running)
echo "SELECT 1" | telepipe exec --id db
# Error: E-EXEC-ALREADY-ACTIVE (81)
```

**Why this design?**
- Prevents response mixing
- Guarantees deterministic ordering
- Keeps Telepipe simple and predictable

---

## Exit Code Reference Table

| Code | Error | Description |
|------|-------|-------------|
| **0** | Success | Operation completed |
| **21** | E-ALLOC-FD | No available file descriptors |
| **22** | E-ALLOC-PORT | No available TCP ports |
| **23** | E-ALLOC-DICT | Cannot write session dictionary |
| **31** | E-FD-BIND | Cannot bind FD to TCP socket |
| **32** | E-FD-REDIRECT | Cannot redirect child stdio |
| **41** | E-TCP-CONNECT | Cannot establish TCP connection |
| **42** | E-TCP-BROKEN | TCP connection broke unexpectedly |
| **43** | E-TCP-RESET | Remote endpoint reset connection |
| **44** | E-TCP-HALF | Unexpected half-close of stream |
| **51** | E-PROC-SPAWN | Cannot spawn child process |
| **52** | E-PROC-SUPERVISOR | Supervisor internal failure |
| **53** | E-PROC-NOCHILD | Child process not found |
| **54** | E-PROC-EXITCODE | Unexpected child exit code |
| **61** | E-DICT-MISSING | Session not found |
| **62** | E-DICT-CORRUPT | Session dictionary corrupt |
| **63** | E-DICT-STALE | Session entry is stale |
| **64** | E-DICT-CONFLICT | Session ID already exists |
| **68** | E-INFO-READ | Cannot read session info |
| **71** | E-CLI-ARGS | Invalid or missing arguments |
| **72** | E-CLI-MODE | Invalid mode or incompatible flags |
| **73** | E-CLI-ID | Invalid session identifier |
| **74** | E-CLI-INVALID-HOST | Invalid host or IP address |
| **75** | E-CLI-INVALID-FD | Invalid file descriptor |
| **76** | E-CLI-INVALID-PORT | Invalid TCP port |
| **81** | E-EXEC-ALREADY-ACTIVE | Another exec is connected |

---

## Error Message Format

All errors follow this format:
```
ERROR <CODE>: <short description>
```

Example:
```
ERROR E-ALLOC-PORT: unable to allocate TCP port
```

For debug information, use `--debug`:
```bash
telepipe redirect --id myapp --debug -- node server.js
```

Debug output appears on stderr and includes timestamps, PIDs, allocated resources, and additional context.
