# Demo: REST API Testing via Telepipe
## The First AI That Could Build and Test APIs with Real HTTP Requests

---

## What Is This?

This folder contains everything needed to watch an AI agent **build a REST API and test every endpoint with actual HTTP requests** - verifying status codes, response bodies, and error handling with proof.

This completes the **full-stack AI verification tetralogy**:
- **CDP** = Frontend (React state verification)
- **LLDB** = Mobile (iOS runtime debugging)
- **SQL** = Database (query optimization)
- **API** = Integration (HTTP endpoint testing)

---

## The Breakthrough

Traditional AI API development:
```
Write API code → Hope it works → Ship → Pray
```

Testing-enabled AI API development:
```
Write API → Start server → Make HTTP requests → Verify responses → Ship with proof
```

---

## Contents

| File | Purpose |
|------|---------|
| `QUICK_START.md` | Get testing in 10 minutes |
| `API_AGENT_PROMPT.md` | The main prompt for the agent |
| `CONSTITUTION.md` | Technical specification for the demo API |
| `CASE_STUDY.md` | Documentation of the experiment and results |

---

## The Key Innovation

The agent sees ACTUAL HTTP responses, not assumptions.

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   REQUEST:                                                  │
│   ════════                                                  │
│                                                             │
│   POST /api/todos                                           │
│   {"title":"Buy milk","completed":false}                    │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ACTUAL RESPONSE:                                          │
│   ════════════════                                          │
│                                                             │
│   HTTP/1.1 201 Created                                      │
│   Content-Type: application/json                            │
│   {"id":1,"title":"Buy milk","completed":false}             │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   AI VERIFICATION:                                          │
│   ════════════════                                          │
│                                                             │
│   ✅ Status code: 201 (Created)                             │
│   ✅ Response body contains id                              │
│   ✅ Response body contains title                           │
│   ✅ Content-Type is application/json                       │
│                                                             │
│   ENDPOINT VERIFIED! 🎯                                     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Not hoped. Verified.**

---

## How It Works

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   1. Agent builds Express API                               │
│                           ↓                                 │
│   2. Agent starts server via Telepipe                       │
│      telepipe redirect --id api -- node server.js           │
│                           ↓                                 │
│   3. Agent makes HTTP requests with curl                    │
│      curl -X POST http://localhost:3000/api/todos           │
│                           ↓                                 │
│   4. Agent sees actual response                             │
│      {"id":1,"title":"Buy milk"} Status: 201                │
│                           ↓                                 │
│   5. Agent verifies status code and body                    │
│      ✅ 201 Created, ✅ ID in response                      │
│                           ↓                                 │
│   6. Agent tests all CRUD operations                        │
│      POST, GET, PUT, DELETE, error cases                    │
│                           ↓                                 │
│   7. Agent produces test report with proof                  │
│      "All 7 endpoints verified ✅"                          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Complete CRUD Verification

| Operation | Request | Status | Verified |
|-----------|---------|--------|----------|
| Create | POST /api/todos | 201 | ✅ ID returned |
| Read All | GET /api/todos | 200 | ✅ Array with items |
| Read One | GET /api/todos/1 | 200 | ✅ Correct data |
| Update | PUT /api/todos/1 | 200 | ✅ Data changed |
| Delete | DELETE /api/todos/1 | 204 | ✅ No content |
| Verify Delete | GET /api/todos/1 | 404 | ✅ Not found |
| Invalid Create | POST (bad data) | 400 | ✅ Error message |

**All verified with actual HTTP responses.**

---

## Why This Matters

### For API Development
- AI becomes a complete API developer (build + test)
- Catches endpoint bugs immediately
- Verifies correct status codes
- Validates response schemas

### For Quality Assurance
- Integration testing without humans
- Error case coverage
- Response body validation
- Status code verification

### For Trust
- "All endpoints tested" backed by real HTTP responses
- Before/after evidence for debugging
- Reproducible and verifiable

---

## Quick Start

```bash
# 1. Ensure Node.js is installed
node --version  # Should be 18+

# 2. Give agent the prompt files:
#    - docs/for-agents/README.md
#    - API_AGENT_PROMPT.md
#    - CONSTITUTION.md

# 3. Agent builds API, starts server via Telepipe
telepipe redirect --id api -- node server.js

# 4. Agent tests all endpoints with curl
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"Buy milk"}' \
  -w "\nStatus: %{http_code}\n"

# 5. Watch the agent verify with proof
```

See `QUICK_START.md` for detailed instructions.

---

## Expected Outcome

The agent produces:

1. **Working REST API** - Express.js with all CRUD endpoints
2. **Complete test report** - All 7 test cases with HTTP output
3. **Verification evidence** - Status codes and response bodies
4. **Proof of functionality** - Not assumptions, actual HTTP responses

---

## The Paradigm Shift

```
┌───────────────────────────────────────────────────┐
│                                                   │
│   BEFORE                    AFTER                 │
│   ══════                    ═════                 │
│                                                   │
│   "Should return 201"   →  "Status: 201 ✅"       │
│   "Body has id"         →  "{"id":1} ✅"          │
│   "DELETE works"        →  "Status: 204 ✅"       │
│   "Hope"                →  "All verified ✅"      │
│                                                   │
│   The agent can TEST.                             │
│   The feedback loop CLOSES.                       │
│   The endoskeleton reaches HTTP.                  │
│                                                   │
└───────────────────────────────────────────────────┘
```

---

## Integration with Other Demos

Combine API testing with other verification methods:

### API + SQL
```
1. POST /api/todos → 201 ✅
2. SELECT * FROM todos → Row exists ✅
3. End-to-end: API creates, database stores
```

### API + CDP
```
1. POST /api/todos → 201 ✅
2. Frontend fetches → window.__STATE__.todos ✅
3. End-to-end: API serves, UI displays
```

### Full Stack
```
Frontend (CDP) ← → API (HTTP) ← → Database (SQL)
     ✅              ✅              ✅
  All verified with actual runtime access
```

---

## The Full-Stack Tetralogy

With this demo, AI can now verify across the entire stack:

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   FRONTEND        MOBILE          BACKEND        INTEGRATION│
│   ════════        ══════          ═══════        ═══════════│
│                                                             │
│   CDP             LLDB            SQL            HTTP       │
│   Chrome          LLVM            PostgreSQL     REST API   │
│   JavaScript      Swift runtime   Execution      Endpoints  │
│                                                             │
│   window.STATE    po viewModel    EXPLAIN        curl -X    │
│                                                             │
│   "board[0]=X ✅" "items=0 ❌"    "52ms ✅"      "201 ✅"   │
│                                                             │
│   COMPLETE VERIFICATION STACK 🎯                            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Related Files

- [docs/for-agents/README.md](../../../docs/for-agents/README.md) - Agent's guide to Telepipe capabilities
- [Sighted Agent Build](../sighted-agent-build/) - The CDP demo (frontend)
- [LLDB Debugging](../lldb-debugging/) - The LLDB demo (mobile)
- [SQL Optimization](../sql-optimization/) - The SQL demo (backend)

---

*This demo completes the full-stack AI verification tetralogy. Frontend + Mobile + Backend + Integration = Complete.*
