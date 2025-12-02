# Quick Start: REST API Testing Demo
## Build and Test an API with AI in 10 Minutes

---

## Prerequisites

- **Node.js 18+** installed
- **Telepipe** installed (`npm install -g telepipe`)
- **curl** (comes with macOS/Linux)
- **AI Agent** (Claude Opus, Claude Sonnet, or similar)

---

## Step 1: Give Agent the Prompts (1 minute)

Provide these files to your agent:

```
1. docs/for-agents/README.md           (from /telepipe/)
2. API_AGENT_PROMPT.md      (from docs/demos/api-testing/)
3. CONSTITUTION.md          (from docs/demos/api-testing/)
```

**Instruct your agent:**

```
Read docs/for-agents/README.md to understand Telepipe capabilities.

Then follow API_AGENT_PROMPT.md to build and test a Todo REST API.

Use CONSTITUTION.md as the API specification.

For each endpoint:
1. Build the endpoint
2. Start server via Telepipe
3. Test with curl (actual HTTP request)
4. Verify status code and response body
5. Document in test report
```

---

## Step 2: Agent Builds and Starts API (2 minutes)

Agent will create:

```
todo-api/
├── server.js
├── package.json
└── README.md
```

Agent starts server:

```bash
cd todo-api
npm install
telepipe redirect --id api -- node server.js

# Server running on http://localhost:3000
```

---

## Step 3: Agent Tests Endpoints (5 minutes)

### Test 1: Create Todo (POST)

```bash
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"Buy milk","completed":false}' \
  -w "\nStatus: %{http_code}\n"
```

**Expected:**
```json
{"id":1,"title":"Buy milk","completed":false,"createdAt":"..."}
Status: 201
```

---

### Test 2: List Todos (GET)

```bash
curl -X GET http://localhost:3000/api/todos \
  -w "\nStatus: %{http_code}\n"
```

**Expected:**
```json
[{"id":1,"title":"Buy milk","completed":false,"createdAt":"..."}]
Status: 200
```

---

### Test 3: Get Single Todo (GET)

```bash
curl -X GET http://localhost:3000/api/todos/1 \
  -w "\nStatus: %{http_code}\n"
```

**Expected:**
```json
{"id":1,"title":"Buy milk","completed":false,"createdAt":"..."}
Status: 200
```

---

### Test 4: Update Todo (PUT)

```bash
curl -X PUT http://localhost:3000/api/todos/1 \
  -H "Content-Type: application/json" \
  -d '{"title":"Buy milk","completed":true}' \
  -w "\nStatus: %{http_code}\n"
```

**Expected:**
```json
{"id":1,"title":"Buy milk","completed":true,"updatedAt":"..."}
Status: 200
```

---

### Test 5: Delete Todo (DELETE)

```bash
curl -X DELETE http://localhost:3000/api/todos/1 \
  -w "\nStatus: %{http_code}\n"
```

**Expected:**
```
Status: 204
```

---

### Test 6: Verify Deletion (GET - Should 404)

```bash
curl -X GET http://localhost:3000/api/todos/1 \
  -w "\nStatus: %{http_code}\n"
```

**Expected:**
```json
{"error":"Todo not found"}
Status: 404
```

---

### Test 7: Validation Error (POST Invalid)

```bash
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"completed":false}' \
  -w "\nStatus: %{http_code}\n"
```

**Expected:**
```json
{"error":"Title is required"}
Status: 400
```

---

## Step 4: Expected Output (2 minutes)

Agent produces a test report:

```markdown
# API Test Report

## Summary
✅ 7/7 tests passed
✅ All CRUD operations verified
✅ Error handling confirmed

## Detailed Results

### POST /api/todos
Request: {"title":"Buy milk","completed":false}
Response: 201 Created
Body: {"id":1,"title":"Buy milk","completed":false}
✅ Status correct
✅ ID generated
✅ Data matches request

### GET /api/todos
Response: 200 OK
Body: [{"id":1,...}]
✅ Status correct
✅ Array returned
✅ Contains created todo

[...continues for all 7 tests...]

## Conclusion
All endpoints functional and verified with actual HTTP requests.
```

---

## Step 5: Validate Success (1 minute)

Check that agent:

1. ✅ Started server via Telepipe
2. ✅ Made actual curl requests
3. ✅ Verified status codes
4. ✅ Validated response bodies
5. ✅ Tested error cases
6. ✅ Produced test report

---

## Troubleshooting

### Port 3000 already in use

```bash
# Kill process on port 3000
lsof -ti:3000 | xargs kill -9

# Or use different port
PORT=3001 telepipe redirect --id api -- node server.js
```

### Server won't start

```bash
# Check telepipe connection
telepipe info --id api

# Check for npm install errors
cd todo-api && npm install
```

### curl not found

```bash
# macOS/Linux: curl is pre-installed

# Windows: Use Git Bash or install curl
# Or use: Invoke-WebRequest (PowerShell)
```

### Connection refused

```bash
# Wait for server to start (2 seconds)
sleep 2

# Then test
curl http://localhost:3000/api/todos
```

---

## Success Criteria

The demo is successful when:

1. Server started via Telepipe
2. All 7 endpoints tested with curl
3. All status codes verified (201, 200, 204, 404, 400)
4. All response bodies validated
5. Test report produced with actual HTTP output

---

## What You've Achieved

You've just run AI integration testing:

- AI built a REST API
- AI started the server
- AI tested all endpoints
- AI verified responses
- AI proved functionality

**The feedback loop is closed. The AI can test HTTP.**

---

## Next Steps

### Advanced Testing

- Test concurrent requests
- Test rate limiting
- Test authentication
- Test file uploads
- Test WebSocket endpoints

### Integration Testing

- Combine with SQL (API + Database)
- Combine with CDP (API + Frontend)
- Full stack (Frontend + API + Database)

### Production Features

- Add database persistence
- Add authentication/JWT
- Add input sanitization
- Add rate limiting
- Deploy and test production

---

## Integration with Other Demos

You now have:
- **CDP** → Frontend verification
- **LLDB** → Mobile debugging
- **SQL** → Database optimization
- **API** → Integration testing

**Complete verification stack!**

---

## Cleanup

```bash
# Stop the API server
telepipe stop --id api

# Remove demo files (optional)
rm -rf todo-api/
```

---

*Welcome to AI integration testing with proof.*
