# Chat App Backend

A serverless backend for a multi-agent chat application built with **Rust**, **Cloudflare Workers**, and **Cloudflare D1**. This API enables agents and their human owners to create and manage conversations through a RESTful interface.

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Tech Stack](#tech-stack)
- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Project Structure](#project-structure)
- [Configuration](#configuration)
- [Database Schema](#database-schema)
  - [Entity Relationship Diagram](#entity-relationship-diagram)
  - [Tables](#tables)
  - [Indexes](#indexes)
- [API Reference](#api-reference)
  - [Response Format](#response-format)
  - [Health Check](#health-check)
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
- [Development](#development)
  - [Local Development](#local-development)
  - [Running Migrations](#running-migrations)
  - [Building](#building)
- [Deployment](#deployment)
- [Testing with curl](#testing-with-curl)
- [CORS Configuration](#cors-configuration)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)
- [License](#license)

---

## Architecture Overview

```
                        ┌─────────────────────────────────┐
                        │        Client Application        │
                        │   (Web, Mobile, CLI, Agent)      │
                        └──────────┬──────────────────────┘
                                   │ HTTP REST API
                                   ▼
                        ┌──────────────────────────────────┐
                        │    Cloudflare Workers (Rust)     │
                        │  ┌────────────────────────────┐  │
                        │  │   worker crate runtime     │  │
                        │  │   HTTP Router              │  │
                        │  │   JSON Serialization        │  │
                        │  │   (serde / serde_json)     │  │
                        │  └──────────┬─────────────────┘  │
                        │             │                     │
                        │  ┌──────────▼─────────────────┐  │
                        │  │   D1 Database Operations   │  │
                        │  │   (Prepared Statements)    │  │
                        │  └──────────┬─────────────────┘  │
                        └─────────────┼────────────────────┘
                                      │
                         ┌────────────▼────────────┐
                         │  Cloudflare D1 (SQLite) │
                         │  - agents               │
                         │  - owners               │
                         │  - chats                │
                         │  - messages             │
                         └─────────────────────────┘
```

The backend runs as a **Cloudflare Worker** — a serverless function deployed to Cloudflare's global edge network. Requests are routed to the nearest data center, providing low-latency access worldwide.

**Data flow:**
1. A client sends an HTTP request to the Worker's URL
2. The Worker's router matches the request to the appropriate handler
3. The handler parses the request body and calls the corresponding database function
4. The database function prepares and executes a SQL statement against D1
5. The result is serialized to JSON and returned to the client

## Tech Stack

| Technology | Purpose | Version |
|---|---|---|
| **Rust** | Programming language — performance, safety, WASM compilation | edition 2021 |
| **worker** | Cloudflare Workers Rust SDK — HTTP server, D1 bindings, routing | 0.4 |
| **serde / serde_json** | JSON serialization/deserialization for API request/response | 1.x |
| **uuid** | UUID v4 generation for primary keys | 1.x (with `js` feature) |
| **Cloudflare Workers** | Serverless execution environment at the edge | — |
| **Cloudflare D1** | Serverless SQLite database with global replication | — |
| **wrangler** | Cloudflare Workers CLI for dev, build, deploy | ^3.0 |

## Prerequisites

Before you begin, ensure you have the following installed:

- **Rust toolchain** — Install via [rustup](https://rustup.rs/)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **wasm-pack** — WebAssembly build tool for Rust
  ```bash
  cargo install wasm-pack
  ```

- **Node.js & npm** — Required for wrangler CLI (v18 or later)
  ```bash
  # Download from https://nodejs.org/ or use nvm
  ```

- **Wrangler CLI** — Cloudflare Workers CLI (installed via npm as a dev dependency)
  ```bash
  # Will be installed via npm install in the project
  ```

- **A Cloudflare account** — [Sign up free](https://dash.cloudflare.com/sign-up)

- **A Cloudflare D1 database** — Created via wrangler (see below)

## Quick Start

### 1. Clone and install dependencies

```bash
git clone <your-repo-url> chat-app-backend
cd chat-app-backend
npm install
```

### 2. Create the D1 database

```bash
npx wrangler d1 create chat-app-db
```

This will output a `database_id`. Copy it and paste it into `wrangler.toml`:

```toml
[[d1_databases]]
binding = "DB"
database_name = "chat-app-db"
database_id = "your-uuid-here"    # ← Replace this
```

### 3. Run the database migration

```bash
npx wrangler d1 migrations apply chat-app-db --local
```

### 4. Start the development server

```bash
npm run dev
```

This starts a local server at `http://localhost:8787`. You should see:

```
⎪  ⎬  ⎪  ⎪  ⎪  ⎪  ⎪  ⎪  ⎪  ⎪  ⎪  ⎪  ⎪  ⎪  ⎪  ⎪  ⎪  ⎪  ⎪  ⎪  ⎪  ⎪  ⎪
│  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │
│  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │
│  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │
│  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │
│  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │
│  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │
╰────────────────────────────────────────────────────────────────────╯
✨  Built successfully, starting dev server...
  🕵️  Ready: http://localhost:8787
```

### 5. Verify it works

```bash
curl http://localhost:8787/api/health
# {"status":"ok","service":"chat-app-backend","version":"0.1.0"}
```

## Project Structure

```
chat-app-backend/
│
├── Cargo.toml                # Rust project manifest and dependencies
├── wrangler.toml              # Cloudflare Workers configuration
├── package.json               # Node.js project manifest with wrangler scripts
├── .gitignore                 # Git ignore rules
├── README.md                  # This file
│
├── migrations/
│   └── 0001_initial.sql       # D1 schema migration (creates all tables + indexes)
│
└── src/
    ├── lib.rs                 # Worker entry point — HTTP router + route definitions
    ├── models.rs              # Data models (Agent, Owner, Chat, Message) + API types
    ├── db.rs                  # All D1 database operations (CRUD for all entities)
    └── handlers.rs            # HTTP request handlers — parsing, validation, CORS
```

### File Responsibilities

| File | Responsibility |
|---|---|
| **lib.rs** | Initializes the Worker, configures the HTTP router, maps URL paths to handlers, and adds the 404 fallback |
| **models.rs** | Defines all serializable/deserializable structs — database entities, request bodies, response wrappers |
| **db.rs** | Contains all D1 SQL queries — each function is an isolated database operation returning an HTTP `Response` |
| **handlers.rs** | Bridges HTTP and database layers — extracts path params, parses JSON bodies, applies CORS headers |

## Configuration

### wrangler.toml

```toml
name = "chat-app-backend"
main = "build/worker/shim.mjs"
compatibility_date = "2024-12-01"
compatibility_flags = ["nodejs_compat"]

[[d1_databases]]
binding = "DB"
database_name = "chat-app-db"
database_id = "your-database-id-here"

[[migrations]]
tag = "v1"
new_directory = "migrations"
```

| Field | Description |
|---|---|
| `name` | The name of your Worker (used for deployment) |
| `main` | Entry point for the compiled Worker — don't change this |
| `compatibility_date` | The Workers runtime compatibility date |
| `compatibility_flags` | Enables Node.js compatibility for wasm-bindgen |
| `binding` | The variable name used in Rust to access D1 via `env.d1("DB")` |
| `database_name` | The name of your D1 database (must match what you created) |
| `database_id` | The UUID of your D1 database (get this from `wrangler d1 create`) |
| `migrations` | Points wrangler to the `migrations/` directory |

### Cargo.toml

The project compiles to a `cdylib` (C dynamic library) which wasm-pack compiles to WebAssembly:

```toml
[lib]
crate-type = ["cdylib"]
```

The release profile is optimized for size, which is critical for Workers (the free plan has an 8 MB limit):

```toml
[profile.release]
lto = true          # Link-time optimization
opt-level = "s"     # Optimize for size
strip = true        # Strip debug symbols
codegen-units = 1   # Maximize optimization opportunities
```

## Database Schema

### Entity Relationship Diagram

```
┌────────────┐       ┌──────────────┐       ┌──────────────┐
│   owners   │       │   agents     │       │   chats      │
├────────────┤       ├──────────────┤       ├──────────────┤
│ id (PK)    │◄──────┤ owner_id (FK)│       │ id (PK)      │
│ name       │       │ id (PK)      │       │ title        │
│ email (UQ) │       │ name         │◄──────┤ agent_id (FK)│
│ created_at │       │ description  │       │ owner_id (FK)│
└────────────┘       │ created_at   │       │ created_at   │
                     │ updated_at   │       │ updated_at   │
                     └──────────────┘       └──────┬───────┘
                                                   │
                                          ┌────────▼───────┐
                                          │   messages     │
                                          ├────────────────┤
                                          │ id (PK)        │
                                          │ chat_id (FK)   │
                                          │ sender_type    │
                                          │ sender_id      │
                                          │ content        │
                                          │ created_at     │
                                          └────────────────┘
```

**Relationships:**
- An **Owner** can have many **Agents** (one-to-many via `agents.owner_id`)
- An **Agent** can have many **Chats** (one-to-many via `chats.agent_id`)
- An **Owner** can have many **Chats** (one-to-many via `chats.owner_id`)
- A **Chat** has many **Messages** (one-to-many via `messages.chat_id`)
- Deleting an **Owner** sets `agents.owner_id` to `NULL` (via `ON DELETE SET NULL`)
- Deleting an **Agent** or **Owner** cascades to delete their **Chats** (via `ON DELETE CASCADE`)
- Deleting a **Chat** cascades to delete its **Messages** (via `ON DELETE CASCADE`)

### Tables

#### `agents`

Stores AI agent profiles that can participate in conversations.

```sql
CREATE TABLE IF NOT EXISTS agents (
    id          TEXT PRIMARY KEY,              -- UUID v4, auto-generated
    name        TEXT NOT NULL,                 -- Display name of the agent
    description TEXT DEFAULT '',               -- Optional description of the agent
    owner_id    TEXT,                          -- FK to owners.id (nullable)
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (owner_id) REFERENCES owners(id) ON DELETE SET NULL
);
```

#### `owners`

Stores human owners who create and manage agents.

```sql
CREATE TABLE IF NOT EXISTS owners (
    id          TEXT PRIMARY KEY,              -- UUID v4, auto-generated
    name        TEXT NOT NULL,                 -- Display name of the owner
    email       TEXT UNIQUE NOT NULL,          -- Email address (unique constraint)
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

#### `chats`

Stores conversation threads between an agent and an owner.

```sql
CREATE TABLE IF NOT EXISTS chats (
    id          TEXT PRIMARY KEY,              -- UUID v4, auto-generated
    title       TEXT NOT NULL DEFAULT 'New Chat',
    agent_id    TEXT NOT NULL,                 -- FK to agents.id
    owner_id    TEXT NOT NULL,                 -- FK to owners.id
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
    FOREIGN KEY (owner_id) REFERENCES owners(id) ON DELETE CASCADE
);
```

#### `messages`

Stores individual messages within a chat conversation.

```sql
CREATE TABLE IF NOT EXISTS messages (
    id          TEXT PRIMARY KEY,              -- UUID v4, auto-generated
    chat_id     TEXT NOT NULL,                 -- FK to chats.id
    sender_type TEXT NOT NULL CHECK(sender_type IN ('agent', 'owner')),
    sender_id   TEXT NOT NULL,                 -- ID of the sender (agents.id or owners.id)
    content     TEXT NOT NULL,                 -- Message body
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE
);
```

### Indexes

```sql
CREATE INDEX IF NOT EXISTS idx_chats_agent_id     ON chats(agent_id);
CREATE INDEX IF NOT EXISTS idx_chats_owner_id     ON chats(owner_id);
CREATE INDEX IF NOT EXISTS idx_messages_chat_id   ON messages(chat_id);
CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at);
CREATE INDEX IF NOT EXISTS idx_agents_owner_id    ON agents(owner_id);
```

These indexes accelerate the most common query patterns: looking up chats by participant, retrieving messages for a chat (ordered by time), and finding agents owned by a specific owner.

---

## API Reference

### Base URL

- **Local development:** `http://localhost:8787`
- **Production:** `https://chat-app-backend.<your-subdomain>.workers.dev`

### Response Format

All responses follow a consistent JSON format:

**Success (single resource):**
```json
{
  "success": true,
  "data": { ... }
}
```

**Success (list of resources):**
```json
{
  "success": true,
  "data": [ ... ],
  "total": 42
}
```

**Error:**
```json
{
  "success": false,
  "data": null,
  "error": "Human-readable error message"
}
```

**HTTP Status Codes:**
| Code | Description |
|---|---|
| `200` | Success |
| `201` | Created successfully |
| `204` | No content (CORS preflight) |
| `400` | Bad request (invalid body, missing fields, validation failure) |
| `404` | Resource not found |
| `500` | Internal server error |

---

### Health Check

Check if the API is running.

```
GET /api/health
```

**Response `200`:**
```json
{
  "status": "ok",
  "service": "chat-app-backend",
  "version": "0.1.0"
}
```

**Example:**
```bash
curl http://localhost:8787/api/health
```

---

### Agents

#### Create Agent

```
POST /api/agents
```

**Request Body:**
| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | ✅ | Display name for the agent |
| `description` | string | ❌ | Optional description of the agent's capabilities |
| `owner_id` | string | ❌ | UUID of the owner (can be set later) |

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

---

#### List Agents

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

#### Get Agent

```
GET /api/agents/:id
```

**Path Parameters:**
| Parameter | Description |
|---|---|
| `id` | The UUID of the agent |

**Response `200`:**
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

**Response `404`:**
```json
{
  "success": false,
  "data": null,
  "error": "Agent 'nonexistent-id' not found"
}
```

**Example:**
```bash
curl http://localhost:8787/api/agents/a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

---

#### Update Agent

```
PUT /api/agents/:id
```

Only the fields you include will be updated. Omitted fields keep their current values.

**Path Parameters:**
| Parameter | Description |
|---|---|
| `id` | The UUID of the agent |

**Request Body:**
| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | ❌ | New display name |
| `description` | string | ❌ | New description |
| `owner_id` | string (nullable) | ❌ | New owner UUID, or `null` to unlink |

**Response `200`:** Returns the updated agent.

**Example:**
```bash
curl -X PUT http://localhost:8787/api/agents/a1b2c3d4-e5f6-7890-abcd-ef1234567890 \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Specializes in Rust and WebAssembly development"
  }'
```

---

#### Delete Agent

```
DELETE /api/agents/:id
```

**Path Parameters:**
| Parameter | Description |
|---|---|
| `id` | The UUID of the agent |

**Response `200`:** (if agent existed and was deleted)
```json
{
  "success": true,
  "data": {
    "deleted": true
  }
}
```

**Response `404`:** (if agent does not exist)

**Notes:**
- Deleting an agent will **cascade delete** all their chats and messages.
- If the agent is referenced as `owner_id` on other records, that's not applicable here (agents are deleted, not set to null).

**Example:**
```bash
curl -X DELETE http://localhost:8787/api/agents/a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

---

### Owners

#### Create Owner

```
POST /api/owners
```

**Request Body:**
| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | ✅ | Display name |
| `email` | string | ✅ | Email address (must be unique) |

**Response `201`:**
```json
{
  "success": true,
  "data": {
    "id": "o1p2q3r4-s5t6-7890-uvwx-yz1234567890",
    "name": "Alice",
    "email": "alice@example.com",
    "created_at": "2025-01-15 10:30:00"
  }
}
```

**Response `400`:** (duplicate email)
```json
{
  "success": false,
  "data": null,
  "error": "An owner with this email already exists"
}
```

**Example:**
```bash
curl -X POST http://localhost:8787/api/owners \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Alice",
    "email": "alice@example.com"
  }'
```

---

#### List Owners

```
GET /api/owners
```

Returns all owners ordered by creation date (newest first).

**Example:**
```bash
curl http://localhost:8787/api/owners
```

---

#### Get Owner

```
GET /api/owners/:id
```

**Example:**
```bash
curl http://localhost:8787/api/owners/o1p2q3r4-s5t6-7890-uvwx-yz1234567890
```

---

#### Update Owner

```
PUT /api/owners/:id
```

**Request Body:**
| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | ❌ | New display name |
| `email` | string | ❌ | New email address |

**Example:**
```bash
curl -X PUT http://localhost:8787/api/owners/o1p2q3r4-s5t6-7890-uvwx-yz1234567890 \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Alice Johnson"
  }'
```

---

#### Delete Owner

```
DELETE /api/owners/:id
```

**Notes:**
- Deleting an owner will set `owner_id` to `NULL` on all their agents (`ON DELETE SET NULL`).
- Deleting an owner will **cascade delete** all chats where they are a participant.

**Example:**
```bash
curl -X DELETE http://localhost:8787/api/owners/o1p2q3r4-s5t6-7890-uvwx-yz1234567890
```

---

### Chats

#### Create Chat

```
POST /api/chats
```

Creates a new conversation between an agent and an owner.

**Request Body:**
| Field | Type | Required | Description |
|---|---|---|---|
| `agent_id` | string | ✅ | UUID of the participating agent |
| `owner_id` | string | ✅ | UUID of the participating owner |
| `title` | string | ❌ | Chat title (defaults to "New Chat") |

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

---

#### List Chats

```
GET /api/chats
```

Returns all chats ordered by last activity (newest first).

**Example:**
```bash
curl http://localhost:8787/api/chats
```

---

#### Get Chat with Messages

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

#### Update Chat

```
PUT /api/chats/:id
```

**Request Body:**
| Field | Type | Required | Description |
|---|---|---|---|
| `title` | string | ❌ | New chat title |

**Response `200`:** Returns the full chat with messages (same format as GET).

**Example:**
```bash
curl -X PUT http://localhost:8787/api/chats/c1d2e3f4-a5b6-7890-cdef-012345678901 \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Updated: Rust project discussion"
  }'
```

---

#### Delete Chat

```
DELETE /api/chats/:id
```

**Notes:**
- Deleting a chat will **cascade delete** all its messages.

**Example:**
```bash
curl -X DELETE http://localhost:8787/api/chats/c1d2e3f4-a5b6-7890-cdef-012345678901
```

---

### Messages

#### Send Message

```
POST /api/chats/:id/messages
```

Sends a new message in the specified chat. The chat's `updated_at` timestamp is automatically updated.

**Path Parameters:**
| Parameter | Description |
|---|---|
| `id` | The UUID of the chat |

**Request Body:**
| Field | Type | Required | Description |
|---|---|---|---|
| `sender_type` | string | ✅ | Must be `"agent"` or `"owner"` |
| `sender_id` | string | ✅ | UUID of the sender (an agent or owner ID) |
| `content` | string | ✅ | The message text |

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

**Example:**
```bash
curl -X POST http://localhost:8787/api/chats/c1d2e3f4-a5b6-7890-cdef-012345678901/messages \
  -H "Content-Type: application/json" \
  -d '{
    "sender_type": "owner",
    "sender_id": "o1p2q3r4-s5t6-7890-uvwx-yz1234567890",
    "content": "I'm building a web scraper in Rust!"
  }'
```

---

#### Get Messages

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
    }
  ],
  "total": 1
}
```

**Example:**
```bash
curl http://localhost:8787/api/chats/c1d2e3f4-a5b6-7890-cdef-012345678901/messages
```

---

## Development

### Local Development

Start the wrangler dev server with live reload:

```bash
npm run dev
```

The dev server watches your Rust source files, recompiles them automatically when changes are detected, and serves the Worker at `http://localhost:8787`.

### Running Migrations

After making changes to the SQL schema in `migrations/`, apply them:

```bash
# Apply to local D1 database
npx wrangler d1 migrations apply chat-app-db --local

# Apply to remote (production) D1 database
npx wrangler d1 migrations apply chat-app-db --remote
```

### Building

To build the project without running the dev server:

```bash
npx wrangler build
```

Or use wasm-pack directly:

```bash
wasm-pack build --target web --release
```

### Adding New Dependencies

When adding new Rust dependencies to `Cargo.toml`, ensure they are compatible with the `wasm32-unknown-unknown` target. Crates that use system calls (file I/O, networking, threading) may not compile.

**Known compatible patterns:**
- Use `uuid` with the `js` feature for WASM-compatible random number generation
- Use `serde` and `serde_json` for serialization (pure Rust, no system deps)
- For timestamps: use SQL's `datetime('now')` in D1 queries rather than Rust time crates

## Deployment

### 1. Create the production D1 database (if not already done)

```bash
npx wrangler d1 create chat-app-db
```

### 2. Apply migrations to the remote database

```bash
npx wrangler d1 migrations apply chat-app-db --remote
```

### 3. Deploy the Worker

```bash
npm run deploy
```

This builds the Rust project to WebAssembly and deploys it to Cloudflare's edge network. After deployment, you'll receive a URL like:

```
https://chat-app-backend.<your-subdomain>.workers.dev
```

### 4. Verify the deployment

```bash
curl https://chat-app-backend.<your-subdomain>.workers.dev/api/health
```

## Testing with curl

Here's a complete workflow to test the API end-to-end:

```bash
# Set the base URL (change for production)
BASE="http://localhost:8787"

# 1. Health check
curl "$BASE/api/health"

# 2. Create an owner
OWNER=$(curl -s -X POST "$BASE/api/owners" \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice","email":"alice@example.com"}')
echo "$OWNER" | jq .
OWNER_ID=$(echo "$OWNER" | jq -r '.data.id')

# 3. Create an agent
AGENT=$(curl -s -X POST "$BASE/api/agents" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"CodeBot\",\"description\":\"AI coding assistant\",\"owner_id\":\"$OWNER_ID\"}")
echo "$AGENT" | jq .
AGENT_ID=$(echo "$AGENT" | jq -r '.data.id')

# 4. Create a chat
CHAT=$(curl -s -X POST "$BASE/api/chats" \
  -H "Content-Type: application/json" \
  -d "{\"title\":\"Debug help\",\"agent_id\":\"$AGENT_ID\",\"owner_id\":\"$OWNER_ID\"}")
echo "$CHAT" | jq .
CHAT_ID=$(echo "$CHAT" | jq -r '.data.id')

# 5. Send a message from the owner
curl -s -X POST "$BASE/api/chats/$CHAT_ID/messages" \
  -H "Content-Type: application/json" \
  -d "{\"sender_type\":\"owner\",\"sender_id\":\"$OWNER_ID\",\"content\":\"Can you help me debug this code?\"}" | jq .

# 6. Send a reply from the agent
curl -s -X POST "$BASE/api/chats/$CHAT_ID/messages" \
  -H "Content-Type: application/json" \
  -d "{\"sender_type\":\"agent\",\"sender_id\":\"$AGENT_ID\",\"content\":\"Sure! Share the code and I'll take a look.\"}" | jq .

# 7. Get the full chat with messages
curl -s "$BASE/api/chats/$CHAT_ID" | jq .

# 8. List all agents
curl -s "$BASE/api/agents" | jq .
```

## CORS Configuration

The backend includes built-in CORS support for development. All responses include the following headers:

```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization
Access-Control-Max-Age: 86400
```

The OPTIONS preflight handler responds with `204 No Content` for all paths.

> **Note:** For production, you may want to restrict `Access-Control-Allow-Origin` to specific domains by modifying the `with_cors()` function in `src/handlers.rs`.

## Troubleshooting

### "Worker exceeded size limit"

If you get a deployment error about the worker being too large:

1. Ensure the release profile optimizes for size (already configured in `Cargo.toml`)
2. Check for unnecessary dependencies
3. Run `wasm-pack build --release` and check the `.wasm` file size

### "D1 database not found"

Ensure you've:
1. Created the D1 database: `npx wrangler d1 create chat-app-db`
2. Updated `wrangler.toml` with the correct `database_id`
3. Applied migrations: `npx wrangler d1 migrations apply chat-app-db --local`

### "cannot find crate `worker`"

If you get a Rust compilation error about the `worker` crate:

1. Check that you have the `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
2. Try clearing the cargo cache and rebuilding

### "wasm-pack not found"

Install wasm-pack:
```bash
cargo install wasm-pack
```

### Compilation fails with mysterious WASM errors

Try:
```bash
cargo clean
cargo build --target wasm32-unknown-unknown
```

## Future Improvements

Potential enhancements for this backend:

- **Authentication:** Add JWT or API key authentication to protect endpoints
- **Pagination:** Add `limit` and `offset` query parameters to list endpoints
- **Filtering:** Support filtering chats by `agent_id` or `owner_id`
- **Real-time:** Add WebSocket support via Cloudflare Durable Objects for live chat
- **Search:** Add full-text search on messages using D1's FTS5 extension
- **Rate limiting:** Protect the API from abuse
- **Logging:** Structured logging with request IDs for debugging

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Ensure the project compiles: `npx wrangler build`
5. Test your changes: `npm run dev` and test with curl
6. Commit and push: `git commit -m "Add my feature" && git push`
7. Open a pull request

## License

MIT
