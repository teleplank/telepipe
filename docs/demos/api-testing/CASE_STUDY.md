# Case Study: AI REST API Testing
## First AI That Could Build and Test APIs with Real HTTP Requests

---

## Executive Summary

We demonstrated that an AI agent can build a REST API and test it end-to-end using actual HTTP requests.

Using Telepipe to manage the API server, Claude Opus:
- Built a complete Todo REST API (Express.js)
- Started the server via Telepipe
- Made real HTTP requests (curl)
- Verified status codes (201, 200, 404, 400)
- Inspected response bodies (JSON validation)
- Tested error cases (validation, not found)
- **Proved all CRUD operations work**

**This is the first documented case of an AI building and testing an API with HTTP verification.**

---

## The Problem: Blind API Development

Traditional AI API building:

```
Developer: "Build me a Todo API"
AI writes code
Developer: "Does it work?"
AI: "It should work..."
Developer tests manually in Postman
Developer finds bugs
Developer: "The POST endpoint is broken"
AI: "Oh, let me fix that..."
```

The AI never:
- Starts the server
- Makes HTTP requests
- Sees actual responses
- Verifies status codes
- Tests error cases

**The AI is blind to HTTP.**

---

## The Solution: HTTP Request Verification

With Telepipe:

```
AI builds API
AI starts: telepipe redirect --id api -- node server.js
AI tests: curl -X POST http://localhost:3000/api/todos -d '{...}'
AI sees: 201 Created, {"id":1,"title":"Buy milk"}
AI verifies: "Status 201 ✅, ID in response ✅"

AI tests: curl -X GET http://localhost:3000/api/todos
AI sees: 200 OK, [{"id":1,"title":"Buy milk"}]
AI verifies: "Status 200 ✅, Array with 1 item ✅"

AI tests: curl -X DELETE http://localhost:3000/api/todos/1
AI sees: 204 No Content
AI verifies: "Status 204 ✅, Deleted successfully"

AI tests: curl -X GET http://localhost:3000/api/todos/1
AI sees: 404 Not Found
AI verifies: "Status 404 ✅, Confirmed deletion"
```

**The AI tests, not assumes.**

---

## The Experiment

### Setup

1. **Node.js + Express** for API server
2. **Telepipe** to manage server process
3. **curl** for HTTP requests
4. **In-memory storage** (simple array)

### API Specification

**Endpoints implemented:**

```javascript
// Express.js routes
app.post('/api/todos', (req, res) => {
    // Create todo
});

app.get('/api/todos', (req, res) => {
    // List all todos
});

app.get('/api/todos/:id', (req, res) => {
    // Get single todo
});

app.put('/api/todos/:id', (req, res) => {
    // Update todo
});

app.delete('/api/todos/:id', (req, res) => {
    // Delete todo
});
```

**Data model:**
```javascript
{
    id: 1,
    title: "Buy milk",
    completed: false,
    createdAt: "2024-01-15T10:30:00Z"
}
```

### The Testing Session

**Step-by-step verification:**

---

#### Test 1: Create Todo (POST)

**Request:**
```bash
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"Buy milk","completed":false}' \
  -w "\nStatus: %{http_code}\n" \
  -s
```

**Response:**
```json
{"id":1,"title":"Buy milk","completed":false,"createdAt":"2024-01-15T10:30:00.123Z"}
Status: 201
```

**AI Verification:**
- ✅ Status code: 201 Created
- ✅ Response contains id field
- ✅ Response contains title field
- ✅ Response contains completed field
- ✅ Response contains createdAt timestamp

---

#### Test 2: List Todos (GET Collection)

**Request:**
```bash
curl -X GET http://localhost:3000/api/todos \
  -w "\nStatus: %{http_code}\n" \
  -s
```

**Response:**
```json
[{"id":1,"title":"Buy milk","completed":false,"createdAt":"2024-01-15T10:30:00.123Z"}]
Status: 200
```

**AI Verification:**
- ✅ Status code: 200 OK
- ✅ Response is an array
- ✅ Array contains 1 item
- ✅ Item matches created todo

---

#### Test 3: Get Single Todo (GET)

**Request:**
```bash
curl -X GET http://localhost:3000/api/todos/1 \
  -w "\nStatus: %{http_code}\n" \
  -s
```

**Response:**
```json
{"id":1,"title":"Buy milk","completed":false,"createdAt":"2024-01-15T10:30:00.123Z"}
Status: 200
```

**AI Verification:**
- ✅ Status code: 200 OK
- ✅ Response is an object
- ✅ ID matches requested ID (1)
- ✅ Data matches created todo

---

#### Test 4: Update Todo (PUT)

**Request:**
```bash
curl -X PUT http://localhost:3000/api/todos/1 \
  -H "Content-Type: application/json" \
  -d '{"title":"Buy milk","completed":true}' \
  -w "\nStatus: %{http_code}\n" \
  -s
```

**Response:**
```json
{"id":1,"title":"Buy milk","completed":true,"createdAt":"2024-01-15T10:30:00.123Z","updatedAt":"2024-01-15T10:31:00.456Z"}
Status: 200
```

**AI Verification:**
- ✅ Status code: 200 OK
- ✅ completed changed from false to true
- ✅ updatedAt timestamp added
- ✅ Other fields unchanged

---

#### Test 5: Delete Todo (DELETE)

**Request:**
```bash
curl -X DELETE http://localhost:3000/api/todos/1 \
  -w "\nStatus: %{http_code}\n" \
  -s
```

**Response:**
```
Status: 204
```

**AI Verification:**
- ✅ Status code: 204 No Content
- ✅ No response body (correct for DELETE)

---

#### Test 6: Verify Deletion (GET - Should 404)

**Request:**
```bash
curl -X GET http://localhost:3000/api/todos/1 \
  -w "\nStatus: %{http_code}\n" \
  -s
```

**Response:**
```json
{"error":"Todo not found"}
Status: 404
```

**AI Verification:**
- ✅ Status code: 404 Not Found
- ✅ Error message in response
- ✅ Deletion confirmed

---

#### Test 7: Validation Error (POST Invalid Data)

**Request:**
```bash
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"completed":false}' \
  -w "\nStatus: %{http_code}\n" \
  -s
```
(Missing required 'title' field)

**Response:**
```json
{"error":"Title is required"}
Status: 400
```

**AI Verification:**
- ✅ Status code: 400 Bad Request
- ✅ Error message explains issue
- ✅ Validation working correctly

---

## The Results

### Complete Test Coverage

| Test | Method | Endpoint | Status | Result |
|------|--------|----------|--------|--------|
| Create | POST | /api/todos | 201 | ✅ Todo created |
| List | GET | /api/todos | 200 | ✅ Array returned |
| Read | GET | /api/todos/1 | 200 | ✅ Todo retrieved |
| Update | PUT | /api/todos/1 | 200 | ✅ Todo updated |
| Delete | DELETE | /api/todos/1 | 204 | ✅ Todo deleted |
| Verify Delete | GET | /api/todos/1 | 404 | ✅ Not found |
| Invalid Create | POST | /api/todos | 400 | ✅ Validation error |

**7 tests, 7 passed, 100% success rate**

### Code Produced

```
server.js       - Express API (~120 lines)
package.json    - Dependencies
test-report.md  - Complete test results
```

### Verification Method

Every endpoint tested with:
1. Actual curl HTTP request
2. Real server response
3. Status code verification
4. Response body validation
5. Error case testing

**All results backed by actual HTTP responses.**

---

## Why This Matters

### For API Development

AI becomes a complete API developer:
- Writes endpoints
- Tests endpoints
- Verifies responses
- Catches bugs
- No human testing needed

### For Quality Assurance

Catches bugs immediately:
- Missing validation
- Wrong status codes
- Incorrect response format
- Error handling issues

### For Trust

When AI builds with verification reports:

**Without testing:**
> "I built a Todo API. It should work."

**With testing:**
> "I built a Todo API. Test results:
> - POST /api/todos: 201 ✅
> - GET /api/todos: 200 ✅
> - PUT /api/todos/1: 200 ✅
> - DELETE /api/todos/1: 204 ✅
> - All CRUD operations verified ✅
>
> Here's the complete test report."

---

## The Paradigm Shift

### Before: Code Without Testing

```
┌─────────────────────────────────────────┐
│                                         │
│   AI writes API code                    │
│                                         │
│      ┌─────────────────────────┐       │
│      │                         │       │
│      │    app.post(...)        │       │
│      │    app.get(...)         │       │
│      │    app.put(...)         │       │
│      │    app.delete(...)      │       │
│      │                         │       │
│      │    (untested)           │       │
│      └─────────────────────────┘       │
│                                         │
│   Human must test in Postman           │
│                                         │
└─────────────────────────────────────────┘
```

### After: Code With Integrated Testing

```
┌─────────────────────────────────────────┐
│                                         │
│   AI writes AND tests API               │
│                                         │
│      ┌─────────────────────────┐       │
│      │    HTTP Testing         │       │
│      │         ↓               │       │
│      │    app.post(...)        │       │
│      │    ✅ 201 Created       │       │
│      │                         │       │
│      │    app.get(...)         │       │
│      │    ✅ 200 OK            │       │
│      │                         │       │
│      │    app.put(...)         │       │
│      │    ✅ 200 OK            │       │
│      │                         │       │
│      │    app.delete(...)      │       │
│      │    ✅ 204 No Content    │       │
│      │                         │       │
│      │    (fully tested)       │       │
│      └─────────────────────────┘       │
│                                         │
│   AI verified all endpoints             │
│                                         │
└─────────────────────────────────────────┘
```

---

## Implications

### 1. AI Can Self-Test

AI doesn't need humans to test APIs. It can:
- Start servers
- Make requests
- Verify responses
- Test edge cases

### 2. The Feedback Loop Closes

AI can iterate:
```
Write endpoint → Test → Fix bugs → Test again → Ship with proof
```

### 3. Higher Quality APIs

APIs verified against real HTTP responses are more reliable than untested code.

### 4. New Development Paradigm

"AI integration testing" becomes standard. Tools that give AI HTTP access become essential.

---

## Integration Opportunities

### Combine with SQL Demo

```
1. AI creates todo via POST /api/todos
2. AI verifies HTTP response: 201 ✅
3. AI queries database: SELECT * FROM todos WHERE id = 1
4. AI verifies database: Row exists ✅
5. End-to-end verification: API + Database
```

### Combine with CDP Demo

```
1. AI creates todo via POST /api/todos
2. AI verifies HTTP response: 201 ✅
3. AI queries React state via CDP: window.__APP_STATE__.todos
4. AI verifies UI updated: [{"id":1,"title":"Buy milk"}] ✅
5. End-to-end verification: API + Frontend
```

### Full Stack Integration

```
Frontend (CDP) ← → API (HTTP) ← → Database (SQL)
       ✅              ✅              ✅
    All verified with actual runtime access
```

---

## The Full-Stack Tetralogy Complete

With CDP, LLDB, SQL, and API demos, we've shown AI can verify:

| Layer | Technology | Runtime Access | Demo |
|-------|------------|----------------|------|
| Frontend | React | CDP (Chrome DevTools) | Sighted Agent Build |
| Mobile | SwiftUI | LLDB (Debugger) | LLDB Debugging |
| Backend | PostgreSQL | EXPLAIN ANALYZE | SQL Optimization |
| Integration | REST API | HTTP (curl) | API Testing |

**Complete stack coverage with runtime verification.**

---

## Conclusion

We demonstrated that an AI agent can build a REST API and test it end-to-end using real HTTP requests. This is:

- **Novel:** First AI API development with HTTP verification
- **Practical:** Produces tested APIs with proof
- **Paradigm-shifting:** Changes AI from "code generator" to "full-stack developer"

The agent didn't hope the API worked. It proved it worked.

**The endoskeleton reaches HTTP. The feedback loop is closed. The AI can test.**

---

## Appendix: How to Reproduce

### Requirements
- Node.js 18+
- Telepipe
- curl (standard on macOS/Linux)
- Claude Opus (or similar capable model)

### Steps
1. Give agent API_AGENT_PROMPT.md
2. Agent builds Express API
3. Start server via Telepipe
4. Agent tests all endpoints with curl
5. Agent verifies responses
6. Collect test report

### Expected Outcome
- Working REST API
- Complete test report with HTTP responses
- Proof of CRUD functionality

---

*This case study demonstrates AI integration testing with HTTP verification.*
