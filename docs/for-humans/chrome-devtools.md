# Chrome DevTools Setup

Complete guide to connecting Telepipe to Chrome DevTools Protocol (CDP).

---

## Understanding the Connection

**Chrome uses WebSockets. Telepipe uses TCP.**

```
Chrome (WebSocket) ←→ websocat (bridge) ←→ Telepipe (TCP) ←→ Your AI
```

This is why we need **websocat** - it bridges WebSocket to TCP.

---

## Prerequisites

### Install websocat

**macOS:**
```bash
brew install websocat
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt install websocat
```

**Via Cargo:**
```bash
cargo install websocat
```

**Binary download:**
https://github.com/vi/websocat/releases

### Verify websocat
```bash
websocat --version
```

---

## Starting Chrome with Debugging

### macOS

**With GUI (visible window):**
```bash
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --remote-debugging-port=9222
```

**Headless (no window):**
```bash
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --remote-debugging-port=9222 --headless=new
```

**Full flags for automation:**
```bash
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
  --remote-debugging-port=9222 \
  --headless=new \
  --disable-gpu \
  --no-sandbox \
  --disable-dev-shm-usage
```

### Linux

**Chromium:**
```bash
chromium --remote-debugging-port=9222
```

**Chrome:**
```bash
google-chrome-stable --remote-debugging-port=9222
```

**Headless:**
```bash
google-chrome --remote-debugging-port=9222 --headless=new
```

### Docker

```dockerfile
FROM chromedp/headless-shell:latest
# Chrome is already running with --remote-debugging-port=9222
```

```bash
docker run -d -p 9222:9222 chromedp/headless-shell
```

---

## Connecting Telepipe

### Step 1: Verify Chrome is Running

```bash
curl http://127.0.0.1:9222/json/version
```

**Should return JSON:**
```json
{
  "Browser": "Chrome/120.0.0.0",
  "webSocketDebuggerUrl": "ws://127.0.0.1:9222/devtools/browser/abc123..."
}
```

### Step 2: Get WebSocket URL

```bash
WS_URL=$(curl -s http://127.0.0.1:9222/json/version | grep -o 'ws://[^"]*')
echo $WS_URL
```

**Alternative (with jq):**
```bash
WS_URL=$(curl -s http://127.0.0.1:9222/json/version | jq -r '.webSocketDebuggerUrl')
```

### Step 3: Connect via websocat

```bash
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

**Important flags:**
- `--no-close` - Keep connection alive after stdin closes
- `--text` - Send text frames (CDP uses JSON)

### Step 4: Verify Connection

```bash
telepipe info --id chrome
```

**Should show session info with mode `redirect`.**

---

## Your First CDP Commands

### Get Browser Version
```bash
echo '{"id":1,"method":"Browser.getVersion"}' | telepipe exec --id chrome
```

### Navigate to URL
```bash
echo '{"id":2,"method":"Page.navigate","params":{"url":"https://example.com"}}' | telepipe exec --id chrome
```

### Take Screenshot
```bash
echo '{"id":3,"method":"Page.captureScreenshot"}' | telepipe exec --id chrome
```

### Save Screenshot to File
```bash
echo '{"id":4,"method":"Page.captureScreenshot"}' | telepipe exec --id chrome | \
  jq -r '.result.data' | base64 -d > screenshot.png
```

### Enable Console
```bash
echo '{"id":5,"method":"Console.enable"}' | telepipe exec --id chrome
```

### Enable Runtime
```bash
echo '{"id":6,"method":"Runtime.enable"}' | telepipe exec --id chrome
```

### Evaluate JavaScript
```bash
echo '{"id":7,"method":"Runtime.evaluate","params":{"expression":"document.title"}}' | telepipe exec --id chrome
```

### Get DOM Document
```bash
echo '{"id":8,"method":"DOM.getDocument"}' | telepipe exec --id chrome
```

### Query Selector
```bash
echo '{"id":9,"method":"DOM.querySelector","params":{"nodeId":1,"selector":"h1"}}' | telepipe exec --id chrome
```

### Enable Network Monitoring
```bash
echo '{"id":10,"method":"Network.enable"}' | telepipe exec --id chrome
```

---

## Common CDP Operations

### Full Page Navigation + Screenshot Flow
```bash
# Navigate
echo '{"id":1,"method":"Page.navigate","params":{"url":"https://example.com"}}' | telepipe exec --id chrome

# Wait for load (simple approach)
sleep 2

# Screenshot
echo '{"id":2,"method":"Page.captureScreenshot","params":{"format":"png","quality":80}}' | telepipe exec --id chrome
```

### Get All Cookies
```bash
echo '{"id":1,"method":"Network.getAllCookies"}' | telepipe exec --id chrome
```

### Set Cookie
```bash
echo '{"id":1,"method":"Network.setCookie","params":{"name":"test","value":"123","domain":"example.com"}}' | telepipe exec --id chrome
```

### Clear Cookies
```bash
echo '{"id":1,"method":"Network.clearBrowserCookies"}' | telepipe exec --id chrome
```

### Get Computed Styles
```bash
echo '{"id":1,"method":"CSS.getComputedStyleForNode","params":{"nodeId":1}}' | telepipe exec --id chrome
```

### Emulate Mobile
```bash
echo '{"id":1,"method":"Emulation.setDeviceMetricsOverride","params":{"width":375,"height":812,"deviceScaleFactor":3,"mobile":true}}' | telepipe exec --id chrome
```

---

## Working with Pages

### List All Pages/Targets
```bash
curl http://127.0.0.1:9222/json/list
```

### Connect to Specific Page
```bash
# Get page WebSocket URL
PAGE_WS=$(curl -s http://127.0.0.1:9222/json/list | jq -r '.[0].webSocketDebuggerUrl')

# Connect to that page
telepipe redirect --id page1 -- websocat --no-close --text "$PAGE_WS"
```

### Create New Page
```bash
curl http://127.0.0.1:9222/json/new?https://example.com
```

### Close Page
```bash
curl http://127.0.0.1:9222/json/close/<targetId>
```

---

## Multiple Chrome Instances

Run multiple Chrome instances on different ports:

```bash
# Instance 1
google-chrome --remote-debugging-port=9222 &

# Instance 2 (needs separate user data dir)
google-chrome --remote-debugging-port=9223 --user-data-dir=/tmp/chrome2 &
```

Connect to each:
```bash
# Get WebSocket URLs
WS1=$(curl -s http://127.0.0.1:9222/json/version | grep -o 'ws://[^"]*')
WS2=$(curl -s http://127.0.0.1:9223/json/version | grep -o 'ws://[^"]*')

# Connect separately
telepipe redirect --id chrome1 -- websocat --no-close --text "$WS1"
telepipe redirect --id chrome2 -- websocat --no-close --text "$WS2"
```

---

## Troubleshooting

### Port 9222 Already in Use

**Check what's using it:**
```bash
lsof -i :9222
```

**Kill it:**
```bash
kill <PID>
```

**Or use different port:**
```bash
google-chrome --remote-debugging-port=9223
```

### Connection Refused

**Verify Chrome is running:**
```bash
ps aux | grep chrome
```

**Check debugging port:**
```bash
curl http://127.0.0.1:9222/json/version
```

**If nothing, start Chrome with debugging enabled.**

### "websocat: command not found"

**Install websocat:**
```bash
brew install websocat  # macOS
```

### WebSocket URL Not Found

**Try alternative method:**
```bash
curl http://127.0.0.1:9222/json | jq '.[0].webSocketDebuggerUrl'
```

### CDP Commands Return Nothing

**Check websocat flags:**
- Must use `--no-close`
- Must use `--text`

**Verify session is alive:**
```bash
telepipe info --id chrome
```

### Session Dies After First Command

**This usually means websocat without `--no-close`.**

Recreate with correct flags:
```bash
telepipe stop --id chrome
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

### Chrome Crashes in Docker

**Add these flags:**
```bash
--no-sandbox --disable-dev-shm-usage --disable-gpu
```

---

## Performance Tips

### Use Headless Mode
Headless is faster for automation:
```bash
--headless=new
```

### Disable Unnecessary Features
```bash
--disable-gpu --disable-extensions --disable-background-networking
```

### Increase Shared Memory (Docker)
```bash
docker run --shm-size=2g ...
```

---

## CDP Resources

- **Protocol Documentation:** https://chromedevtools.github.io/devtools-protocol/
- **Domain Reference:** https://chromedevtools.github.io/devtools-protocol/tot/
- **Puppeteer (reference):** https://pptr.dev/

---

## Complete Example Script

```bash
#!/bin/bash
set -e

# Start Chrome headless
google-chrome --remote-debugging-port=9222 --headless=new &
CHROME_PID=$!
sleep 2

# Get WebSocket URL
WS_URL=$(curl -s http://127.0.0.1:9222/json/version | grep -o 'ws://[^"]*')

# Connect Telepipe
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"

# Navigate
echo '{"id":1,"method":"Page.navigate","params":{"url":"https://example.com"}}' | telepipe exec --id chrome
sleep 2

# Screenshot
echo '{"id":2,"method":"Page.captureScreenshot"}' | telepipe exec --id chrome | \
  jq -r '.result.data' | base64 -d > screenshot.png

echo "Screenshot saved to screenshot.png"

# Cleanup
telepipe stop --id chrome
kill $CHROME_PID 2>/dev/null
```

---

## Next Steps

- **Quick Start:** [quick-start.md](quick-start.md)
- **Enable Your AI:** [agent-setup.md](agent-setup.md)
- **CDP Patterns:** [../for-agents/protocols/cdp.md](../for-agents/protocols/cdp.md)
- **Troubleshooting:** [troubleshooting.md](troubleshooting.md)
