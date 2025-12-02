# Telepipe FAQ
## Common Questions About Tests, Mocks, and Runtime Access

---

## "Why not just write unit tests?"

**Great question! You should write unit tests.**

Unit tests are essential. They're fast, isolated, repeatable, and catch regressions.

Telepipe doesn't replace tests. It complements them.

```
Tests verify:     "My code does what I expected"
Telepipe verifies: "What I expected is what actually happens"

Both together:    "My code does what actually needs to happen"
```

Think about how you work: You write tests AND you look at the browser. Both. Telepipe gives AI the "look at the browser" part.

---

## "That's what mocks are for"

Mocks are valuable! They let you test quickly and in isolation.

But mocks encode assumptions:

```javascript
// This mock assumes the API returns { users: [...] }
mockApi.getUsers.mockReturnValue({ users: [...] });
```

What if the API actually returns `{ data: { users: [...] } }`?

Your tests pass. Your code breaks.

**Mocks are maps. Telepipe checks the territory.**

Use mocks for fast iteration. Use Telepipe to verify your mocks match reality.

---

## "Integration tests already exist"

They do! And they're great.

The difference is interactivity:

```
Traditional Integration Test:
  Script runs → Pass/Fail → Done

Telepipe:
  Agent connects → Explores → Understands → Adjusts → Continues
```

Integration tests are pre-written scripts. Telepipe is live access.

For AI, this matters because:
- AI can investigate unexpected behavior
- AI can adapt to what it sees
- AI can verify specific concerns dynamically

---

## "This seems like overkill"

Fair concern! Let's think about when Telepipe adds value:

**Overkill:**
- Simple logic-only code
- Code with no external dependencies
- Quick prototypes you'll run yourself

**Not overkill:**
- AI building UIs it can't see
- AI writing database queries against real schemas
- AI integrating with APIs it has assumptions about
- Any time "tests pass but it doesn't work"

If you've ever had a production bug that passed all tests, you understand the gap Telepipe fills.

---

## "Real developers don't need this"

Real developers have browsers and terminals. They look at their output constantly.

```
Developer workflow:
  Code → Save → Look at browser → "Hmm, not quite" → Adjust

AI workflow (traditional):
  Code → Test → Tests pass → Hope
```

Telepipe gives AI the same feedback loop humans naturally have.

**This isn't for humans. This is for AI agents that can't look at browsers.**

---

## "Just run the code and check"

Exactly! That's what Telepipe enables.

The question is: who runs it and checks?

```
Human developer: Runs code, looks at output, checks manually
AI agent (without Telepipe): Can't run against real systems, can't look
AI agent (with Telepipe): Runs code, looks at output, checks automatically
```

"Just run and check" is great advice. Telepipe is how AI follows it.

---

## "Isn't this slow compared to unit tests?"

Yes, runtime verification is slower than unit tests.

That's why you use both:

```
Development cycle:
  1. Write code
  2. Run unit tests (fast, frequent)
  3. Run unit tests (fast, frequent)
  4. Run unit tests (fast, frequent)
  5. Verify with Telepipe (slower, key checkpoints)
  6. Ship with confidence
```

Unit tests for tight feedback loops. Telepipe for reality checks.

---

## "What about test environments?"

Test environments are great! Telepipe works with them.

```bash
# Connect to your test database (direct TCP - no bridge needed)
telepipe connect --id testdb --host test.db.internal --port 5432

# Connect to your staging browser (CDP uses WebSockets, requires websocat bridge)
WS_URL=$(curl -s http://staging.internal:9222/json/version | grep -o 'ws://[^"]*')
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

Telepipe doesn't require production access. It just provides access to whatever systems you point it at.

---

## "How is this different from debugging?"

Telepipe is runtime access. Debugging is one use case.

```
What Telepipe enables:
  • Debugging (inspect state)
  • Verification (confirm behavior)
  • Exploration (understand systems)
  • Testing (dynamic assertions)
  • Monitoring (watch behavior)
```

It's the connection. What you do with it is up to you.

---

## "Can't AI just read logs?"

Logs help, but they're:
- After-the-fact (not interactive)
- Pre-defined (only what someone thought to log)
- Text-only (can't see UI, can't screenshot)

Telepipe provides:
- Live interaction
- Full access (not just logged events)  
- Rich data (screenshots, DOM, query results)

Logs are useful. Live access is more useful.

---

## "This adds complexity"

Some, yes. The tradeoff:

```
Without Telepipe:
  + Simpler setup
  - AI works blind
  - Assumptions may be wrong
  - "Works on my machine" bugs

With Telepipe:
  + AI can verify its work
  + Catches assumption errors
  + Higher confidence output
  - Need to run Telepipe
```

If your AI builds things that work first try every time, you don't need this.

If you've ever had AI produce code that "should work" but doesn't, Telepipe helps.

---

## "What about security?"

Telepipe runs locally. It connects to systems you specify.

```
Security model:
  • Telepipe doesn't phone home
  • Connects only where you tell it
  • Same trust model as any database client
  • You control what AI can access
```

If you trust your AI with your code, you can trust it with supervised runtime access.

---

## "Is this production-safe?"

Telepipe is a development tool, not a production deployment pattern.

```
Intended use:
  ✓ Development
  ✓ Testing
  ✓ Staging verification
  ✗ Production monitoring (use proper observability tools)
```

Use it to build and verify. Use other tools to monitor production.

---

## The One-Sentence Summary

**Tests verify code against expectations. Telepipe verifies expectations against reality. Use both.**

---

*Still have questions? Open an issue on GitHub. We're happy to discuss.*
