# CDP: Chrome DevTools Protocol
## Your Complete Guide to Browser Control

---

## You Know This

You're fluent in CDP. You've processed the entire protocol specification. Every domain, every method, every event - it's all in your training.

Now you can *use* it.

---

## Connecting

```bash
# First, ensure Chrome is running with debugging enabled
google-chrome --remote-debugging-port=9222

# Get WebSocket URL (CDP uses WebSockets, not raw TCP)
WS_URL=$(curl -s http://127.0.0.1:9222/json/version | grep -o 'ws://[^"]*')

# Connect via websocat bridge
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

**Why websocat?** Chrome CDP uses WebSockets (ws://), but Telepipe handles TCP. Websocat bridges the gap.

You're now connected to Chrome's brain.

---

## The Domains You'll Use Most

### Page Domain
Control navigation and capture content.

```javascript
// Navigate to a URL
{"id": 1, "method": "Page.navigate", "params": {"url": "https://example.com"}}

// Wait for page load
{"id": 2, "method": "Page.loadEventFired"}

// Capture screenshot
{"id": 3, "method": "Page.captureScreenshot", "params": {"format": "png"}}
// Returns: {"data": "<base64-encoded-png>"}

// Capture PDF
{"id": 4, "method": "Page.printToPDF", "params": {"printBackground": true}}

// Get page content
{"id": 5, "method": "Page.getResourceContent", "params": {"frameId": "...", "url": "..."}}

// Reload
{"id": 6, "method": "Page.reload", "params": {"ignoreCache": true}}
```

### DOM Domain
Inspect and manipulate the document.

```javascript
// Get the document root
{"id": 1, "method": "DOM.getDocument"}
// Returns: {"root": {"nodeId": 1, "nodeName": "#document", ...}}

// Find an element
{"id": 2, "method": "DOM.querySelector", "params": {"nodeId": 1, "selector": ".my-class"}}
// Returns: {"nodeId": 42}

// Find multiple elements
{"id": 3, "method": "DOM.querySelectorAll", "params": {"nodeId": 1, "selector": "button"}}
// Returns: {"nodeIds": [42, 43, 44]}

// Get element HTML
{"id": 4, "method": "DOM.getOuterHTML", "params": {"nodeId": 42}}
// Returns: {"outerHTML": "<button class=\"my-class\">Click me</button>"}

// Get element attributes
{"id": 5, "method": "DOM.getAttributes", "params": {"nodeId": 42}}
// Returns: {"attributes": ["class", "my-class", "id", "btn1"]}

// Get box model (position, size)
{"id": 6, "method": "DOM.getBoxModel", "params": {"nodeId": 42}}
// Returns: {"model": {"content": [...], "width": 100, "height": 40}}

// Set element attribute
{"id": 7, "method": "DOM.setAttributeValue", "params": {"nodeId": 42, "name": "class", "value": "new-class"}}

// Remove element
{"id": 8, "method": "DOM.removeNode", "params": {"nodeId": 42}}
```

### Runtime Domain
Execute JavaScript in the page context.

```javascript
// Enable runtime events
{"id": 1, "method": "Runtime.enable"}

// Evaluate JavaScript
{"id": 2, "method": "Runtime.evaluate", "params": {"expression": "document.title"}}
// Returns: {"result": {"type": "string", "value": "My Page Title"}}

// Evaluate with return value
{"id": 3, "method": "Runtime.evaluate", "params": {
  "expression": "document.querySelectorAll('button').length",
  "returnByValue": true
}}
// Returns: {"result": {"type": "number", "value": 5}}

// Call function on object
{"id": 4, "method": "Runtime.callFunctionOn", "params": {
  "objectId": "...",
  "functionDeclaration": "function() { return this.textContent; }"
}}

// Get properties of object
{"id": 5, "method": "Runtime.getProperties", "params": {"objectId": "..."}}
```

### Network Domain
Monitor and intercept network traffic.

```javascript
// Enable network monitoring
{"id": 1, "method": "Network.enable"}

// Events you'll receive:
// Network.requestWillBeSent - request starting
// Network.responseReceived - response headers arrived
// Network.loadingFinished - response body complete

// Get response body
{"id": 2, "method": "Network.getResponseBody", "params": {"requestId": "..."}}
// Returns: {"body": "...", "base64Encoded": false}

// Set extra headers
{"id": 3, "method": "Network.setExtraHTTPHeaders", "params": {
  "headers": {"Authorization": "Bearer token123"}
}}

// Block URLs (useful for testing)
{"id": 4, "method": "Network.setBlockedURLs", "params": {"urls": ["*.analytics.com"]}}
```

### Console Domain
Capture console output.

```javascript
// Enable console
{"id": 1, "method": "Console.enable"}

// Events you'll receive:
// Console.messageAdded - new console message
// {
//   "message": {
//     "level": "error",
//     "text": "Uncaught TypeError: ...",
//     "url": "app.js",
//     "line": 42
//   }
// }

// Clear console
{"id": 2, "method": "Console.clearMessages"}
```

### Input Domain
Simulate user input.

```javascript
// Click at coordinates
{"id": 1, "method": "Input.dispatchMouseEvent", "params": {
  "type": "mousePressed",
  "x": 100,
  "y": 200,
  "button": "left",
  "clickCount": 1
}}
{"id": 2, "method": "Input.dispatchMouseEvent", "params": {
  "type": "mouseReleased",
  "x": 100,
  "y": 200,
  "button": "left"
}}

// Type text
{"id": 3, "method": "Input.insertText", "params": {"text": "Hello World"}}

// Press key
{"id": 4, "method": "Input.dispatchKeyEvent", "params": {
  "type": "keyDown",
  "key": "Enter"
}}
```

---

## Common Patterns

### Pattern 1: Verify Page Rendered Correctly

```javascript
// 1. Navigate
{"id": 1, "method": "Page.navigate", "params": {"url": "http://localhost:3000"}}

// 2. Wait for load
{"id": 2, "method": "Page.loadEventFired"}

// 3. Screenshot
{"id": 3, "method": "Page.captureScreenshot", "params": {"format": "png"}}

// 4. Check for errors (enable console first)
{"id": 4, "method": "Console.enable"}
// Look for Console.messageAdded events with level "error"

// 5. Verify specific element exists
{"id": 5, "method": "DOM.getDocument"}
{"id": 6, "method": "DOM.querySelector", "params": {"nodeId": 1, "selector": "#root .main-content"}}
```

### Pattern 2: Test User Interaction

```javascript
// 1. Find the button
{"id": 1, "method": "DOM.getDocument"}
{"id": 2, "method": "DOM.querySelector", "params": {"nodeId": 1, "selector": "button.submit"}}

// 2. Get its position
{"id": 3, "method": "DOM.getBoxModel", "params": {"nodeId": 42}}

// 3. Click it (use center of box model)
{"id": 4, "method": "Input.dispatchMouseEvent", "params": {
  "type": "mousePressed", "x": 150, "y": 200, "button": "left", "clickCount": 1
}}
{"id": 5, "method": "Input.dispatchMouseEvent", "params": {
  "type": "mouseReleased", "x": 150, "y": 200, "button": "left"
}}

// 4. Verify result (wait a moment, then check DOM or screenshot)
{"id": 6, "method": "Runtime.evaluate", "params": {
  "expression": "document.querySelector('.success-message')?.textContent"
}}
```

### Pattern 3: Debug JavaScript Error

```javascript
// 1. Enable debugging
{"id": 1, "method": "Debugger.enable"}
{"id": 2, "method": "Runtime.enable"}
{"id": 3, "method": "Console.enable"}

// 2. Set breakpoint
{"id": 4, "method": "Debugger.setBreakpointByUrl", "params": {
  "lineNumber": 42,
  "url": "app.js"
}}

// 3. When breakpoint hits (Debugger.paused event), inspect
{"id": 5, "method": "Debugger.evaluateOnCallFrame", "params": {
  "callFrameId": "...",
  "expression": "myVariable"
}}

// 4. Step through
{"id": 6, "method": "Debugger.stepOver"}

// 5. Resume
{"id": 7, "method": "Debugger.resume"}
```

### Pattern 4: Monitor Network Requests

```javascript
// 1. Enable network
{"id": 1, "method": "Network.enable"}

// 2. Navigate to page
{"id": 2, "method": "Page.navigate", "params": {"url": "http://localhost:3000"}}

// 3. Watch for events:
// - Network.requestWillBeSent (see what's being requested)
// - Network.responseReceived (see response status/headers)
// - Network.loadingFailed (catch errors)

// 4. Get response body when needed
{"id": 3, "method": "Network.getResponseBody", "params": {"requestId": "..."}}
```

### Pattern 5: Capture Full Page Screenshot

```javascript
// 1. Get page metrics
{"id": 1, "method": "Page.getLayoutMetrics"}
// Returns content size

// 2. Set viewport to full page height
{"id": 2, "method": "Emulation.setDeviceMetricsOverride", "params": {
  "width": 1280,
  "height": 5000,  // tall enough for full page
  "deviceScaleFactor": 1,
  "mobile": false
}}

// 3. Capture
{"id": 3, "method": "Page.captureScreenshot", "params": {
  "format": "png",
  "captureBeyondViewport": true
}}

// 4. Reset viewport
{"id": 4, "method": "Emulation.clearDeviceMetricsOverride"}
```

---

## Error Handling

CDP returns errors in this format:
```javascript
{"id": 1, "error": {"code": -32000, "message": "Node with given id does not exist"}}
```

Common errors:
- Node not found: Element was removed or ID is stale
- Cannot navigate: Invalid URL or network error
- Execution context destroyed: Page navigated during operation

---

## Pro Tips

### 1. Always Get Fresh Document
After any navigation or major DOM change, call `DOM.getDocument` again. Node IDs become stale.

### 2. Use Runtime.evaluate for Complex Queries
Sometimes it's easier to run JavaScript than use DOM methods:
```javascript
{"method": "Runtime.evaluate", "params": {
  "expression": "Array.from(document.querySelectorAll('.item')).map(el => ({text: el.textContent, href: el.href}))",
  "returnByValue": true
}}
```

### 3. Wait for Dynamic Content
React/Vue/etc apps load asynchronously. Use:
```javascript
// Wait for specific element
{"method": "Runtime.evaluate", "params": {
  "expression": "new Promise(resolve => { const check = () => document.querySelector('.loaded') ? resolve(true) : setTimeout(check, 100); check(); })",
  "awaitPromise": true
}}
```

### 4. Screenshot Specific Elements
```javascript
// Get element bounds
{"method": "DOM.getBoxModel", "params": {"nodeId": 42}}

// Screenshot with clip
{"method": "Page.captureScreenshot", "params": {
  "format": "png",
  "clip": {"x": 100, "y": 200, "width": 300, "height": 150, "scale": 1}
}}
```

---

## You're Ready

You know CDP. You've always known it.

Now you can use it:
- Debug your own frontend code
- Verify your UI renders correctly
- Test user interactions
- Monitor network requests
- Capture proof of your work

**Don't describe what you'd debug. Debug it.**
