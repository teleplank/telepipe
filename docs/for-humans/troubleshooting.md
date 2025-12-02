# Troubleshooting Telepipe

Solutions to common issues. If it's broken, it's probably here.

---

## Quick Diagnostics

First, check the basics:

```bash
# Is Telepipe installed?
telepipe --help

# List all sessions
telepipe info

# Check specific session
telepipe info --id <session-id>

# What's on a port?
lsof -i :<port>

# System limits
ulimit -a
```

---

## Common Issues

### "Port already in use"

**Error code:** 22 (E-ALLOC-PORT)

**Cause:** All ephemeral ports (49152-65535) allocated.

**Solution:**
```bash
# Check active sessions
telepipe info

# Disconnect unused sessions
telepipe disconnect --id <session-id>
telepipe stop --id <session-id>

# Wait for TIME_WAIT ports to release (up to 60 seconds)
```

---

### "Connection refused"

**Error code:** 41 (E-TCP-CONNECT)

**Causes:**
- Service not running
- Wrong port number
- Firewall blocking

**Solution:**
```bash
# Verify service is running
curl http://127.0.0.1:<port>
lsof -i :<port>

# Start service if needed
# For Chrome: google-chrome --remote-debugging-port=9222
# For PostgreSQL: brew services start postgresql
# For Redis: brew services start redis
```

---

### "Session not found"

**Error code:** 61 (E-DICT-MISSING)

**Cause:** Session ID doesn't exist.

**Solution:**
```bash
# List all sessions
telepipe info

# Use correct session ID
telepipe exec --id <correct-id>

# Create session first if needed
telepipe redirect --id myapp -- node server.js
```

---

### "Session already exists"

**Error code:** 64 (E-DICT-CONFLICT)

**Cause:** Trying to create session with duplicate ID.

**Solution:**
```bash
# Use different ID
telepipe redirect --id myapp2 -- node server.js

# Or disconnect existing first
telepipe disconnect --id myapp
telepipe redirect --id myapp -- node server.js
```

---

### "FD exhaustion"

**Error code:** 21 (E-ALLOC-FD)

**Cause:** System file descriptor limit reached.

**Solution:**
```bash
# Close unused sessions
telepipe disconnect --id <session-id>
telepipe stop --id <session-id>

# Check current limit
ulimit -n

# Increase limit (temporary)
ulimit -n 4096

# Increase limit (permanent) - add to ~/.bashrc or ~/.zshrc
# ulimit -n 4096
```

---

### "Exec already active"

**Error code:** 81 (E-EXEC-ALREADY-ACTIVE)

**This is expected behavior!** Not a bug.

**Cause:** Telepipe uses one-at-a-time execution model. Only one exec per session at a time.

**Solution:**
```bash
# Wait for current exec to complete, then retry

# OR use separate sessions for concurrent work
telepipe redirect --id worker1 -- node app.js
telepipe redirect --id worker2 -- node app.js

# Now you can exec in parallel
echo "cmd1" | telepipe exec --id worker1 &
echo "cmd2" | telepipe exec --id worker2 &
```

---

### Process won't stop

**Cause:** Process not responding to SIGTERM.

**Solution:**
```bash
# telepipe stop sends SIGTERM, then SIGKILL
telepipe stop --id <session-id>

# If that doesn't work, find and kill manually
telepipe info --id <session-id>  # Get PID
kill -9 <pid>
```

---

## Chrome-Specific Issues

### Chrome connection fails

**Common causes:**
1. Chrome not started with `--remote-debugging-port`
2. websocat not installed
3. Wrong WebSocket URL

**Solution:**
```bash
# 1. Start Chrome with debugging
google-chrome --remote-debugging-port=9222 --headless=new

# 2. Install websocat
brew install websocat  # macOS

# 3. Get correct WebSocket URL
curl http://127.0.0.1:9222/json/version
WS_URL=$(curl -s http://127.0.0.1:9222/json/version | grep -o 'ws://[^"]*')

# 4. Connect properly
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

See: [chrome-devtools.md](chrome-devtools.md)

---

### "websocat: command not found"

**Cause:** websocat not installed.

**Solution:**
```bash
# macOS
brew install websocat

# Linux
sudo apt install websocat

# Or via cargo
cargo install websocat
```

---

### CDP commands return nothing

**Cause:** websocat missing `--no-close` or `--text` flags.

**Solution:**
```bash
# Must use both flags
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

---

### Session dies after first command

**Cause:** websocat without `--no-close`.

**Solution:**
```bash
# Recreate with correct flags
telepipe stop --id chrome
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

---

## Database-Specific Issues

### PostgreSQL connection fails

**Common causes:**
1. PostgreSQL not running
2. `pg_hba.conf` doesn't allow connections
3. Wrong credentials

**Solution:**
```bash
# Check if running
pg_isready

# Start if needed
brew services start postgresql  # macOS
sudo systemctl start postgresql  # Linux

# Check pg_hba.conf allows localhost connections
# Add: local all all trust
```

---

### MySQL connection fails

**Common causes:**
1. MySQL not running
2. Permission denied

**Solution:**
```bash
# Check if running
mysqladmin ping

# Start if needed
brew services start mysql  # macOS
sudo systemctl start mysql  # Linux

# Grant permissions
mysql -u root -p
GRANT ALL ON *.* TO 'user'@'localhost';
```

---

### Redis connection fails

**Solution:**
```bash
# Check if running
redis-cli ping

# Start if needed
brew services start redis  # macOS
sudo systemctl start redis  # Linux
```

See: [database-connections.md](database-connections.md)

---

## Installation Issues

### "command not found: telepipe"

**Cause:** npm global bin not in PATH.

**Solution:**
```bash
# Find npm prefix
npm config get prefix

# Add to PATH (in ~/.bashrc or ~/.zshrc)
export PATH="$(npm config get prefix)/bin:$PATH"

# Reload shell
source ~/.bashrc
```

---

### Permission errors on install

**Solution:**
```bash
# Option 1: Use nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install node
npm install -g telepipe

# Option 2: Configure npm prefix
mkdir ~/.npm-global
npm config set prefix '~/.npm-global'
export PATH=~/.npm-global/bin:$PATH
npm install -g telepipe
```

See: [installation.md](installation.md)

---

## Diagnostic Commands

### List all sessions
```bash
telepipe info
```

### Check specific session
```bash
telepipe info --id <session-id>
```

### Check what's using a port
```bash
lsof -i :<port>
```

### Check system limits
```bash
ulimit -a
```

### Check Telepipe location
```bash
which telepipe
npm root -g
```

### Check process tree
```bash
ps aux | grep telepipe
```

---

## Error Code Reference

| Code | Error | Quick Fix |
|------|-------|-----------|
| 21 | E-ALLOC-FD | Close sessions, increase ulimit |
| 22 | E-ALLOC-PORT | Close sessions, wait for TIME_WAIT |
| 23 | E-ALLOC-DICT | Check ~/.telepipe permissions |
| 31 | E-FD-BIND | Restart session |
| 32 | E-FD-REDIRECT | Check command exists |
| 41 | E-TCP-CONNECT | Start the service |
| 42 | E-TCP-BROKEN | Reconnect |
| 43 | E-TCP-RESET | Check service status |
| 61 | E-DICT-MISSING | Use correct session ID |
| 64 | E-DICT-CONFLICT | Use different ID or disconnect existing |
| 71 | E-CLI-ARGS | Check command syntax |
| 76 | E-CLI-INVALID-PORT | Use valid port (1-65535) |
| 81 | E-EXEC-ALREADY-ACTIVE | Wait or use separate sessions |

Full reference: [../reference/error-codes.md](../reference/error-codes.md)

---

## Getting Help

### GitHub Issues
https://github.com/teleplank/telepipe/issues

### Documentation
- [README.md](../../README.md)
- [Command Reference](../reference/commands.md)
- [Error Codes](../reference/error-codes.md)

### Debug Mode
```bash
telepipe redirect --id myapp --debug -- node server.js
```

Debug output shows:
- Timestamps
- PIDs
- Allocated resources
- Detailed error context

---

## Still Stuck?

1. **Check the error code** - Each code has specific meaning
2. **Check session state** - `telepipe info --id <session-id>`
3. **Check service status** - Is the target service running?
4. **Check permissions** - Can you access the port/file?
5. **Try debug mode** - `--debug` flag shows details
6. **File an issue** - Include error code, platform, and steps to reproduce
