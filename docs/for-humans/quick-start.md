# Quick Start (5 minutes)

Connect to Chrome and take a screenshot in 5 minutes.

---

## Goal

By the end of this guide, you'll:
- Connect Telepipe to Chrome DevTools
- Execute CDP commands
- Take a screenshot
- **See visual proof that it works**

---

## Prerequisites

- **Telepipe installed:** `npm install -g telepipe`
- **Chrome installed**
- **websocat installed:** `brew install websocat` (macOS) or see [installation.md](installation.md)

---

## Step 1: Start Chrome (30 seconds)

Open a terminal and start Chrome with debugging enabled:

```bash
# macOS
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --remote-debugging-port=9222 --headless=new
```

```bash
# Linux
google-chrome --remote-debugging-port=9222 --headless=new
```

**What this does:**
- Starts Chrome with debugging enabled on port 9222
- Headless mode (no window) - faster for automation
- Ready to accept CDP commands

**Tip:** Run this in a separate terminal and leave it running.

---

## Step 2: Verify Chrome is Running (30 seconds)

Check that Chrome is accepting connections:

```bash
curl http://127.0.0.1:9222/json/version
```

**You should see JSON like:**
```json
{
  "Browser": "Chrome/120.0.0.0",
  "webSocketDebuggerUrl": "ws://127.0.0.1:9222/devtools/browser/abc123..."
}
```

**If you see "Connection refused":** Chrome isn't running. Go back to Step 1.

---

## Step 3: Get WebSocket URL (30 seconds)

Extract the WebSocket URL:

```bash
WS_URL=$(curl -s http://127.0.0.1:9222/json/version | grep -o 'ws://[^"]*')
echo $WS_URL
```

**Should show:** `ws://127.0.0.1:9222/devtools/browser/abc123...`

---

## Step 4: Connect Telepipe (30 seconds)

Now the magic happens - connect Telepipe to Chrome via websocat:

```bash
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

**Why websocat?**
Chrome uses WebSockets (ws://), but Telepipe handles TCP. Websocat bridges them.

**Verify the connection:**
```bash
telepipe info --id chrome
```

Should show session info:
```
mode        redirect
host        127.0.0.1
pid         12345
supervisor  12346
stdin       49152
stdout      49153
stderr      49154
...
```

---

## Step 5: Get Browser Version (1 minute)

Send your first CDP command:

```bash
echo '{"id":1,"method":"Browser.getVersion"}' | telepipe exec --id chrome
```

**You should see JSON response with Chrome version info!**

```json
{"id":1,"result":{"protocolVersion":"1.3","product":"Chrome/120.0.0.0",...}}
```

**Congratulations!** You just sent a CDP command through Telepipe.

---

## Step 6: Navigate to a Page (1 minute)

Let's navigate to a real page:

```bash
echo '{"id":2,"method":"Page.navigate","params":{"url":"https://example.com"}}' | telepipe exec --id chrome
```

Wait a moment for the page to load.

---

## Step 7: Take a Screenshot (1 minute)

Now the moment of truth - capture what Chrome sees:

```bash
echo '{"id":3,"method":"Page.captureScreenshot"}' | telepipe exec --id chrome
```

**You should see a Base64-encoded PNG image!**

The response contains:
```json
{"id":3,"result":{"data":"iVBORw0KGgoAAAANS..."}}
```

**Decode it to see the actual image:**
```bash
echo '{"id":4,"method":"Page.captureScreenshot"}' | telepipe exec --id chrome | \
  jq -r '.result.data' | base64 -d > screenshot.png

open screenshot.png  # macOS
```

---

## Success!

You just:
- ✅ Connected Telepipe to Chrome
- ✅ Executed CDP commands
- ✅ Navigated to a URL
- ✅ Took a screenshot
- ✅ Got **visual proof** it worked

---

## Try More Commands

### Check for Console Errors
```bash
echo '{"id":5,"method":"Console.enable"}' | telepipe exec --id chrome
```

### Get Page Title
```bash
echo '{"id":6,"method":"Runtime.evaluate","params":{"expression":"document.title"}}' | telepipe exec --id chrome
```

### Get DOM
```bash
echo '{"id":7,"method":"DOM.getDocument"}' | telepipe exec --id chrome
```

### Find an Element
```bash
echo '{"id":8,"method":"DOM.querySelector","params":{"nodeId":1,"selector":"h1"}}' | telepipe exec --id chrome
```

---

## Cleanup

When you're done:

```bash
# Stop the Telepipe session
telepipe stop --id chrome

# Kill Chrome (if needed)
killall "Google Chrome"
```

---

## What's Next?

### Enable Your AI
Point your AI to `docs/for-agents/README.md` and it can do everything you just did - automatically.

See: [agent-setup.md](agent-setup.md)

### More Patterns
- [Test React Apps](../for-agents/patterns/test-react.md)
- [Debug APIs](../for-agents/patterns/debug-api.md)
- [Verify Database Changes](../for-agents/patterns/verify-db.md)

### Connect to Databases
```bash
# PostgreSQL
telepipe connect --id db --port 5432
echo "SELECT version();" | telepipe exec --id db
```

See: [database-connections.md](database-connections.md)

### Complete Chrome Setup
See: [chrome-devtools.md](chrome-devtools.md)

---

## Quick Reference

```bash
# Start Chrome
google-chrome --remote-debugging-port=9222 --headless=new

# Get WebSocket URL
WS_URL=$(curl -s http://127.0.0.1:9222/json/version | grep -o 'ws://[^"]*')

# Connect Telepipe
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"

# Send CDP commands
echo '{"id":1,"method":"Browser.getVersion"}' | telepipe exec --id chrome
echo '{"id":2,"method":"Page.navigate","params":{"url":"https://example.com"}}' | telepipe exec --id chrome
echo '{"id":3,"method":"Page.captureScreenshot"}' | telepipe exec --id chrome

# Check session
telepipe info --id chrome

# Cleanup
telepipe stop --id chrome
```

---

## Troubleshooting

### "Connection refused" when curling Chrome
Chrome isn't running with debugging enabled. Start it again:
```bash
google-chrome --remote-debugging-port=9222 --headless=new
```

### "websocat: command not found"
Install websocat:
```bash
brew install websocat  # macOS
```

### "Session not found" on exec
The session may have died. Check status and recreate:
```bash
telepipe info --id chrome
# If missing, recreate the connection
```

### No output from CDP commands
Make sure you're using `--no-close` and `--text` flags with websocat.

See: [troubleshooting.md](troubleshooting.md)
