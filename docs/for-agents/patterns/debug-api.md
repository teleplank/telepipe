# Pattern: Debug Your API
## From "The Endpoint Should Work" to "Here's the Response"

---

## The Scenario

You wrote an API endpoint. The logic looks right, the route is defined, the handler is implemented.

But does it actually return the right data? Handle errors correctly? Respond quickly?

**Don't guess. Call it.**

---

## Prerequisites

```bash
# Your API server running
npm run dev  # or cargo run, python app.py, etc.

# For request inspection, optional:
# Chrome with debugging enabled for network monitoring
google-chrome --remote-debugging-port=9222

# Get WebSocket URL and connect via websocat
WS_URL=$(curl -s http://127.0.0.1:9222/json/version | grep -o 'ws://[^"]*')
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

---

## Pattern 1: Direct API Testing with nc/curl

### Start a client through Telepipe

```bash
telepipe redirect --id client -- nc localhost 3000
```

### Send HTTP Request

```http
GET /api/users HTTP/1.1
Host: localhost:3000
Accept: application/json

```

(Note: blank line at end is required for HTTP)

### Verify Response

```http
HTTP/1.1 200 OK
Content-Type: application/json
Content-Length: 245

{"users":[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]}
```

**You just called your API and got a real response.**

---

## Pattern 2: Test CRUD Operations

### CREATE
```http
POST /api/users HTTP/1.1
Host: localhost:3000
Content-Type: application/json
Content-Length: 42

{"name":"Charlie","email":"charlie@x.com"}
```

Verify: Status 201, response contains new user with ID.

### READ
```http
GET /api/users/3 HTTP/1.1
Host: localhost:3000

```

Verify: Status 200, response matches what you created.

### UPDATE
```http
PUT /api/users/3 HTTP/1.1
Host: localhost:3000
Content-Type: application/json
Content-Length: 23

{"name":"Charles"}
```

Verify: Status 200, name changed.

### DELETE
```http
DELETE /api/users/3 HTTP/1.1
Host: localhost:3000

```

Verify: Status 204 (no content) or 200.

### Confirm Deletion
```http
GET /api/users/3 HTTP/1.1
Host: localhost:3000

```

Verify: Status 404.

---

## Pattern 3: Error Handling Verification

### Test 400 Bad Request
```http
POST /api/users HTTP/1.1
Host: localhost:3000
Content-Type: application/json
Content-Length: 2

{}
```

Expect:
```http
HTTP/1.1 400 Bad Request
Content-Type: application/json

{"error":"name is required","code":"VALIDATION_ERROR"}
```

### Test 401 Unauthorized
```http
GET /api/admin/users HTTP/1.1
Host: localhost:3000

```

Expect:
```http
HTTP/1.1 401 Unauthorized
Content-Type: application/json

{"error":"Authentication required"}
```

### Test 404 Not Found
```http
GET /api/users/99999 HTTP/1.1
Host: localhost:3000

```

Expect:
```http
HTTP/1.1 404 Not Found
Content-Type: application/json

{"error":"User not found"}
```

### Test 500 Internal Error
(Trigger a bug intentionally to verify error handling)

---

## Pattern 4: Authentication Testing

### Test Without Token
```http
GET /api/protected HTTP/1.1
Host: localhost:3000

```

Expect: 401 Unauthorized

### Test With Invalid Token
```http
GET /api/protected HTTP/1.1
Host: localhost:3000
Authorization: Bearer invalid_token_here

```

Expect: 401 or 403

### Test With Valid Token
```http
GET /api/protected HTTP/1.1
Host: localhost:3000
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...

```

Expect: 200 with data

---

## Pattern 5: Monitor via CDP Network

Use Chrome's network monitoring to see all requests:

```javascript
// Enable network monitoring
{"id": 1, "method": "Network.enable"}

// Navigate to your frontend that calls the API
{"id": 2, "method": "Page.navigate", "params": {"url": "http://localhost:3000"}}

// Watch for network events:
// Network.requestWillBeSent - see the request
// Network.responseReceived - see status, headers
// Network.loadingFinished - request complete

// Get response body
{"id": 3, "method": "Network.getResponseBody", "params": {"requestId": "..."}}
```

This shows you:
- What requests your frontend makes
- Request headers, body
- Response status, headers, timing
- Response body

---

## Pattern 6: Performance Testing

### Time Your Endpoint

```bash
# Simple timing
time curl http://localhost:3000/api/users
```

### Load Testing (if ab is available)
```bash
# 100 requests, 10 concurrent
ab -n 100 -c 10 http://localhost:3000/api/users
```

### Manual Timing via CDP

```javascript
// Record start time
{"method": "Runtime.evaluate", "params": {"expression": "performance.now()"}}

// Make request (via frontend navigation or fetch)
{"method": "Runtime.evaluate", "params": {
  "expression": "fetch('/api/users').then(r => r.json())",
  "awaitPromise": true
}}

// Record end time
{"method": "Runtime.evaluate", "params": {"expression": "performance.now()"}}
```

---

## Pattern 7: Request/Response Validation

### Verify Response Schema

After getting response, validate structure:

```javascript
// In browser context via CDP
{"method": "Runtime.evaluate", "params": {
  "expression": `
    fetch('/api/users')
      .then(r => r.json())
      .then(data => {
        const valid = Array.isArray(data.users) &&
                      data.users.every(u => u.id && u.name && u.email);
        return {valid, sampleUser: data.users[0]};
      })
  `,
  "awaitPromise": true,
  "returnByValue": true
}}
```

### Verify Headers

```javascript
{"method": "Runtime.evaluate", "params": {
  "expression": `
    fetch('/api/users')
      .then(r => ({
        status: r.status,
        contentType: r.headers.get('content-type'),
        cacheControl: r.headers.get('cache-control')
      }))
  `,
  "awaitPromise": true,
  "returnByValue": true
}}
```

---

## Complete Debug Session

```bash
# 1. Start your API
npm run dev

# 2. Connect Telepipe client
telepipe redirect --id client -- nc localhost 3000

# 3. Test happy path
GET /api/users HTTP/1.1
...
# Verify: 200, correct data

# 4. Test creation
POST /api/users HTTP/1.1
Content-Type: application/json
{"name":"Test","email":"test@test.com"}
# Verify: 201, has ID

# 5. Test validation error
POST /api/users HTTP/1.1
Content-Type: application/json
{}
# Verify: 400, error message

# 6. Test not found
GET /api/users/99999 HTTP/1.1
# Verify: 404

# 7. Test auth (if applicable)
GET /api/protected HTTP/1.1
# Verify: 401

GET /api/protected HTTP/1.1
Authorization: Bearer valid_token
# Verify: 200
```

---

## What You Can Now Say

**Before:**
> "I implemented a REST API for users. The routes are defined, the handlers look correct. It should work."

**After:**
> "I implemented and tested a REST API for users. Here's what I verified:
> 
> | Endpoint | Method | Test | Status | Result |
> |----------|--------|------|--------|--------|
> | /api/users | GET | List users | 200 ✅ | Returns array of 3 users |
> | /api/users | POST | Create user | 201 ✅ | Returns new user with id=4 |
> | /api/users | POST | Missing name | 400 ✅ | Error: 'name is required' |
> | /api/users/4 | GET | Get created | 200 ✅ | Returns user id=4 |
> | /api/users/999 | GET | Not found | 404 ✅ | Error: 'User not found' |
> | /api/users/4 | DELETE | Delete | 204 ✅ | No content |
> | /api/users/4 | GET | Confirm deleted | 404 ✅ | User gone |
> 
> All endpoints behave correctly."

---

## The Debugging Mindset

APIs are contracts. They promise:
- This route exists
- It accepts this input
- It returns this output
- It handles these errors

With Telepipe, you don't just write the contract - you verify it.

**Call your API. Check the response. Prove it works.**
