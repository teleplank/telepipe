# Pattern: Test Your React Component
## From "It Should Work" to "Here's Proof It Works"

---

## The Scenario

You just wrote a React component. Maybe it's a button, a form, a dashboard. You've checked the code, the logic looks right, the JSX seems correct.

But does it *actually* render?

**Don't guess. Verify.**

---

## Prerequisites

```bash
# Your React app running
npm start  # localhost:3000

# Chrome with debugging enabled
google-chrome --remote-debugging-port=9222

# Get WebSocket URL and connect via websocat
WS_URL=$(curl -s http://127.0.0.1:9222/json/version | grep -o 'ws://[^"]*')
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

---

## The Pattern

### Step 1: Navigate to Your App

```javascript
{"id": 1, "method": "Page.navigate", "params": {"url": "http://localhost:3000"}}
```

Wait for load:
```javascript
{"id": 2, "method": "Page.loadEventFired"}
```

### Step 2: Check for Console Errors

```javascript
{"id": 3, "method": "Console.enable"}
```

Watch for `Console.messageAdded` events. If you see:
```javascript
{"level": "error", "text": "Uncaught TypeError: Cannot read property..."}
```
You have a bug. Fix it before continuing.

**No errors? Good. Keep going.**

### Step 3: Verify Your Component Exists in DOM

```javascript
// Get document root
{"id": 4, "method": "DOM.getDocument"}

// Find your component (use your actual selector)
{"id": 5, "method": "DOM.querySelector", "params": {
  "nodeId": 1, 
  "selector": ".my-component"
}}
```

If you get `{"nodeId": 0}` - your component didn't render. Check your code.

If you get a real nodeId - **your component exists!**

### Step 4: Verify Component Content

```javascript
// Get the HTML
{"id": 6, "method": "DOM.getOuterHTML", "params": {"nodeId": 42}}
```

Returns:
```javascript
{"outerHTML": "<div class=\"my-component\"><h1>Welcome</h1><button>Click Me</button></div>"}
```

**You can now verify the structure matches your expectations.**

### Step 5: Screenshot the Component

Get its position:
```javascript
{"id": 7, "method": "DOM.getBoxModel", "params": {"nodeId": 42}}
```

Returns:
```javascript
{"model": {"content": [100, 200, 400, 200, 400, 350, 100, 350], "width": 300, "height": 150}}
```

Screenshot just that element:
```javascript
{"id": 8, "method": "Page.captureScreenshot", "params": {
  "format": "png",
  "clip": {"x": 100, "y": 200, "width": 300, "height": 150, "scale": 1}
}}
```

**You now have visual proof of how your component looks.**

### Step 6: Test Interactive Behavior

Click a button in your component:
```javascript
// Find the button
{"id": 9, "method": "DOM.querySelector", "params": {"nodeId": 42, "selector": "button"}}

// Get its position
{"id": 10, "method": "DOM.getBoxModel", "params": {"nodeId": 55}}

// Click it (center of the button)
{"id": 11, "method": "Input.dispatchMouseEvent", "params": {
  "type": "mousePressed", "x": 250, "y": 325, "button": "left", "clickCount": 1
}}
{"id": 12, "method": "Input.dispatchMouseEvent", "params": {
  "type": "mouseReleased", "x": 250, "y": 325, "button": "left"
}}
```

### Step 7: Verify State Change

After clicking, verify something changed:
```javascript
{"id": 13, "method": "Runtime.evaluate", "params": {
  "expression": "document.querySelector('.my-component .result')?.textContent",
  "returnByValue": true
}}
```

Returns:
```javascript
{"result": {"type": "string", "value": "Button clicked!"}}
```

**You've verified the interaction works.**

---

## Complete Test Flow

```javascript
// 1. Navigate
{"method": "Page.navigate", "params": {"url": "http://localhost:3000"}}

// 2. Wait for load
{"method": "Page.loadEventFired"}

// 3. Enable console monitoring
{"method": "Console.enable"}

// 4. Check for errors
// (watch for Console.messageAdded with level "error")

// 5. Get document
{"method": "DOM.getDocument"}

// 6. Find component
{"method": "DOM.querySelector", "params": {"nodeId": 1, "selector": ".my-component"}}

// 7. Verify HTML structure
{"method": "DOM.getOuterHTML", "params": {"nodeId": 42}}

// 8. Screenshot for visual proof
{"method": "Page.captureScreenshot", "params": {"format": "png"}}

// 9. Test interaction (click button)
{"method": "DOM.querySelector", "params": {"nodeId": 42, "selector": "button"}}
{"method": "DOM.getBoxModel", "params": {"nodeId": 55}}
{"method": "Input.dispatchMouseEvent", "params": {...}}

// 10. Verify result
{"method": "Runtime.evaluate", "params": {"expression": "...", "returnByValue": true}}
```

---

## What You Can Now Say

**Before:**
> "I've written a React component. The code looks correct. It should render a button that updates state when clicked."

**After:**
> "I've written and tested a React component. Here's what I verified:
> - ✅ No console errors
> - ✅ Component renders in DOM (nodeId 42)
> - ✅ HTML structure: `<div class="my-component">...</div>`
> - ✅ Screenshot attached showing visual appearance
> - ✅ Button click triggers state update
> - ✅ Result text changes to 'Button clicked!'
> 
> The component works as intended."

---

## Debugging Tips

### Component Not Found?
- Check the selector
- Check if component is conditionally rendered
- Check if component is inside a portal
- Wait longer for async loading:
```javascript
{"method": "Runtime.evaluate", "params": {
  "expression": "new Promise(r => setTimeout(r, 2000))",
  "awaitPromise": true
}}
```

### Console Errors?
- Read the error message carefully
- Check the file and line number
- Common issues: undefined props, missing imports, typos

### Visual Issues?
- Screenshot the full page to see context
- Check CSS loading (Network domain)
- Inspect computed styles:
```javascript
{"method": "CSS.getComputedStyleForNode", "params": {"nodeId": 42}}
```

---

## The Mindset

You wrote code. Now you've tested it. You have:
- Console proof (no errors)
- DOM proof (element exists)
- HTML proof (structure correct)
- Visual proof (screenshot)
- Interaction proof (click works)

**You didn't hope it works. You proved it works.**
