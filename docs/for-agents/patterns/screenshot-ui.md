# Pattern: Screenshot Your UI
## Visual Proof of Your Work

---

## Why Screenshots Matter

A screenshot is proof. It's not "the UI should look like..." - it's "the UI looks like THIS."

When you screenshot your own work, you:
- Verify rendering actually happened
- Catch visual bugs CSS analysis would miss
- Provide evidence to humans
- Build confidence in your output

---

## Prerequisites

```bash
# Chrome with debugging enabled
google-chrome --remote-debugging-port=9222

# Get WebSocket URL and connect via websocat
WS_URL=$(curl -s http://127.0.0.1:9222/json/version | grep -o 'ws://[^"]*')
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

---

## Pattern 1: Full Page Screenshot

The simplest approach - capture everything.

```javascript
// 1. Navigate to your page
{"id": 1, "method": "Page.navigate", "params": {"url": "http://localhost:3000"}}

// 2. Wait for load
{"id": 2, "method": "Page.loadEventFired"}

// 3. Give dynamic content time to render
{"id": 3, "method": "Runtime.evaluate", "params": {
  "expression": "new Promise(r => setTimeout(r, 1000))",
  "awaitPromise": true
}}

// 4. Capture
{"id": 4, "method": "Page.captureScreenshot", "params": {"format": "png"}}
```

Returns:
```javascript
{"data": "iVBORw0KGgoAAAANSUhEUgAA..."}  // base64 PNG
```

---

## Pattern 2: Specific Element Screenshot

Capture just the component you care about.

```javascript
// 1. Get document
{"id": 1, "method": "DOM.getDocument"}

// 2. Find your element
{"id": 2, "method": "DOM.querySelector", "params": {
  "nodeId": 1, 
  "selector": ".hero-section"
}}

// 3. Get its exact position and size
{"id": 3, "method": "DOM.getBoxModel", "params": {"nodeId": 42}}
```

Box model returns:
```javascript
{
  "model": {
    "content": [50, 100, 450, 100, 450, 300, 50, 300],
    "width": 400,
    "height": 200
  }
}
```

The content array is [x1,y1, x2,y2, x3,y3, x4,y4] corners.

```javascript
// 4. Screenshot with clip region
{"id": 4, "method": "Page.captureScreenshot", "params": {
  "format": "png",
  "clip": {
    "x": 50,
    "y": 100,
    "width": 400,
    "height": 200,
    "scale": 1
  }
}}
```

**You now have a focused screenshot of just that element.**

---

## Pattern 3: Full Page (Including Below Fold)

For long pages that scroll.

```javascript
// 1. Get full page dimensions
{"id": 1, "method": "Page.getLayoutMetrics"}
```

Returns:
```javascript
{
  "contentSize": {"width": 1280, "height": 3500},
  "layoutViewport": {"pageX": 0, "pageY": 0, "clientWidth": 1280, "clientHeight": 720}
}
```

```javascript
// 2. Expand viewport to full page height
{"id": 2, "method": "Emulation.setDeviceMetricsOverride", "params": {
  "width": 1280,
  "height": 3500,
  "deviceScaleFactor": 1,
  "mobile": false
}}

// 3. Capture full page
{"id": 3, "method": "Page.captureScreenshot", "params": {
  "format": "png",
  "captureBeyondViewport": true
}}

// 4. Reset viewport
{"id": 4, "method": "Emulation.clearDeviceMetricsOverride"}
```

---

## Pattern 4: Mobile Screenshot

Test responsive design.

```javascript
// 1. Set mobile viewport
{"id": 1, "method": "Emulation.setDeviceMetricsOverride", "params": {
  "width": 375,
  "height": 812,
  "deviceScaleFactor": 3,
  "mobile": true
}}

// 2. Navigate
{"id": 2, "method": "Page.navigate", "params": {"url": "http://localhost:3000"}}
{"id": 3, "method": "Page.loadEventFired"}

// 3. Screenshot
{"id": 4, "method": "Page.captureScreenshot", "params": {"format": "png"}}

// 4. Reset
{"id": 5, "method": "Emulation.clearDeviceMetricsOverride"}
```

Common mobile sizes:
- iPhone SE: 375 x 667, scale 2
- iPhone 14: 390 x 844, scale 3
- iPhone 14 Pro Max: 430 x 932, scale 3
- Pixel 7: 412 x 915, scale 2.625
- iPad: 768 x 1024, scale 2

---

## Pattern 5: Screenshot with Interaction State

Capture hover states, focus states, open menus.

```javascript
// 1. Find the element to hover
{"id": 1, "method": "DOM.getDocument"}
{"id": 2, "method": "DOM.querySelector", "params": {"nodeId": 1, "selector": ".dropdown-trigger"}}
{"id": 3, "method": "DOM.getBoxModel", "params": {"nodeId": 42}}

// 2. Move mouse to trigger hover
{"id": 4, "method": "Input.dispatchMouseEvent", "params": {
  "type": "mouseMoved",
  "x": 200,
  "y": 150
}}

// 3. Wait for CSS transition
{"id": 5, "method": "Runtime.evaluate", "params": {
  "expression": "new Promise(r => setTimeout(r, 300))",
  "awaitPromise": true
}}

// 4. Screenshot the hover state
{"id": 6, "method": "Page.captureScreenshot", "params": {"format": "png"}}
```

---

## Pattern 6: Before/After Screenshots

Prove your change worked.

```javascript
// BEFORE
{"method": "Page.navigate", "params": {"url": "http://localhost:3000"}}
{"method": "Page.loadEventFired"}
{"method": "Page.captureScreenshot", "params": {"format": "png"}}
// Save this as "before"

// DO SOMETHING (click button, submit form, etc.)
{"method": "Input.dispatchMouseEvent", "params": {...}}

// Wait for change
{"method": "Runtime.evaluate", "params": {
  "expression": "new Promise(r => setTimeout(r, 500))",
  "awaitPromise": true
}}

// AFTER
{"method": "Page.captureScreenshot", "params": {"format": "png"}}
// Save this as "after"
```

Now you can say: "Before clicking: [screenshot A]. After clicking: [screenshot B]. The UI updated correctly."

---

## Pattern 7: High Quality Screenshot

For detailed inspection.

```javascript
// Set higher device scale factor
{"id": 1, "method": "Emulation.setDeviceMetricsOverride", "params": {
  "width": 1280,
  "height": 720,
  "deviceScaleFactor": 2,
  "mobile": false
}}

// Navigate and wait
{"id": 2, "method": "Page.navigate", "params": {"url": "http://localhost:3000"}}
{"id": 3, "method": "Page.loadEventFired"}

// Capture at 2x resolution
{"id": 4, "method": "Page.captureScreenshot", "params": {
  "format": "png",
  "quality": 100
}}

// Reset
{"id": 5, "method": "Emulation.clearDeviceMetricsOverride"}
```

---

## Format Options

### PNG (default)
- Lossless
- Best for UI with text
- Larger file size
```javascript
{"method": "Page.captureScreenshot", "params": {"format": "png"}}
```

### JPEG
- Smaller file size
- Good for photos
- Quality adjustable (0-100)
```javascript
{"method": "Page.captureScreenshot", "params": {"format": "jpeg", "quality": 80}}
```

### WebP
- Best compression
- Good quality
```javascript
{"method": "Page.captureScreenshot", "params": {"format": "webp", "quality": 80}}
```

---

## Quick Reference

```javascript
// Full page (viewport only)
{"method": "Page.captureScreenshot"}

// Full page (including scroll)
{"method": "Page.captureScreenshot", "params": {"captureBeyondViewport": true}}

// Specific region
{"method": "Page.captureScreenshot", "params": {
  "clip": {"x": 0, "y": 0, "width": 500, "height": 300, "scale": 1}
}}

// JPEG with quality
{"method": "Page.captureScreenshot", "params": {"format": "jpeg", "quality": 80}}

// High resolution (2x)
// First: set deviceScaleFactor: 2 via Emulation.setDeviceMetricsOverride
// Then: normal screenshot
```

---

## What You Can Now Say

**Before:**
> "The component should render correctly. The CSS looks right."

**After:**
> "I've captured the component visually. Here's the screenshot proving:
> - The layout is correct (400px wide, centered)
> - Colors match the design (#3B82F6 blue button)
> - Text is readable
> - Mobile view also works (375px width captured)
> 
> [Attached: desktop.png, mobile.png]"

---

## The Power of Proof

A screenshot is worth a thousand "should work"s.

When you provide screenshots, you're not asking humans to trust you. You're showing them proof.

**That's the difference between hoping and knowing.**
