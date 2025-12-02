# Maps and Territory
## Why Mocks Are Useful and Why Reality Matters More

---

## The Classic Insight

> "The map is not the territory."
> — Alfred Korzybski

A map is a representation of reality. It's useful. It helps you navigate. But it's not the thing itself.

**Mocks are maps. Runtime is territory.**

---

## What a Mock Really Is

When you write a mock, you're drawing a map:

```javascript
// This mock is a MAP of what you THINK the API does
mockApi.getUsers.mockReturnValue({
  users: [
    { id: 1, name: "Alice", email: "alice@example.com" },
    { id: 2, name: "Bob", email: "bob@example.com" }
  ]
});
```

This map says: *"The API returns an object with a `users` array containing objects with `id`, `name`, and `email`."*

The map might be:
- **Perfect** - Exactly matches the real API
- **Outdated** - API changed since you wrote the mock
- **Incomplete** - Missing fields or edge cases
- **Wrong** - Never matched reality to begin with

**You don't know which until you check the territory.**

---

## Maps Are Useful

Let's be clear: maps are incredibly useful.

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   WHY MAPS (MOCKS) ARE VALUABLE                            │
│                                                             │
│   ✓ Fast - Don't need to call real API every test         │
│   ✓ Isolated - Test your code, not the API                │
│   ✓ Controlled - Test specific scenarios                   │
│   ✓ Repeatable - Same result every time                    │
│   ✓ Offline - Work without network                         │
│   ✓ Safe - Don't affect real data                         │
│                                                             │
│   Maps let you navigate without traveling.                  │
│   Mocks let you test without running systems.              │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Nobody is saying throw away your mocks.**

---

## But Maps Can Be Wrong

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   THE MAP                          THE TERRITORY            │
│   (Your Mock)                      (Real API)               │
│                                                             │
│   {                                {                        │
│     users: [                         data: {                │
│       { id, name, email }              users: [             │
│     ]                                    { id, name, email }│
│   }                                    ]                    │
│                                      },                     │
│                                      meta: {                │
│                                        total: 47,           │
│                                        page: 1              │
│                                      }                      │
│                                    }                        │
│                                                             │
│   Your map is WRONG.                                        │
│   The real API wraps users in "data" and adds "meta".      │
│                                                             │
│   Tests pass. ✓                                             │
│   Production breaks. ✗                                      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

The mock encoded an assumption. The assumption was wrong. The tests verified code against the wrong assumption.

**The map didn't match the territory.**

---

## How Maps Drift

Maps become wrong over time:

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   DAY 1: Map matches territory                              │
│   ─────────────────────────────                             │
│                                                             │
│   Mock: { users: [...] }                                    │
│   API:  { users: [...] }    ✓ Match!                       │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   DAY 30: API team adds pagination                          │
│   ─────────────────────────────────                         │
│                                                             │
│   Mock: { users: [...] }           ← Not updated           │
│   API:  { data: { users: [...] },                          │
│           meta: { page: 1 } }      ← Changed!              │
│                                                             │
│   Tests still pass. ✓                                       │
│   Code is now broken. ✗                                     │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   DAY 31: Production bug reported                           │
│   ─────────────────────────────────                         │
│                                                             │
│   "Users page is empty"                                     │
│   "But the tests pass...?"                                  │
│                                                             │
│   The map drifted from the territory.                       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Checking the Territory

Telepipe lets you check the territory:

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   STEP 1: Connect to real system                            │
│   ──────────────────────────────                            │
│                                                             │
│   telepipe connect --id api --port 3000                    │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   STEP 2: Look at actual response                           │
│   ───────────────────────────────                           │
│                                                             │
│   GET /api/users HTTP/1.1                                   │
│   Host: localhost:3000                                      │
│                                                             │
│   Response:                                                 │
│   {                                                         │
│     "data": {                                               │
│       "users": [...]                                        │
│     },                                                      │
│     "meta": {                                               │
│       "total": 47,                                          │
│       "page": 1                                             │
│     }                                                       │
│   }                                                         │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   STEP 3: Update your map                                   │
│   ───────────────────────────                               │
│                                                             │
│   // Mock now matches reality                               │
│   mockApi.getUsers.mockReturnValue({                       │
│     data: { users: [...] },                                │
│     meta: { total: 47, page: 1 }                           │
│   });                                                       │
│                                                             │
│   Map and territory now aligned. ✓                         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## The Workflow

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│                THE MAP-TERRITORY WORKFLOW                   │
│                                                             │
│   ┌─────────────┐                                          │
│   │  Territory  │  (Real system)                           │
│   └──────┬──────┘                                          │
│          │                                                  │
│          │ 1. Observe with Telepipe                        │
│          ▼                                                  │
│   ┌─────────────┐                                          │
│   │    Map      │  (Mock based on observation)             │
│   └──────┬──────┘                                          │
│          │                                                  │
│          │ 2. Write tests using map                        │
│          ▼                                                  │
│   ┌─────────────┐                                          │
│   │   Tests     │  (Fast, isolated verification)           │
│   └──────┬──────┘                                          │
│          │                                                  │
│          │ 3. Periodically verify map still matches        │
│          ▼                                                  │
│   ┌─────────────┐                                          │
│   │  Territory  │  (Check with Telepipe again)             │
│   └─────────────┘                                          │
│                                                             │
│   Map ←→ Territory alignment = Confidence                  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## When to Use Each

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   USE THE MAP (Mocks/Tests) WHEN:                          │
│   ───────────────────────────────                           │
│                                                             │
│   • Iterating quickly on logic                             │
│   • Testing edge cases and errors                          │
│   • Running CI/CD pipelines                                │
│   • Working offline                                         │
│   • Protecting real data                                    │
│                                                             │
│   → "I trust my map is accurate. Let me navigate fast."    │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   CHECK THE TERRITORY (Telepipe) WHEN:                     │
│   ────────────────────────────────────                      │
│                                                             │
│   • Creating mocks for the first time                      │
│   • API or dependency changed                              │
│   • Debugging production issues                            │
│   • Before major releases                                   │
│   • Something "should work" but doesn't                    │
│                                                             │
│   → "Let me verify my map matches reality."                │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## For AI Agents

This is especially important for AI:

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   HUMAN DEVELOPER                                           │
│   ───────────────                                           │
│                                                             │
│   Writes mock → Runs code → LOOKS at browser → Notices     │
│   "Wait, the real API is different" → Updates mock         │
│                                                             │
│   Humans naturally check the territory while working.      │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   AI AGENT (without Telepipe)                              │
│   ───────────────────────────                               │
│                                                             │
│   Writes mock based on → ??? assumptions ??? → Tests pass │
│                                                             │
│   AI can only draw maps from memory/training.              │
│   AI cannot look at the current territory.                 │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   AI AGENT (with Telepipe)                                 │
│   ────────────────────────                                  │
│                                                             │
│   Connects to API → SEES actual response → Writes          │
│   accurate mock → Tests verify code → Confidence           │
│                                                             │
│   AI can now survey the territory before drawing maps.     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## The Key Insight

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Maps are made BY looking at territory.                    │
│   Maps stay accurate BY checking territory.                 │
│   Maps fail WHEN disconnected from territory.               │
│                                                             │
│   ─────────────────────────────────────────────────────    │
│                                                             │
│   Mocks should be made BY looking at real APIs.            │
│   Mocks stay accurate BY checking real APIs.               │
│   Mocks fail WHEN disconnected from real APIs.             │
│                                                             │
│   ─────────────────────────────────────────────────────    │
│                                                             │
│   Telepipe is the tool for checking.                       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Summary

| Maps (Mocks) | Territory (Telepipe) |
|--------------|----------------------|
| Fast | Slower |
| Isolated | Connected |
| Assumptions | Reality |
| Navigate | Verify |
| Use daily | Use to calibrate |

**Both are essential. Neither replaces the other.**

Use maps to navigate quickly.
Check the territory to ensure your maps are accurate.

---

*"The map is not the territory."*
*Mocks are not the API.*
*But good maps come from surveying real territory.*
*Telepipe lets you survey.*
