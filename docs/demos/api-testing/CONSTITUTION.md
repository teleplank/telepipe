# Constitution: REST API Testing Demo
## Technical Specification for the Todo API

---

## Purpose

This document defines the exact API specification, endpoints, data model, and expected responses for the REST API Testing demo. It serves as the source of truth for both implementation and testing.

---

## API Specification

**Name:** Todo REST API
**Framework:** Express.js (Node.js)
**Purpose:** Demonstrate AI HTTP testing
**Port:** 3000 (configurable via PORT env)

---

## Endpoints

### POST /api/todos

**Purpose:** Create a new todo

**Request:**
```json
{
    "title": "Buy milk",
    "completed": false
}
```

**Response (201 Created):**
```json
{
    "id": 1,
    "title": "Buy milk",
    "completed": false,
    "createdAt": "2024-01-15T10:30:00.123Z"
}
```

**Validation:**
- `title` is required (400 if missing)
- `completed` defaults to false

---

### GET /api/todos

**Purpose:** List all todos

**Request:** (none)

**Response (200 OK):**
```json
[
    {
        "id": 1,
        "title": "Buy milk",
        "completed": false,
        "createdAt": "2024-01-15T10:30:00.123Z"
    },
    {
        "id": 2,
        "title": "Walk dog",
        "completed": true,
        "createdAt": "2024-01-15T10:31:00.456Z"
    }
]
```

---

### GET /api/todos/:id

**Purpose:** Get a single todo

**Request:** (id in URL)

**Response (200 OK):**
```json
{
    "id": 1,
    "title": "Buy milk",
    "completed": false,
    "createdAt": "2024-01-15T10:30:00.123Z"
}
```

**Response (404 Not Found):**
```json
{
    "error": "Todo not found"
}
```

---

### PUT /api/todos/:id

**Purpose:** Update a todo

**Request:**
```json
{
    "title": "Buy milk",
    "completed": true
}
```

**Response (200 OK):**
```json
{
    "id": 1,
    "title": "Buy milk",
    "completed": true,
    "createdAt": "2024-01-15T10:30:00.123Z",
    "updatedAt": "2024-01-15T10:35:00.789Z"
}
```

**Response (404 Not Found):**
```json
{
    "error": "Todo not found"
}
```

**Validation:**
- At least one field must be provided (400 if empty)

---

### DELETE /api/todos/:id

**Purpose:** Delete a todo

**Request:** (id in URL)

**Response (204 No Content):**
(empty body)

**Response (404 Not Found):**
```json
{
    "error": "Todo not found"
}
```

---

## Data Model

```typescript
interface Todo {
    id: number;           // Auto-increment
    title: string;        // Required
    completed: boolean;   // Default: false
    createdAt: string;    // ISO 8601 timestamp
    updatedAt?: string;   // ISO 8601 timestamp (only on update)
}
```

**Storage:** In-memory array (for demo purposes)

```javascript
let todos = [];
let nextId = 1;
```

---

## HTTP Status Codes

| Code | Meaning | When Used |
|------|---------|-----------|
| 200 | OK | Successful GET/PUT |
| 201 | Created | Successful POST |
| 204 | No Content | Successful DELETE |
| 400 | Bad Request | Validation error |
| 404 | Not Found | Todo doesn't exist |
| 500 | Server Error | Unexpected error |

---

## Test Scenarios

### Happy Path (CRUD)

| Test | Method | Endpoint | Expected Status | Expected Body |
|------|--------|----------|-----------------|---------------|
| Create | POST | /api/todos | 201 | Todo with ID |
| List | GET | /api/todos | 200 | Array of todos |
| Read | GET | /api/todos/1 | 200 | Single todo |
| Update | PUT | /api/todos/1 | 200 | Updated todo |
| Delete | DELETE | /api/todos/1 | 204 | (empty) |

### Error Cases

| Test | Method | Endpoint | Data | Expected Status | Expected Body |
|------|--------|----------|------|-----------------|---------------|
| Missing title | POST | /api/todos | `{"completed":false}` | 400 | Error message |
| Not found (read) | GET | /api/todos/999 | - | 404 | Error message |
| Not found (update) | PUT | /api/todos/999 | `{...}` | 404 | Error message |
| Not found (delete) | DELETE | /api/todos/999 | - | 404 | Error message |

---

## Implementation Template

### Express.js Structure

```javascript
const express = require('express');
const app = express();

app.use(express.json());

let todos = [];
let nextId = 1;

// Create todo
app.post('/api/todos', (req, res) => {
    const { title, completed = false } = req.body;

    if (!title) {
        return res.status(400).json({ error: 'Title is required' });
    }

    const todo = {
        id: nextId++,
        title,
        completed,
        createdAt: new Date().toISOString()
    };

    todos.push(todo);
    res.status(201).json(todo);
});

// List todos
app.get('/api/todos', (req, res) => {
    res.json(todos);
});

// Get single todo
app.get('/api/todos/:id', (req, res) => {
    const id = parseInt(req.params.id);
    const todo = todos.find(t => t.id === id);

    if (!todo) {
        return res.status(404).json({ error: 'Todo not found' });
    }

    res.json(todo);
});

// Update todo
app.put('/api/todos/:id', (req, res) => {
    const id = parseInt(req.params.id);
    const todo = todos.find(t => t.id === id);

    if (!todo) {
        return res.status(404).json({ error: 'Todo not found' });
    }

    const { title, completed } = req.body;

    if (title !== undefined) todo.title = title;
    if (completed !== undefined) todo.completed = completed;
    todo.updatedAt = new Date().toISOString();

    res.json(todo);
});

// Delete todo
app.delete('/api/todos/:id', (req, res) => {
    const id = parseInt(req.params.id);
    const index = todos.findIndex(t => t.id === id);

    if (index === -1) {
        return res.status(404).json({ error: 'Todo not found' });
    }

    todos.splice(index, 1);
    res.status(204).send();
});

// Error handling
app.use((err, req, res, next) => {
    console.error(err.stack);
    res.status(500).json({ error: 'Internal server error' });
});

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
    console.log(`Server running on http://localhost:${PORT}`);
});
```

### package.json

```json
{
    "name": "todo-api",
    "version": "1.0.0",
    "description": "Todo REST API for Telepipe demo",
    "main": "server.js",
    "scripts": {
        "start": "node server.js"
    },
    "dependencies": {
        "express": "^4.18.2"
    }
}
```

---

## curl Testing Commands

### Create Todo
```bash
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"Buy milk","completed":false}' \
  -w "\nStatus: %{http_code}\n" \
  -s
```

### List Todos
```bash
curl -X GET http://localhost:3000/api/todos \
  -w "\nStatus: %{http_code}\n" \
  -s
```

### Get Single Todo
```bash
curl -X GET http://localhost:3000/api/todos/1 \
  -w "\nStatus: %{http_code}\n" \
  -s
```

### Update Todo
```bash
curl -X PUT http://localhost:3000/api/todos/1 \
  -H "Content-Type: application/json" \
  -d '{"title":"Buy milk","completed":true}' \
  -w "\nStatus: %{http_code}\n" \
  -s
```

### Delete Todo
```bash
curl -X DELETE http://localhost:3000/api/todos/1 \
  -w "\nStatus: %{http_code}\n" \
  -s
```

### Test Missing Title (400)
```bash
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"completed":false}' \
  -w "\nStatus: %{http_code}\n" \
  -s
```

### Test Not Found (404)
```bash
curl -X GET http://localhost:3000/api/todos/999 \
  -w "\nStatus: %{http_code}\n" \
  -s
```

---

## Telepipe Commands

```bash
# Start server via Telepipe
telepipe redirect --id api -- node server.js

# Check server status
telepipe info --id api

# Stop server
telepipe stop --id api
```

---

## Success Criteria

1. ✅ All endpoints implemented
2. ✅ All status codes correct
3. ✅ All response bodies match spec
4. ✅ Validation errors handled
5. ✅ 404 errors for missing resources
6. ✅ All tests pass with curl

---

## curl Flags Reference

| Flag | Purpose |
|------|---------|
| `-X METHOD` | HTTP method (POST, GET, PUT, DELETE) |
| `-H "Header"` | Add header (e.g., Content-Type) |
| `-d 'data'` | Request body (JSON) |
| `-w "\nStatus: %{http_code}\n"` | Print status code |
| `-s` | Silent mode (hide progress) |

---

*This constitution defines the Todo REST API specification.*
