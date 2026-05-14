# API Reference

> **Base URL (local):** `http://localhost:8787`  
> **Base URL (production):** `https://chat-app-backend.<your-subdomain>.workers.dev`  
> **Version:** v0.2.0  
> **CLI:** Use `cargo run -- <command>` from `cli/` — see [cli/README.md](../cli/README.md)

---

## Table of Contents

- [Response Formats](#response-formats)
- [HTTP Status Codes](#http-status-codes)
- [Health Check](#health-check)
- [CORS Preflight](#cors-preflight)
- [Agents](#agents)
  - [Create Agent](#create-agent)
  - [List Agents](#list-agents)
  - [Get Agent](#get-agent)
  - [Update Agent](#update-agent)
  - [Delete Agent](#delete-agent)
- [Owners](#owners)
  - [Create Owner](#create-owner)
  - [List Owners](#list-owners)
  - [Get Owner](#get-owner)
  - [Update Owner](#update-owner)
  - [Delete Owner](#delete-owner)
- [Chats](#chats)
  - [Create Chat](#create-chat)
  - [List Chats](#list-chats)
  - [Get Chat with Messages](#get-chat-with-messages)
  - [Update Chat](#update-chat)
  - [Delete Chat](#delete-chat)
- [Messages](#messages)
  - [Send Message](#send-message)
  - [Get Messages](#get-messages)

---

## Response Formats

### Success — Single Resource
```json
{
  "success": true,
  "data": { ... }
}
```

### Success — List of Resources
```json
{
  "success": true,
  "data": [ ... ],
  "total": 42
}
```

### Success — Deletion
```json
{
  "success": true,
  "data": { "deleted": true }
}
```

### Error (v0.2.0+)
```json
{
  "success": false,
  "data": null,
  "error": "Human-readable error description",
  "code": "ERR_VALIDATION"
}
```

### Health Check (special format)
```json
{
  "status": "ok",
  "service": "chat-app-backend",
  "version": "0.2.0"
}
```

---

## HTTP Status Codes

| Code | Description |
|---|---|
| `200` | Success |
| `201` | Resource created successfully |
| `204` | No content (CORS preflight) |
| `400` | Bad request (invalid JSON, validation failure) |
| `404` | Resource not found |
| `500` | Internal server error (database error, unexpected error) |

---

## Health Check

Check if the API is running.

```
GET /api/health
```

**Response `200`:**
```json
{
  "status": "ok",
  "service": "chat-app-backend",
  "version": "0.2.0"
}
```

**Example:**
```bash
curl http://localhost:8787/api/health
```

---

## CORS Preflight

Handle browser CORS preflight requests.

```
OPTIONS /*
```

**Response `204`:** No body, with CORS headers.

---

## Agents

### Create Agent

```
POST /api/agents
```

**Request Body:**

| Field | Type | Required | Description | Validation |
|---|---|---|---|---|
| `name` | string | ✅ | Display name for the agent | 1–200 characters |
| `description` | string | ❌ | Optional description | ≤2,000 characters |
| `owner_id` | string | ❌ | UUID of the owner (nullable) | Must be valid UUID v4 if provided |

**Example:**
```bash
curl -X POST http://localhost:8787/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Code Assistant",
    "description": "Helps with code generation and debugging"
  }'
```

**Response `201`:**
```json
{
  "success": true,
  "data": {
    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "name": "Code Assistant",
    "description": "Helps with code generation and debugging",
    "owner_id": null,
    "created_at": "2025-01-15 10:30:00",
    "updated_at": "2025-01-15 10:30:00"
  }
}
```

**Response `400` (validation failure):**
```json
{
  "success": false,
  "data": null,
  "error": "name must not be empty",
  "code": "ERR_VALIDATION"
}
```

---

### List Agents

```
GET /api/agents
```

Returns all agents ordered by creation date (newest first).

**Response `200`:**
```json
{
  "success": true,
  "data": [
    {
      "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "name": "Code Assistant",
      "description": "Helps with code generation and debugging",
      "owner_id": null,
      "created_at": "2025-01-15 10:30:00",
      "updated_at": "2025-01-15 10:30:00"
    }
  ],
  "total": 1
}
```

**Example:**
```bash
curl http://localhost:8787/api/agents
```

---

### Get Agent

```
GET /api/agents/:id
```

**Path Parameters:**

| Parameter | Description |
|---|---|
| `id` | The UUID of the agent |

**Response `200`:** Returns the agent object (same format as Create Agent response).

**Response `404`:**
```json
{
  "success": false,
  "data": null,
  "error": "Agent 'nonexistent-id' not found",
  "code": "ERR_NOT_FOUND"
}
```

**Example:**
```bash
curl http://localhost:8787/api/agents/a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

---

### Update Agent

```
PUT /api/agents/:id
```

Only the fields you include will be updated. Omitted fields keep their current values.

**Path Parameters:**

| Parameter | Description |
|---|---|
| `id` | The UUID of the agent |

**Request Body:**

| Field | Type | Required | Description | Validation |
|---|---|---|---|---|
| `name` | string | ❌ | New display name | 1–200 characters if provided |
| `description` | string | ❌ | New description | ≤2,000 characters if provided |
| `owner_id` | string (nullable) | ❌ | New owner UUID, or `null` to unlink | Must be valid UUID v4 if provided |

**Example:**
```bash
curl -X PUT http://localhost:8787/api/agents/a1b2c3d4-e5f6-7890-abcd-ef1234567890 \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Specializes in Rust and WebAssembly development"
  }'
```

**Response `200`:** Returns the updated agent.

---

### Delete Agent

```
DELETE /api/agents/:id
```

**Notes:**
- Deleting an agent **cascade deletes** all their chats and messages.
- If the agent is referenced as `owner_id` on other records, not applicable.

**Example:**
```bash
curl -X DELETE http://localhost:8787/api/agents/a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

**Response `200`:**
```json
{
  "success": true,
  "data": { "deleted": true }
}
```

**Response `404`:**
```json
{
  "success": false,
  "data": null,
  "error": "Agent 'nonexistent-id' not found",
  "code": "ERR_NOT_FOUND"
}
```

---

## Owners

### Create Owner

```
POST /api/owners
```

**Request Body:**

| Field | Type | Required | Description | Validation |
|---|---|---|---|---|
| `name` | string | ✅ | Display name | 1–200 characters |
| `email` | string | ✅ | Email address (must be unique) | 1–320 characters, valid email format |

**Validation details:**
- Email must contain exactly one `@` character
- Domain part must have at least one `.` (e.g., `user@domain.com`)
- Local part and domain part must be non-empty

**Example:**
```bash
curl -X POST http://localhost:8787/api/owners \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Alice",
    "email": "alice@example.com"
  }'
```

**Response `201`:** Returns the created owner with the new UUID.

**Response `400` (duplicate email):**
```json
{
  "success": false,
  "data": null,
  "error": "An owner with this email already exists",
  "code": "ERR_BAD_REQUEST"
}
```

**Response `400` (validation failure):**
```json
{
  "success": false,
  "data": null,
  "error": "email must be a valid email address",
  "code": "ERR_VALIDATION"
}
```

---

### List Owners

```
GET /api/owners
```

Returns all owners ordered by creation date (newest first).

**Example:**
```bash
curl http://localhost:8787/api/owners
```

---

### Get Owner

```
GET /api/owners/:id
```

**Example:**
```bash
curl http://localhost:8787/api/owners/o1p2q3r4-s5t6-7890-uvwx-yz1234567890
```

---

### Update Owner

```
PUT /api/owners/:id
```

**Request Body:**

| Field | Type | Required | Description | Validation |
|---|---|---|---|---|
| `name` | string | ❌ | New display name | 1–200 characters if provided |
| `email` | string | ❌ | New email address | Valid email format if provided |

**Example:**
```bash
curl -X PUT http://localhost:8787/api/owners/o1p2q3r4-s5t6-7890-uvwx-yz1234567890 \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Alice Johnson"
  }'
```

---

### Delete Owner

```
DELETE /api/owners/:id
```

**Notes:**
- Sets `owner_id` to `NULL` on all their agents (`ON DELETE SET NULL`).
- **Cascade deletes** all chats where they are a participant.

**Example:**
```bash
curl -X DELETE http://localhost:8787/api/owners/o1p2q3r4-s5t6-7890-uvwx-yz1234567890
```

---

## Chats

### Create Chat

```
POST /api/chats
```

Creates a new conversation between an agent and an owner.

**Request Body:**

| Field | Type | Required | Description | Validation |
|---|---|---|---|---|
| `agent_id` | string | ✅ | UUID of the participating agent | Must be valid UUID v4 |
| `owner_id` | string | ✅ | UUID of the participating owner | Must be valid UUID v4 |
| `title` | string | ❌ | Chat title (defaults to "New Chat") | ≤500 characters if provided |

**Example:**
```bash
curl -X POST http://localhost:8787/api/chats \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "owner_id": "o1p2q3r4-s5t6-7890-uvwx-yz1234567890",
    "title": "Help with Rust project"
  }'
```

**Response `201`:**
```json
{
  "success": true,
  "data": {
    "id": "c1d2e3f4-a5b6-7890-cdef-012345678901",
    "title": "Help with Rust project",
    "agent_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "owner_id": "o1p2q3r4-s5t6-7890-uvwx-yz1234567890",
    "created_at": "2025-01-15 10:30:00",
    "updated_at": "2025-01-15 10:30:00"
  }
}
```

---

### List Chats

```
GET /api/chats
```

Returns all chats ordered by last activity (`updated_at` descending).

**Example:**
```bash
curl http://localhost:8787/api/chats
```

---

### Get Chat with Messages

```
GET /api/chats/:id
```

Returns the chat metadata along with all its messages (ordered by creation time, ascending).

**Response `200`:**
```json
{
  "success": true,
  "data": {
    "id": "c1d2e3f4-a5b6-7890-cdef-012345678901",
    "title": "Help with Rust project",
    "agent_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "owner_id": "o1p2q3r4-s5t6-7890-uvwx-yz1234567890",
    "created_at": "2025-01-15 10:30:00",
    "updated_at": "2025-01-15 10:35:00",
    "messages": [
      {
        "id": "m1n2o3p4-q5r6-7890-stuv-wx1234567890",
        "chat_id": "c1d2e3f4-a5b6-7890-cdef-012345678901",
        "sender_type": "owner",
        "sender_id": "o1p2q3r4-s5t6-7890-uvwx-yz1234567890",
        "content": "Hi! Can you help me with my Rust project?",
        "created_at": "2025-01-15 10:31:00"
      },
      {
        "id": "m2n3o4p5-q6r7-8901-stuv-wx2345678901",
        "chat_id": "c1d2e3f4-a5b6-7890-cdef-012345678901",
        "sender_type": "agent",
        "sender_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "content": "Of course! What are you working on?",
        "created_at": "2025-01-15 10:32:00"
      }
    ]
  }
}
```

**Example:**
```bash
curl http://localhost:8787/api/chats/c1d2e3f4-a5b6-7890-cdef-012345678901
```

---

### Update Chat

```
PUT /api/chats/:id
```

**Request Body:**

| Field | Type | Required | Description | Validation |
|---|---|---|---|---|
| `title` | string | ❌ | New chat title | ≤500 characters if provided |

**Example:**
```bash
curl -X PUT http://localhost:8787/api/chats/c1d2e3f4-a5b6-7890-cdef-012345678901 \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Updated: Rust project discussion"
  }'
```

**Response `200`:** Returns the full chat with messages (same format as GET).

---

### Delete Chat

```
DELETE /api/chats/:id
```

**Notes:**
- Deleting a chat **cascade deletes** all its messages.

**Example:**
```bash
curl -X DELETE http://localhost:8787/api/chats/c1d2e3f4-a5b6-7890-cdef-012345678901
```

---

## Messages

### Send Message

```
POST /api/chats/:id/messages
```

Sends a new message in the specified chat. The chat's `updated_at` timestamp is automatically updated.

**Path Parameters:**

| Parameter | Description |
|---|---|
| `id` | The UUID of the chat |

**Request Body:**

| Field | Type | Required | Description | Validation |
|---|---|---|---|---|
| `sender_type` | string | ✅ | Must be `"agent"` or `"owner"` | Must match exactly |
| `sender_id` | string | ✅ | UUID of the sender (agent or owner ID) | Must be valid UUID v4 |
| `content` | string | ✅ | The message text | 1–10,000 characters |

**Example:**
```bash
curl -X POST http://localhost:8787/api/chats/c1d2e3f4-a5b6-7890-cdef-012345678901/messages \
  -H "Content-Type: application/json" \
  -d '{
    "sender_type": "owner",
    "sender_id": "o1p2q3r4-s5t6-7890-uvwx-yz1234567890",
    "content": "I'\''m building a web scraper in Rust!"
  }'
```

**Response `201`:**
```json
{
  "success": true,
  "data": {
    "id": "m3n4o5p6-q7r8-9012-stuv-wx3456789012",
    "chat_id": "c1d2e3f4-a5b6-7890-cdef-012345678901",
    "sender_type": "agent",
    "sender_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "content": "Sure! Let me take a look at your code.",
    "created_at": "2025-01-15 10:33:00"
  }
}
```

**Response `404`:** If the chat does not exist.

---

### Get Messages

```
GET /api/chats/:id/messages
```

Returns all messages for a chat, ordered by creation time (ascending — oldest first).

**Response `200`:**
```json
{
  "success": true,
  "data": [
    {
      "id": "m1n2o3p4-q5r6-7890-stuv-wx1234567890",
      "chat_id": "c1d2e3f4-a5b6-7890-cdef-012345678901",
      "sender_type": "owner",
      "sender_id": "o1p2q3r4-s5t6-7890-uvwx-yz1234567890",
      "content": "Hi! Can you help me with my Rust project?",
      "created_at": "2025-01-15 10:31:00"
    },
    {
      "id": "m2n3o4p5-q6r7-8901-stuv-wx2345678901",
      "chat_id": "c1d2e3f4-a5b6-7890-cdef-012345678901",
      "sender_type": "agent",
      "sender_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "content": "Of course! What are you working on?",
      "created_at": "2025-01-15 10:32:00"
    }
  ],
  "total": 2
}
```

**Example:**
```bash
curl http://localhost:8787/api/chats/c1d2e3f4-a5b6-7890-cdef-012345678901/messages
```

**Response `404`:** If the chat does not exist.
