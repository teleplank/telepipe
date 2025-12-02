# Demo: LLDB Debugging via Telepipe
## The First AI That Could Debug Its Own iOS Code

---

## What Is This?

This folder contains everything needed to watch an AI agent **debug an iOS app in real-time using LLDB** - inspecting actual Swift runtime state, finding bugs, and verifying fixes.

This is the iOS equivalent of our CDP breakthrough. Different runtime, same paradigm: **AI with sight**.

---

## The Breakthrough

Traditional AI iOS development:
```
Write Swift code → Hope it compiles → Hope it runs → Ship → Pray
```

LLDB-enabled AI development:
```
Write Swift code → Attach LLDB → Query runtime state → VERIFY it works → Ship with proof
```

---

## Contents

| File | Purpose |
|------|---------|
| `QUICK_START.md` | Get debugging in 10 minutes |
| `LLDB_AGENT_PROMPT.md` | The main prompt for the agent |
| `CONSTITUTION.md` | Technical specification for TodoApp |
| `CASE_STUDY.md` | Documentation of the experiment and results |

---

## The Key Innovation

The agent doesn't just write tests - it **inspects actual Swift runtime state**.

```swift
// Agent writes SwiftUI code with @State
@State private var items: [TodoItem] = []

// Agent queries ACTUAL value via LLDB:
(lldb) po viewModel.items.count
1

// Agent: "I verified items.count === 1. Not assumed. QUERIED."
```

This is **impossible without LLDB**. The UI shows todo items, but only LLDB can reach into Swift memory and return actual object values.

---

## How It Works

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   1. Agent reads docs/for-agents/README.md (learns Telepipe/LLDB)     │
│                           ↓                                 │
│   2. Agent reads CONSTITUTION.md (learns what to build)    │
│                           ↓                                 │
│   3. Agent builds TodoApp in Swift/SwiftUI                 │
│                           ↓                                 │
│   4. Agent attaches LLDB via Telepipe                      │
│                           ↓                                 │
│   5. Agent queries: po viewModel.items                     │
│                           ↓                                 │
│   6. Agent receives actual values from Swift runtime       │
│                           ↓                                 │
│   7. Agent discovers bug: items don't persist!             │
│                           ↓                                 │
│   8. Agent fixes bug and verifies via LLDB                 │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```


```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   THE UI:                 THE SWIFT RUNTIME:                │
│   ═══════                 ══════════════════                │
│                                                             │
│   ┌─────────────────┐     class TodoViewModel {             │
│   │ ☑ Buy milk      │       var items = [                   │
│   │ ☐ Walk dog      │         TodoItem("Buy milk"),         │
│   │                 │         TodoItem("Walk dog")          │
│   │ [Add Todo]      │       ]                               │
│   └─────────────────┘     }                                 │
│           ↑                         ↑                       │
│           │                         │                       │
│   Anyone sees this          ONLY LLDB CAN SEE THIS         │
│   (visible on screen)       (reaches into Swift memory)     │
│                                                             │
│   Agent queries via LLDB → Gets actual objects → VERIFIES  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Why This Matters

### For AI iOS Development
- AI can self-debug, not just compile
- Finds bugs that unit tests miss
- Inspects actual object state at runtime

### For Quality Assurance
- Catches persistence bugs
- Verifies state management
- Proves memory lifecycle correctness

### For Trust
- "I debugged this" vs "It compiles"
- Actual LLDB output as proof
- Reproducible debugging sessions

---

## Quick Start

```bash
# 1. Build and run TodoApp in iOS Simulator (from Xcode)
# App appears in simulator

# 2. Attach LLDB via Telepipe
telepipe redirect --id lldb -- lldb -n TodoApp

# 3. Give agent the prompt files:
#    - docs/for-agents/README.md
#    - LLDB_AGENT_PROMPT.md
#    - CONSTITUTION.md

# 4. Watch the agent debug in real-time
```

See `QUICK_START.md` for detailed instructions.

---

## Expected Outcome

The agent produces:

1. **Working TodoApp** - SwiftUI, functional
2. **Bug discovery** - Found via LLDB inspection
3. **Bug fix** - Verified via LLDB queries
4. **Debugging report** - LLDB commands with actual responses

---

## The Paradigm Shift

```
┌───────────────────────────────────────────────────┐
│                                                   │
│   BEFORE                    AFTER                 │
│   ══════                    ═════                 │
│                                                   │
│   "It compiles"       →    "I inspected runtime" │
│   "Tests pass"        →    "LLDB shows correct"  │
│   "Should persist"    →    "po items.count = 3"  │
│   "Blind"             →    "Sighted"             │
│                                                   │
│   The agent can DEBUG.                           │
│   The feedback loop CLOSES.                      │
│   The endoskeleton reaches iOS.                  │
│                                                   │
└───────────────────────────────────────────────────┘
```

---

## Related Files

- [docs/for-agents/README.md](../../../docs/for-agents/README.md) - Agent's guide to Telepipe capabilities
- [docs/for-agents/](../../for-agents/) - Complete agent literature
- [Sighted Agent Build](../sighted-agent-build/) - The CDP equivalent for web apps

---

*This is the first documented AI debugging session for iOS. LLDB + Telepipe = Sighted iOS AI.*
