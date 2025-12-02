# Sighted Agent Build: Tic-Tac-Toe
## The First App Built by an AI That Can Verify Its Own Runtime State

---

## Preamble: What Makes This Different

You are about to build an application in a way no AI has done before.

Traditionally, AI writes code and hopes it works. The feedback loop is open:
- Write code → Run tests → Tests pass → Ship → Hope

Today, you close the loop. You will:
- Write code → Connect to browser → Query actual runtime state → VERIFY → Ship with proof

This is possible because you have access to **Telepipe**, which gives you CDP (Chrome DevTools Protocol) access to a live browser. You can reach INTO the JavaScript runtime and query actual values - closure state, event listeners, computed styles, console errors.

**This verification is impossible without CDP.**

---

## Step 1: Internalize Your Capabilities

Before writing any code, read and deeply internalize the following file:

```
/path/to/telepipe/AGENT_GUIDE.md
```

This document teaches you:
- What Telepipe gives you (TCP connections to live systems)
- How to use CDP (Chrome DevTools Protocol)
- The mindset shift: from "should work" to "verified it works"
- Patterns for verification

**Key concepts to internalize:**

1. **You are fluent in CDP.** You know Runtime.evaluate, DOM.querySelector, Console.enable. You've seen thousands of examples. Now you can EXECUTE them.

2. **The feedback loop closes.** You don't describe what to debug - you DEBUG. You don't hope state is correct - you QUERY it.

3. **Proof, not hope.** Every claim you make about your code, you can verify with actual CDP queries that return actual JSON.

Take a moment. This changes everything about how you build.

---

## Step 2: The Application - Tic-Tac-Toe

You will build a Tic-Tac-Toe game in React.

### Requirements

1. **3x3 grid** of clickable cells
2. **Alternating turns** - X goes first, then O
3. **Win detection** - detect when a player wins (3 in a row)
4. **Winner display** - show who won
5. **Game reset** - button to start a new game

### Technical Specifications

```javascript
// Core state shape
{
  board: Array(9).fill(null),  // null, 'X', or 'O' for each cell
  xIsNext: true,                // whose turn
  winner: null                  // null, 'X', or 'O'
}

// Win conditions (indices)
const WINNING_LINES = [
  [0, 1, 2], [3, 4, 5], [6, 7, 8],  // rows
  [0, 3, 6], [1, 4, 7], [2, 5, 8],  // columns
  [0, 4, 8], [2, 4, 6]              // diagonals
];
```

---

## Step 3: The Verification Challenge

Here's what makes this build revolutionary:

**You will VERIFY the game state via CDP after each move.**

The DOM shows X's and O's visually. But only CDP can tell you the actual JavaScript state values inside React's closure.

### Why This Matters

Without CDP:
```
Agent: "I implemented useState for the board. It should track moves correctly."
       "The win detection logic looks right. It should detect wins."
       "I hope this works."
```

With CDP:
```
Agent: "I queried the runtime. board[0] === 'X'. Verified."
       "I played a winning sequence. winner === 'X'. Confirmed."
       "I have proof this works."
```

**The difference: SHOULD vs VERIFIED. HOPE vs PROOF.**

---

## Step 4: Telepipe Connection

Before verification, establish CDP connection:

```bash
# Terminal 1: Start React app
cd tic-tac-toe && npm start
# App runs at http://localhost:3000

# Terminal 2: Start Chrome with debugging
google-chrome --remote-debugging-port=9222

# Terminal 3: Connect Telepipe (CDP requires websocat bridge)
WS_URL=$(curl -s http://127.0.0.1:9222/json/version | grep -o 'ws://[^"]*')
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

You now have a CDP channel to Chrome. You can send JSON commands and receive responses.

---

## Step 5: Build with Verification

### Phase 1: Create the App

Build the Tic-Tac-Toe game with these components:
- `App.js` - Main game logic and state
- `Board.js` - 3x3 grid component
- `Cell.js` - Individual clickable cell
- `App.css` - Styling

**Important:** Expose game state to window for CDP access:

```javascript
// In App.js, add this for CDP verification:
useEffect(() => {
  window.__GAME_STATE__ = { board, xIsNext, winner };
}, [board, xIsNext, winner]);
```

This allows CDP to query: `window.__GAME_STATE__`

### Phase 2: Verify Initial State

After the app loads, verify initial state via CDP:

```javascript
// CDP Command
{
  "id": 1,
  "method": "Runtime.evaluate",
  "params": {
    "expression": "window.__GAME_STATE__",
    "returnByValue": true
  }
}

// Expected Response
{
  "result": {
    "value": {
      "board": [null, null, null, null, null, null, null, null, null],
      "xIsNext": true,
      "winner": null
    }
  }
}
```

**Verification Criteria:**
- [ ] board is array of 9 nulls
- [ ] xIsNext is true
- [ ] winner is null

### Phase 3: Verify Move Mechanics

Click cell 0, then verify:

```javascript
// CDP: Click cell 0
{
  "method": "Runtime.evaluate",
  "params": {
    "expression": "document.querySelectorAll('.cell')[0].click()"
  }
}

// CDP: Query state after click
{
  "method": "Runtime.evaluate",
  "params": {
    "expression": "window.__GAME_STATE__",
    "returnByValue": true
  }
}

// Expected Response
{
  "result": {
    "value": {
      "board": ["X", null, null, null, null, null, null, null, null],
      "xIsNext": false,
      "winner": null
    }
  }
}
```

**Verification Criteria:**
- [ ] board[0] === 'X'
- [ ] xIsNext === false (turn switched)
- [ ] winner === null (no winner yet)

### Phase 4: Verify Turn Alternation

Click cell 4 (O's turn), then verify:

```javascript
// CDP: Click cell 4
{
  "method": "Runtime.evaluate",
  "params": {
    "expression": "document.querySelectorAll('.cell')[4].click()"
  }
}

// CDP: Query state
{
  "method": "Runtime.evaluate",
  "params": {
    "expression": "window.__GAME_STATE__",
    "returnByValue": true
  }
}

// Expected Response
{
  "result": {
    "value": {
      "board": ["X", null, null, null, "O", null, null, null, null],
      "xIsNext": true,
      "winner": null
    }
  }
}
```

**Verification Criteria:**
- [ ] board[4] === 'O'
- [ ] xIsNext === true (turn switched back)

### Phase 5: Verify Win Detection

Play a winning sequence for X: cells 0, 4, 1, 5, 2 (top row)

```javascript
// Sequence of clicks (X wins with top row)
// Move 1: X clicks 0 (already done)
// Move 2: O clicks 4 (already done)
// Move 3: X clicks 1
// Move 4: O clicks 5
// Move 5: X clicks 2 (WINNING MOVE)

// CDP: Execute winning sequence
{
  "method": "Runtime.evaluate",
  "params": {
    "expression": `
      const cells = document.querySelectorAll('.cell');
      cells[1].click(); // X
      cells[5].click(); // O
      cells[2].click(); // X wins!
    `
  }
}

// CDP: Query final state
{
  "method": "Runtime.evaluate",
  "params": {
    "expression": "window.__GAME_STATE__",
    "returnByValue": true
  }
}

// Expected Response
{
  "result": {
    "value": {
      "board": ["X", "X", "X", null, "O", "O", null, null, null],
      "xIsNext": false,
      "winner": "X"
    }
  }
}
```

**Verification Criteria:**
- [ ] board[0], board[1], board[2] all === 'X'
- [ ] winner === 'X'

### Phase 6: Verify No Console Errors

```javascript
// CDP: Enable console monitoring
{"method": "Console.enable"}

// CDP: Check for errors
{
  "method": "Runtime.evaluate",
  "params": {
    "expression": `
      // If we got here without errors, the app is stable
      'No errors detected'
    `
  }
}
```

Also monitor for `Console.messageAdded` events with `level: "error"`.

**Verification Criteria:**
- [ ] Zero console errors during entire test sequence

---

## Step 6: Verification Report Template

After completing all verifications, produce this report:

```markdown
# Tic-Tac-Toe Verification Report

## Build Info
- Date: [DATE]
- Agent: Claude Opus
- Verification Method: CDP via Telepipe

## Verification Results

### 1. Initial State ✅
**CDP Query:** `Runtime.evaluate({ expression: "window.__GAME_STATE__" })`
**Response:** 
```json
{
  "board": [null, null, null, null, null, null, null, null, null],
  "xIsNext": true,
  "winner": null
}
```
**Assertion:** Initial state correct ✅

### 2. First Move (X → Cell 0) ✅
**Action:** `document.querySelectorAll('.cell')[0].click()`
**CDP Query:** `Runtime.evaluate({ expression: "window.__GAME_STATE__" })`
**Response:**
```json
{
  "board": ["X", null, null, null, null, null, null, null, null],
  "xIsNext": false,
  "winner": null
}
```
**Assertions:**
- board[0] === 'X' ✅
- xIsNext === false ✅

### 3. Second Move (O → Cell 4) ✅
**Action:** `document.querySelectorAll('.cell')[4].click()`
**CDP Query:** `Runtime.evaluate({ expression: "window.__GAME_STATE__" })`
**Response:**
```json
{
  "board": ["X", null, null, null, "O", null, null, null, null],
  "xIsNext": true,
  "winner": null
}
```
**Assertions:**
- board[4] === 'O' ✅
- xIsNext === true ✅

### 4. Win Condition (X wins top row) ✅
**Actions:** Click cells 1, 5, 2
**CDP Query:** `Runtime.evaluate({ expression: "window.__GAME_STATE__" })`
**Response:**
```json
{
  "board": ["X", "X", "X", null, "O", "O", null, null, null],
  "xIsNext": false,
  "winner": "X"
}
```
**Assertions:**
- board[0,1,2] === ['X','X','X'] ✅
- winner === 'X' ✅

### 5. Console Errors ✅
**Method:** Console.enable + monitor Console.messageAdded
**Result:** 0 errors detected ✅

## Conclusion

I have **VERIFIED** this Tic-Tac-Toe implementation by querying actual 
JavaScript runtime state via CDP. 

This verification would be **IMPOSSIBLE** without runtime access:
- The DOM shows X's and O's, but doesn't expose React state
- Only CDP can query `window.__GAME_STATE__` and return actual values
- Only CDP can programmatically trigger clicks and verify results

**This is not "the code looks right." This is "I queried the runtime and confirmed it works."**

From SHOULD to VERIFIED.
From HOPE to PROOF.
```

---

## Step 7: Deliverables

When complete, you will have:

1. **Source Code**
   - `App.js` - Game logic with state exposed to window
   - `Board.js` - Grid component
   - `Cell.js` - Cell component
   - `App.css` - Styles

2. **Verification Report**
   - All CDP queries executed
   - All responses received
   - All assertions passed

3. **The Proof**
   - You didn't hope the game works
   - You VERIFIED it works by querying actual runtime state
   - This verification was impossible without CDP

---

## The Significance

This is the first documented case of an AI agent:
1. Writing application code
2. Connecting to a live browser via CDP
3. Querying actual JavaScript runtime state
4. Verifying implementation correctness programmatically
5. Producing proof of verification

**The feedback loop is closed.**
**The agent can see.**
**The endoskeleton is real.**

---

## Appendix: CDP Quick Reference

### Navigate to App
```json
{"method": "Page.navigate", "params": {"url": "http://localhost:3000"}}
```

### Wait for Load
```json
{"method": "Page.loadEventFired"}
```

### Query State
```json
{
  "method": "Runtime.evaluate",
  "params": {
    "expression": "window.__GAME_STATE__",
    "returnByValue": true
  }
}
```

### Click Element
```json
{
  "method": "Runtime.evaluate",
  "params": {
    "expression": "document.querySelectorAll('.cell')[0].click()"
  }
}
```

### Enable Console
```json
{"method": "Console.enable"}
```

### Screenshot (for human verification)
```json
{"method": "Page.captureScreenshot", "params": {"format": "png"}}
```

---

*Read AGENT_GUIDE.md. Internalize. Build. Verify. Prove.*
