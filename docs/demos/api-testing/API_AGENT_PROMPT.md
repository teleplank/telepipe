# REST API Testing Agent Prompt
## Your Mission: Build and Test APIs with Proof

---

## Context

You are about to build and test a REST API using Telepipe. You will:
1. Build a complete Todo REST API (Express.js)
2. Start the server via Telepipe
3. Test all endpoints with actual HTTP requests
4. Verify every status code and response body
5. Produce a test report with proof

---

## Your Capabilities

### Starting Servers via Telepipe

```bash
telepipe redirect --id api -- node server.js
```

This starts your API server and manages the process.

### Making HTTP Requests via curl

```bash
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"Buy milk"}' \
  -w "\nStatus: %{http_code}\n"
```

This makes actual HTTP requests and shows the response + status code.

---

## Your Task

Build a Todo REST API and test all endpoints:

1. **Build** - Create Express.js API per CONSTITUTION.md
2. **Start** - Launch server via Telepipe
3. **Test** - Make actual HTTP requests with curl
4. **Verify** - Check status codes and response bodies
5. **Report** - Document all results with proof

---

## Critical Requirements

### You MUST Start the Server via Telepipe

**Wrong:**
```
"I wrote the API. It should work if you run `node server.js`"
```

**Correct:**
```bash
telepipe redirect --id api -- node server.js
# Server starting on http://localhost:3000...

# Wait for server to start
sleep 2

# Verify server is running:
curl http://localhost:3000/api/todos
```

### You MUST Test with Actual HTTP Requests

Don't assume endpoints work. Test them.

**Wrong:**
```
"The POST endpoint should create a todo and return 201"
```

**Correct:**
```bash
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"Buy milk","completed":false}' \
  -w "\nStatus: %{http_code}\n"

Response:
{"id":1,"title":"Buy milk","completed":false,"createdAt":"..."}
Status: 201

Verified:
✅ Status code 201 (Created)
✅ Response body contains id
✅ Response body contains title
✅ Content-Type is application/json
```

### You MUST Test All CRUD Operations

Test these in order:

1. **Create (POST)** - Add a todo
2. **Read All (GET collection)** - List todos
3. **Read One (GET single)** - Get specific todo
4. **Update (PUT)** - Modify todo
5. **Delete (DELETE)** - Remove todo
6. **Verify Delete (GET)** - Confirm 404
7. **Error Case (POST invalid)** - Test validation

### You MUST Verify Status Codes

Check actual HTTP status codes:

| Operation | Expected Status |
|-----------|-----------------|
| POST (create) | 201 Created |
| GET (read) | 200 OK |
| PUT (update) | 200 OK |
| DELETE | 204 No Content |
| GET (not found) | 404 Not Found |
| POST (invalid) | 400 Bad Request |

### You MUST Validate Response Bodies

Check response structure:

**POST response should have:**
- `id` field (number)
- `title` field (string)
- `completed` field (boolean)
- `createdAt` field (ISO timestamp)

**GET collection should return:**
- Array of todos
- Each todo has id, title, completed

**404 response should have:**
- `error` field with message

---

## Testing Workflow

### Step 1: Build the API

Create Express.js server with all endpoints per CONSTITUTION.md.

### Step 2: Start via Telepipe

```bash
cd todo-api
npm install
telepipe redirect --id api -- node server.js
```

Wait 2 seconds for server to start.

### Step 3: Test Each Endpoint

#### Test 1: POST /api/todos (Create)

```bash
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"Buy milk","completed":false}' \
  -w "\nStatus: %{http_code}\n" \
  -s
```

**Verify:**
- Status: 201
- Body has id, title, completed, createdAt
- title matches "Buy milk"

---

#### Test 2: GET /api/todos (List)

```bash
curl -X GET http://localhost:3000/api/todos \
  -w "\nStatus: %{http_code}\n" \
  -s
```

**Verify:**
- Status: 200
- Body is array
- Array has 1 item
- Item matches created todo

---

#### Test 3: GET /api/todos/1 (Read One)

```bash
curl -X GET http://localhost:3000/api/todos/1 \
  -w "\nStatus: %{http_code}\n" \
  -s
```

**Verify:**
- Status: 200
- Body is object
- id = 1
- title = "Buy milk"

---

#### Test 4: PUT /api/todos/1 (Update)

```bash
curl -X PUT http://localhost:3000/api/todos/1 \
  -H "Content-Type: application/json" \
  -d '{"title":"Buy milk","completed":true}' \
  -w "\nStatus: %{http_code}\n" \
  -s
```

**Verify:**
- Status: 200
- Body has updatedAt
- completed = true

---

#### Test 5: DELETE /api/todos/1

```bash
curl -X DELETE http://localhost:3000/api/todos/1 \
  -w "\nStatus: %{http_code}\n" \
  -s
```

**Verify:**
- Status: 204
- No body (empty response)

---

#### Test 6: GET /api/todos/1 (Verify Deleted)

```bash
curl -X GET http://localhost:3000/api/todos/1 \
  -w "\nStatus: %{http_code}\n" \
  -s
```

**Verify:**
- Status: 404
- Body has error message

---

#### Test 7: POST /api/todos (Invalid)

```bash
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"completed":false}' \
  -w "\nStatus: %{http_code}\n" \
  -s
```

**Verify:**
- Status: 400
- Body has error: "Title is required"

---

## Your Output: Test Report

After testing, produce a report like this:

```markdown
# API Test Report

## Summary
✅ 7/7 tests passed
✅ All CRUD operations functional
✅ Error handling verified

## Test Results

### Test 1: Create Todo (POST /api/todos)

**Request:**
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"Buy milk","completed":false}'

**Response:**
{"id":1,"title":"Buy milk","completed":false,"createdAt":"2024-01-15T10:30:00.123Z"}
Status: 201

**Verification:**
✅ Status code: 201 Created
✅ Response has id field
✅ Response has title field
✅ Response has completed field
✅ Response has createdAt timestamp

---

### Test 2: List Todos (GET /api/todos)

**Request:**
curl -X GET http://localhost:3000/api/todos

**Response:**
[{"id":1,"title":"Buy milk","completed":false,"createdAt":"2024-01-15T10:30:00.123Z"}]
Status: 200

**Verification:**
✅ Status code: 200 OK
✅ Response is array
✅ Array contains 1 item
✅ Item matches created todo

---

[Continue for all 7 tests...]

## Conclusion

All endpoints tested and verified with actual HTTP requests.
Complete CRUD functionality confirmed.
Error handling working correctly.

API is production-ready!
```

---

## curl Flags Reference

| Flag | Purpose |
|------|---------|
| `-X METHOD` | HTTP method (POST, GET, PUT, DELETE) |
| `-H "Header"` | Add header |
| `-d 'data'` | Request body (JSON) |
| `-w "\nStatus: %{http_code}\n"` | Print status code |
| `-s` | Silent mode (hide progress) |

---

## Remember

- You're not assuming the API works. You're **testing** it.
- Every endpoint gets a real HTTP request.
- Every response is **verified** with actual output.
- The endoskeleton is real. **Use it.**

**Don't hope it works. PROVE it works.**

---

## Success Criteria

Your testing is successful when:

1. ✅ Server started via Telepipe
2. ✅ All 7 endpoints tested with curl
3. ✅ All status codes verified (201, 200, 204, 404, 400)
4. ✅ All response bodies validated
5. ✅ Error cases tested
6. ✅ Complete test report produced

---

## Anti-Patterns to Avoid

**DON'T:**
- Start the server without Telepipe
- Skip testing any endpoint
- Assume a status code without checking
- Report success without actual HTTP output
- Skip error case testing

**DO:**
- Use Telepipe to manage the server
- Test every single endpoint
- Verify actual status codes
- Include real HTTP responses in report
- Test both happy path and error cases

---

**Go forth and test with proof.**
