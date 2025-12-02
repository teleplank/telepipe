# Case Study: Sighted Agent Build
## The First App Built by an AI That Could Verify Its Own Runtime State

---

## What Is This?

This folder contains everything needed to run the first "sighted" AI build - where an AI agent builds an application while verifying its own runtime state through CDP.

---

## The Breakthrough

Traditional AI development:
```
Write code → Hope it works → Ship
```

Sighted AI development:
```
Write code → Query runtime state via CDP → VERIFY it works → Ship with proof
```

---

## Contents

| File | Purpose |
|------|---------|
| `QUICK_START.md` | Get running in 10 minutes |
| `SIGHTED_AGENT_BUILD_PROMPT.md` | The main prompt for the agent |
| `CONSTITUTION.md` | Technical specification for Tic-Tac-Toe |
| `CASE_STUDY.md` | Documentation of the experiment and results |

---

## The Key Innovation

The agent doesn't just write tests - it **queries actual JavaScript runtime state**.

```javascript
// Agent writes React component with useState
const [board, setBoard] = useState([...]);

// Agent queries ACTUAL closure value via CDP:
Runtime.evaluate({ expression: "window.__GAME_STATE__.board" })

// Response: ["X", null, null, null, "O", ...]

// Agent: "I verified board[0] === 'X'. Not assumed. QUERIED."
```

This is **impossible without CDP**. The DOM shows X's and O's, but only CDP can reach into JavaScript closures and return actual values.

---

## How It Works

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   1. Agent reads docs/for-agents/README.md (learns Telepipe/CDP)      │
│                           ↓                                 │
│   2. Agent reads CONSTITUTION.md (learns what to build)    │
│                           ↓                                 │
│   3. Agent writes code with state exposed to window        │
│                           ↓                                 │
│   4. Agent connects to Chrome via Telepipe (CDP)           │
│                           ↓                                 │
│   5. Agent queries window.__GAME_STATE__                   │
│                           ↓                                 │
│   6. Agent receives actual JSON: {board: [...], ...}       │
│                           ↓                                 │
│   7. Agent verifies values match expectations              │
│                           ↓                                 │
│   8. Agent produces proof: "board[0] === 'X' ✅"           │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```


```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   THE DOM:              THE CLOSURE:                        │
│   ════════              ════════════                        │
│                                                             │
│   ┌───┬───┬───┐         {                                  │
│   │ X │   │   │           board: ['X', null, null, ...]    │
│   ├───┼───┼───┤           xIsNext: false,                  │
│   │   │ O │   │           winner: null                     │
│   ├───┼───┼───┤         }                                  │
│   │   │   │   │                                            │
│   └───┴───┴───┘         ↑                                  │
│         ↑               │                                  │
│         │               ONLY CDP CAN SEE THIS              │
│   Anyone can see this   (reaches into JavaScript runtime)  │
│   (visible in browser)                                     │
│                                                             │
│   Agent queries via CDP → Gets actual JSON → VERIFIES      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Why This Matters

### For AI Development
- AI can self-verify, not just self-test
- The feedback loop closes
- Higher quality output with proof

### For Quality Assurance
- Catches bugs that pass tests
- Verifies assumptions against reality
- Produces evidence, not assertions

### For Trust
- "I verified this" vs "This should work"
- Actual CDP query responses as proof
- Reproducible verification

---

## Quick Start

```bash
# 1. Start Chrome with debugging
google-chrome --remote-debugging-port=9222

# 2. Connect Telepipe (CDP requires websocat bridge)
WS_URL=$(curl -s http://127.0.0.1:9222/json/version | grep -o 'ws://[^"]*')
./telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"

# 3. Give agent the prompt files:
#    - docs/for-agents/README.md
#    - SIGHTED_AGENT_BUILD_PROMPT.md
#    - CONSTITUTION.md

# 4. Watch the magic happen
```

See `QUICK_START.md` for detailed instructions.

---

## Expected Outcome

The agent produces:

1. **Working Tic-Tac-Toe app** - React, functional
2. **Verification report** - CDP queries with actual responses
3. **Proof of verification** - "I queried runtime state and confirmed it works"

---

## The Paradigm Shift

```
┌───────────────────────────────────────────────────┐
│                                                   │
│   BEFORE                    AFTER                 │
│   ══════                    ═════                 │
│                                                   │
│   "It should work"    →    "I verified it works" │
│   "Tests pass"        →    "Runtime state correct"│
│   "Hope"              →    "Proof"               │
│   "Blind"             →    "Sighted"             │
│                                                   │
│   The agent can SEE.                             │
│   The feedback loop CLOSES.                      │
│   The endoskeleton is REAL.                      │
│                                                   │
└───────────────────────────────────────────────────┘
```

---

## Related Files

- [docs/for-agents/README.md](../../../docs/for-agents/README.md) - Agent's guide to Telepipe capabilities
- [docs/for-agents/](../../for-agents/) - Complete agent literature
- [docs/for-agents/protocols/cdp.md](../../for-agents/protocols/cdp.md) - CDP reference

---

*This is the first documented "sighted" AI build. We expect this to become standard practice.*
