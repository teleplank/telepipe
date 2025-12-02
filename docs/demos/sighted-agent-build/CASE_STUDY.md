# Case Study: The Sighted Agent Build
## First App Built by an AI That Could Verify Its Own Runtime State

---

## Executive Summary

On [DATE], we conducted an experiment: Could an AI agent build an application while verifying its own work through runtime access?

The answer is yes. And the implications are significant.

Using Telepipe for CDP (Chrome DevTools Protocol) access, Claude Opus built a Tic-Tac-Toe game while:
- Querying actual JavaScript runtime state
- Verifying React closure values matched expectations
- Confirming win detection logic worked correctly
- Monitoring for console errors in real-time

**This is the first documented case of an AI developer with "sight" - the ability to verify its own code by inspecting live runtime state.**

---

## The Problem: Blind AI Development

Traditional AI code generation follows a pattern:

```
AI writes code → AI runs tests → Tests pass → Ship → Hope
```

The AI never sees the actual runtime. It writes tests based on assumptions, and if those assumptions are wrong, the tests pass but the code fails.

**Example:**
```javascript
// AI writes this
const [count, setCount] = useState(0);

// AI assumes: "count will be 0 initially, then increment"
// AI writes test based on assumption
// Test passes (testing the assumption, not reality)
// But what if there's a subtle bug?
// AI can't see the actual runtime state
```

The feedback loop is open. The AI is blind.

---

## The Solution: Runtime State Verification

We gave the AI access to CDP via Telepipe. Now:

```
AI writes code → AI queries ACTUAL runtime state → Verified → Ship with proof
```

**The same example, with sight:**
```javascript
// AI writes this
const [count, setCount] = useState(0);

// AI queries via CDP:
Runtime.evaluate({ expression: "window.__COMPONENT_STATE__.count" })

// Response: { "result": { "value": 0 } }

// AI: "I verified count === 0. Not assumed. Queried."
```

The feedback loop closes. The AI can see.

---

## The Experiment

### Setup

1. **Telepipe** connected to Chrome via CDP (port 9222)
2. **Claude Opus** instructed to read AGENT_GUIDE.md
3. **Task:** Build Tic-Tac-Toe with verified runtime state
4. **Requirement:** Query actual JavaScript state after each move

### The Key Insight

The DOM shows X's and O's. But React's state lives in closures - invisible to normal inspection.

**Only CDP can reach into the JavaScript runtime and query:**
- What is `board[0]` right now?
- Has `xIsNext` flipped?
- Is `winner` set?

This is impossible without CDP. The agent must have runtime access to verify these values.

### Verification Points

| Checkpoint | Action | CDP Query | Result |
|------------|--------|-----------|--------|
| Initial | Load app | `window.__GAME_STATE__` | board: 9 nulls ✅ |
| Move 1 | Click cell 0 | `window.__GAME_STATE__.board[0]` | 'X' ✅ |
| Move 2 | Click cell 4 | `window.__GAME_STATE__.board[4]` | 'O' ✅ |
| Turn Check | After move 2 | `window.__GAME_STATE__.xIsNext` | true ✅ |
| Win | X completes row | `window.__GAME_STATE__.winner` | 'X' ✅ |
| Errors | Throughout | Console.messageAdded | 0 errors ✅ |

### What Made This Different

The agent didn't write:
> "The board state should update correctly when clicked."

The agent wrote:
> "I queried `window.__GAME_STATE__.board[0]` via CDP. Response: `'X'`. Verified."

**That's the difference between assumption and proof.**

---

## The Results

### Code Produced
- `App.js` - 89 lines, game logic with state exposed
- `Board.js` - 24 lines, grid component
- `Cell.js` - 18 lines, interactive cell
- `App.css` - 45 lines, styling

### Verifications Performed
- **5** state queries via CDP
- **5** click actions via CDP
- **1** console error monitoring session
- **0** errors detected

### Proof Generated
Complete verification report with:
- Actual CDP commands sent
- Actual JSON responses received
- Assertion results (all passed)

---

## Why This Matters

### For AI Development

Traditional AI coding assistants are "suggestion engines." They propose code but can't verify it works. Developers must:
- Run the code themselves
- Check the browser themselves
- Debug issues themselves

With runtime access, AI becomes a "verification engine." It can:
- Run its own code (via CDP actions)
- Check its own output (via CDP queries)
- Debug its own issues (via Console monitoring)

**The AI becomes a complete developer, not just a code generator.**

### For Quality Assurance

AI-generated code is often "plausible but wrong." It looks correct but has subtle bugs. Runtime verification catches these:

```javascript
// Plausible but wrong
const winner = calculateWinner(board);
if (winner) {
  setWinner(winner);
}

// Bug: calculateWinner returns 'X' or 'O', but what if it 
// returns undefined vs null? The AI assumed null.

// With CDP verification:
// AI queries winner value, sees undefined, catches the bug
```

### For Trust

When AI produces code with verification reports, humans can trust it more:

**Without verification:**
> "I wrote a Tic-Tac-Toe game. It should work."

**With verification:**
> "I wrote a Tic-Tac-Toe game. I verified:
> - board[0] === 'X' after first click (queried via CDP)
> - winner === 'X' after winning sequence (queried via CDP)
> - 0 console errors (monitored via CDP)
> 
> Here's the verification log."

---

## The Paradigm Shift

### Before: Exoskeleton Only

```
┌─────────────────────────────────────────┐
│                                         │
│   Tests wrap around code from outside   │
│                                         │
│      ┌─────────────────────────┐       │
│      │   ┌─────────────────┐   │       │
│      │   │                 │   │       │
│      │   │      CODE       │   │       │
│      │   │                 │   │       │
│      │   └─────────────────┘   │       │
│      │                         │       │
│      │  T E S T S  (outside)   │       │
│      └─────────────────────────┘       │
│                                         │
│   Tests verify assumptions, not reality │
│                                         │
└─────────────────────────────────────────┘
```

### After: Exoskeleton + Endoskeleton

```
┌─────────────────────────────────────────┐
│                                         │
│   Tests outside + CDP access inside     │
│                                         │
│      ┌─────────────────────────┐       │
│      │   ┌─────────────────┐   │       │
│      │   │    👁️ CDP       │   │       │
│      │   │      ↓         │   │       │
│      │   │     CODE       │   │       │
│      │   │                 │   │       │
│      │   └─────────────────┘   │       │
│      │                         │       │
│      │  T E S T S  (outside)   │       │
│      └─────────────────────────┘       │
│                                         │
│   Tests verify logic + CDP verifies     │
│   actual runtime state                  │
│                                         │
└─────────────────────────────────────────┘
```

---

## Implications

### 1. AI Can Self-Verify

AI doesn't need humans to check its work. It can check itself.

### 2. The Feedback Loop Closes

AI can iterate: write → verify → fix → verify → ship.

### 3. Higher Quality Output

Code verified against runtime is more reliable than code verified against assumptions.

### 4. New Development Paradigm

"Sighted AI development" becomes a category. Tools that give AI runtime access become essential.

---

## Conclusion

We demonstrated that an AI agent can build an application while verifying its own runtime state. This is:

- **Novel:** First documented case of AI runtime self-verification
- **Practical:** Produces higher-quality code with proof
- **Paradigm-shifting:** Changes AI from "generator" to "developer"

The agent didn't hope the code worked. It proved it worked.

**The endoskeleton is real. The feedback loop is closed. The AI can see.**

---

## Appendix: How to Reproduce

### Requirements
- Telepipe 0.3.0+
- Chrome with `--remote-debugging-port=9222`
- Node.js for React app
- Claude Opus (or similar capable model)

### Steps
1. Read `SIGHTED_AGENT_BUILD_PROMPT.md`
2. Ensure agent reads `AGENT_GUIDE.md`
3. Start React dev server
4. Connect Telepipe to Chrome
5. Execute build with verification requirements
6. Collect verification report

### Expected Outcome
- Working Tic-Tac-Toe application
- Complete verification report
- Proof of runtime state verification

---

*This case study documents the first "sighted" AI build. We expect this to become standard practice.*
