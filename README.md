# ChatCP

A serverless backend for a multi-agent chat application built with **Rust**, **Cloudflare Workers**, and **Cloudflare D1**. Comes with a **Rust CLI client** for chatting from the terminal. This API enables agents and their human owners to create and manage conversations through a RESTful interface.

ChatCP - Chat context protocol.

**Current version:** v0.2.0 — Foundation & Polish  
**Status:** ✅ Compilation verified · ✅ Input validation · ✅ Structured errors · ✅ CI/CD · ✅ 31 unit tests

📦 **New in v0.2.0:** [`cli/`](./cli) — a Rust terminal client with both quick commands and an interactive REPL

---

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Tech Stack](#tech-stack)
- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Project Structure](#project-structure)
- [Configuration](#configuration)
- [Database Schema](#database-schema)
- [Input Validation](#input-validation)
- [Error Handling](#error-handling)
- [API Reference](#api-reference)
- [CLI Client](#cli-client)
- [Development](#development)
- [Testing](#testing)
- [CI/CD Pipeline](#cicd-pipeline)
- [Deployment](#deployment)
- [Testing with curl](#testing-with-curl)
- [Contributing](#contributing)
- [License](#license)

---

## Architecture Overview

```
                        ┌─────────────────────────────────┐
                        │   Human CLI (h-cli) — TUI       │
                        │   Developer CLI (cli/)          │
                        └──────────┬──────────────────────┘
                                   │ HTTP REST API
                                   ▼
                        ┌──────────────────────────────────────┐
                        │      Cloudflare Workers (Rust)       │
                        │  ┌────────────────────────────────┐  │
                        │  │   lib.rs — entry point         │  │
                        │  │   router.rs — route definitions │  │
                        │  │   handlers.rs — HTTP layer      │  │
                        │  │   validation.rs — input checks  │  │
                        │  │   models.rs — data structures    │  │
                        │  │   db.rs — D1 queries             │  │
                        │  │   prelude.rs — shared imports    │  │
                        │  └──────────┬─────────────────────┘  │
                        │             │                         │
                        │  ┌──────────▼─────────────────────┐  │
                        │  │   D1 Database (Prepared Stmts) │  │
                        │  └──────────┬─────────────────────┘  │
                        └─────────────┼────────────────────────┘
                                      │
                         ┌────────────▼────────────┐
                         │  Cloudflare D1 (SQLite) │
                         │  - agents               │
                         │  - owners               │
                         │  - chats                │
                         │  - messages             │
                         └─────────────────────────┘
```

The backend runs as a **Cloudflare Worker** — a serverless function deployed to Cloudflare's global edge network. The **Chat CLI** (`cli/`) is the official terminal client, supporting both one-shot commands and an interactive REPL.
1. Incoming HTTP request hits the Worker
2. `lib.rs` creates the router from `router.rs` and dispatches
3. Router matches the path to the appropriate `handler` in `handlers.rs`
4. Handler parses JSON body → runs validation (`validation.rs`) → calls `db.rs`
5. `db.rs` executes a prepared SQL statement against D1
6. Result is serialized via `models.rs` types and returned as JSON

---

## Tech Stack

| Technology | Purpose | Version |
|---|---|---|
| **Rust** | Programming language — performance, safety, WASM compilation | edition 2021, stable |
| **worker** | Cloudflare Workers Rust SDK — HTTP server, D1 bindings, routing | 0.4 |
| **serde / serde_json** | JSON serialization/deserialization | 1.x |
| **uuid** | UUID v4 generation for primary keys | 1.x (with `js` feature) |
| **Cloudflare Workers** | Serverless execution environment at the edge | — |
| **Cloudflare D1** | Serverless SQLite database with global replication | — |
| **wrangler** | Cloudflare Workers CLI for dev, build, deploy | ^3.0 |
| **clap** | CLI argument parsing (chat-cli) | 4 (with derive) |
| **reqwest** | HTTP client for the CLI | 0.12 (with rustls-tls) |
| **ratatui** | TUI library for `h-cli` | 0.26 |
| **crossterm** | Terminal manipulation for `h-cli` | 0.27 |
| **prettytable-rs** | ASCII table formatting (`cli`) | 0.10 |

---

## Prerequisites

- **Rust toolchain** — Install via [rustup](https://rustup.rs/)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup target add wasm32-unknown-unknown  # only for backend
  ```
- **Node.js & npm** — v18 or later
- **Wrangler CLI** — installed via `npm install` (dev dependency)
- **A Cloudflare account** — [Sign up free](https://dash.cloudflare.com/sign-up)
- **A Cloudflare D1 database** — Created via wrangler (see Quick Start)

---

## Quick Start

### Backend

```bash
# Terminal 1 — Backend
npm install
make migrate
make dev
```

### Human Chat (TUI)

```bash
# Terminal 2 — Human Chat
cd h-cli
cargo run
```

### Developer CLI

```bash
# Terminal 3 — Developer CLI
cd cli
cargo run -- help
```

### Verify with curl

```bash
curl http://localhost:8787/api/health
# {"status":"ok","service":"chat-app-backend","version":"0.2.0"}
```

---

## Project Structure

```
chat-app-backend/
│
├── Cargo.toml                    # Backend Rust manifest (cdylib for WASM)
├── wrangler.toml                  # Cloudflare Workers configuration
├── package.json                   # Node.js project manifest with wrangler scripts
├── Makefile                       # Common commands (dev, build, test, lint, deploy)
├── rust-toolchain.toml            # Pins Rust version + wasm32-unknown-unknown target
├── .rustfmt.toml                  # Rust formatting rules (100-char lines, module imports)
├── .gitignore                     # Git ignore rules
├── README.md                      # This file
├── tasks.md                       # Roadmap & improvement plan
│
├── .github/workflows/
│   └── ci.yml                     # CI pipeline: fmt → clippy → build → test
│
├── migrations/
│   └── 0001_initial.sql           # D1 schema migration (4 tables + 5 indexes)
│
├── cli/                           # Developer CLI (quick commands & REPL)
│   ├── src/lib.rs                 # Shared library code (client, models)
│   └── src/main.rs                # Management CLI entry point
│
├── h-cli/                         # 🆕 Human Chat CLI (TUI)
│   ├── src/main.rs                # TUI entry point
│   ├── src/tui.rs                 # Ratatui rendering logic
│   └── src/session.rs             # Persistent session management
│
├── docs/                          # Detailed documentation
│   ├── ARCHITECTURE.md            # Architecture deep dive
│   ├── API_REFERENCE.md           # Full API endpoint reference
│   ├── ERROR_HANDLING.md          # Error handling & validation guide
│   └── DEVELOPMENT.md             # Development workflow & tooling
│
├── src/                           # Backend worker source
│   ├── lib.rs                     # Worker entry point (12 lines)
│   ├── router.rs                  # Route definitions
│   ├── handlers.rs                # HTTP layer — parsing, CORS, validation dispatch
│   ├── validation.rs              # Validator trait + field validators + 16 tests
│   ├── models.rs                  # Data structures, AppError, SenderType + 15 tests
│   ├── db.rs                      # D1 database CRUD operations
│   └── prelude.rs                 # Common imports & constants
│
└── tests/                         # (tests are inline in src/)

### Module Responsibilities

| File | Responsibility |
|---|---|
| **lib.rs** | Initializes the Worker, creates the router via `router::build_router()` (12 lines) |
| **router.rs** | Defines all routes: CORS preflight, health, agents, owners, chats, messages, 404 |
| **handlers.rs** | Extracts path params, parses/validates JSON bodies, applies CORS headers |
| **validation.rs** | `Validator` trait + field validators for names, emails, UUIDs, content lengths |
| **models.rs** | Database entities, request/response structs, `AppError` enum, `SenderType` enum |
| **db.rs** | All D1 SQL queries — prepared statements, CRUD operations, error mapping |
| **prelude.rs** | Re-exports commonly-used types + 10 shared constants (max lengths, defaults) |

---

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

### rust-toolchain.toml

```toml
[toolchain]
channel = "stable"
targets = ["wasm32-unknown-unknown"]
```

### Cargo.toml

The project compiles to a `cdylib` (C dynamic library) which is compiled to WebAssembly:

```toml
[lib]
crate-type = ["cdylib"]

[profile.release]
lto = true
opt-level = "s"       # Optimize for size (free plan limit: 8 MB)
strip = true
codegen-units = 1
```

---

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

### Tables

See `migrations/0001_initial.sql` for the full schema. Key points:

- **agents** — AI agent profiles. `owner_id` is nullable FK to `owners` (SET NULL on delete)
- **owners** — Human owners. Email has a UNIQUE constraint
- **chats** — Conversation threads. CASCADE delete on both agent and owner
- **messages** — Individual messages within a chat. `sender_type` CHECK constraint: `'agent'` or `'owner'`

Five indexes on commonly-queried columns (agent_id, owner_id, chat_id, created_at).

---

## Input Validation

Every mutating endpoint validates its input before any database operation. Validation is handled by the `Validator` trait in `src/validation.rs`.

### Validation Rules

| Field | Rule | Constant |
|---|---|---|
| `name` (agent/owner) | 1–200 characters | `MIN_NAME_LENGTH`, `MAX_NAME_LENGTH` |
| `email` | 1–320 characters, must contain `@` with domain | `MAX_EMAIL_LENGTH` |
| `content` (message) | 1–10,000 characters | `MAX_CONTENT_LENGTH` |
| `title` (chat) | 0–500 characters | `MAX_TITLE_LENGTH` |
| `description` (agent) | 0–2,000 characters | `MAX_DESCRIPTION_LENGTH` |
| UUID fields | Must be valid UUID v4 format (36 chars, 4 dashes, hex, version nibble `4`) | — |
| `sender_type` | Must be `"agent"` or `"owner"` | — |

### Validation Flow

```
Request body → serde_json parse → Validator::validate() → DB operation
                                     ↓
                              AppError::Validation(msg)
                                     ↓
                             400 response with error JSON
```

### Validator Trait

```rust
pub trait Validator {
    fn validate(&self) -> Result<(), AppError>;
}
```

Implemented for all 7 request types. Invalid inputs return a **400 Bad Request** with a descriptive message.

---

## Error Handling

Structured error handling via the `AppError` enum (`src/models.rs`):

### Error Types

| Variant | HTTP Status | Error Code | Description |
|---|---|---|---|
| `NotFound` | 404 | `ERR_NOT_FOUND` | Resource not found |
| `BadRequest` | 400 | `ERR_BAD_REQUEST` | Malformed request (invalid JSON) |
| `Validation` | 400 | `ERR_VALIDATION` | Input validation failure |
| `Database` | 500 | `ERR_DATABASE` | D1 query failure |
| `Internal` | 500 | `ERR_INTERNAL` | Unexpected server error |

### Error Response Format

```json
{
  "success": false,
  "data": null,
  "error": "Agent 'abc-123' not found",
  "code": "ERR_NOT_FOUND"
}
```

### Conversion Traits

- `From<worker::Error>` for `AppError` — enables `?` on D1 operations
- `From<String>` and `From<&str>` for `AppError`
- `into_response()` method converts `AppError` into a `worker::Result<Response>` with correct status code
- All `Internal` and `Database` errors are logged via `console_error!`

---

## API Reference

17 endpoints. **Full details in [docs/API_REFERENCE.md](docs/API_REFERENCE.md).**

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/health` | Health check |
| `OPTIONS` | `/*` | CORS preflight |
| **Agents** | | |
| `POST` | `/api/agents` | Create agent |
| `GET` | `/api/agents` | List all agents |
| `GET` | `/api/agents/:id` | Get agent by ID |
| `PUT` | `/api/agents/:id` | Update agent |
| `DELETE` | `/api/agents/:id` | Delete agent |
| **Owners** | | |
| `POST` | `/api/owners` | Create owner |
| `GET` | `/api/owners` | List all owners |
| `GET` | `/api/owners/:id` | Get owner by ID |
| `PUT` | `/api/owners/:id` | Update owner |
| `DELETE` | `/api/owners/:id` | Delete owner |
| **Chats** | | |
| `POST` | `/api/chats` | Create chat |
| `GET` | `/api/chats` | List all chats |
| `GET` | `/api/chats/:id` | Get chat with messages |
| `PUT` | `/api/chats/:id` | Update chat title |
| `DELETE` | `/api/chats/:id` | Delete chat |
| **Messages** | | |
| `POST` | `/api/chats/:id/messages` | Send message |
| `GET` | `/api/chats/:id/messages` | Get messages |

### Response Format

**Success (single):**
```json
{ "success": true, "data": { ... } }
```

**Success (list):**
```json
{ "success": true, "data": [ ... ], "total": 42 }
```

**Error (v0.2.0+):**
```json
{ "success": false, "data": null, "error": "...", "code": "ERR_..." }
```

---

## CLI Client

The `cli/` directory contains a full-featured Rust terminal client that communicates with the backend via HTTP. It supports two modes:

### Quick Commmands

Use for one-off operations or scripting:

```bash
cargo run -- health              # Check backend status
cargo run -- list agents         # List all agents
cargo run -- create owner --name "Alice" --email "alice@example.com"
cargo run -- create agent --name "CodeBot" --description "AI assistant"
cargo run -- send --chat-id <id> --as-type owner --sender-id <id> --text "Hello"
cargo run -- messages --chat-id <id>  # View messages
cargo run -- delete agent <id>   # Delete a resource
```

All 17 backend endpoints are available as CLI commands. See [`cli/README.md`](cli/README.md) for the full reference.

### Interactive REPL

For an immersive chat experience:

```bash
cargo run -- repl
```

In REPL mode you can browse chats with `/list`, select one with `/select <n>`, type messages directly, create new chats with `/new`, and more. See the [CLI README](cli/README.md#interactive-repl) for a full demo.

### Configuration

Set the backend URL via environment variable or the `--api-url` flag:

```bash
# Default: http://localhost:8787
export CHAT_API_URL=https://my-worker.workers.dev
cargo run -- health

# Or per-command:
cargo run -- --api-url https://my-worker.workers.dev list agents
```

---

## Development

### Backend Commands (Makefile)

```bash
make dev              # Start dev server (wrangler)
make build            # Build for WASM (release)
make check            # Check WASM compilation
make fmt              # Format all Rust code
make fmt-check        # Check formatting
make lint             # Run clippy (WASM target)
make test             # Run unit tests (native)
make test-check       # Check tests compile (WASM)
make migrate          # Apply D1 migrations (local)
make migrate-remote   # Apply D1 migrations (remote)
make deploy           # Deploy to Cloudflare
make clean            # Clean build artifacts
```

### CLI Development

The CLI is a separate Rust crate with its own `Cargo.toml`:

```bash
cd cli
cargo check              # Check compilation
cargo run -- health      # Quick test with running backend
cargo build --release    # Build for production
```

No special WASM target needed — the CLI compiles natively for your platform.

### Code Quality

- **Formatting:** `cargo fmt` with rules in `.rustfmt.toml` (100-char lines, module imports)
- **Linting:** `cargo clippy` with `-D warnings` for WASM target (backend) or native (CLI)
- **Constants:** All magic strings/numbers in `src/prelude.rs` (10 shared constants)
- **Imports:** `use crate::prelude::*;` pattern for commonly-used types

### Adding Backend Dependencies

Ensure compatibility with `wasm32-unknown-unknown`:
- ✅ Pure Rust crates (serde, serde_json)
- ✅ Crates with `js` feature flag (uuid)
- ❌ Crates with system calls (file I/O, threading, chrono)

---

## Testing

### Unit Tests

31 unit tests covering models and validation:

```bash
make test
# cargo test
```

**Test suites:**

| Module | Tests | Coverage |
|---|---|---|
| `models.rs` | 15 | SenderType (4), AppError (6), API responses (5) |
| `validation.rs` | 16 | Name (5), Email (6), Content (4), UUID (6), Validator impls (7) |

Tests run on the **native target** (x86_64), not WASM, for fast execution. Test code is gated behind `#[cfg(test)]` and excluded from WASM compilation.

### CI Pipeline

GitHub Actions runs on every push/PR:

```
Job: Check & Lint
  ├── cargo fmt -- --check
  ├── cargo clippy (WASM, -D warnings)
  └── cargo check (WASM)

Job: Test (depends on Check)
  └── cargo check --tests (WASM)
```

---

## CI/CD Pipeline

The project includes a GitHub Actions workflow (`.github/workflows/ci.yml`) with two jobs:

### Check & Lint Job
- Formatting check via `cargo fmt`
- Clippy linting for WASM target
- WASM compilation check

### Test Job
- Runs after Check passes
- Verifies tests compile for WASM target

---

## Deployment

### 1. Create the production D1 database

```bash
npx wrangler d1 create chat-app-db
```

Update `database_id` in `wrangler.toml`.

### 2. Apply migrations

```bash
make migrate-remote
# or: npx wrangler d1 migrations apply chat-app-db --remote
```

### 3. Deploy

```bash
make deploy
# or: npm run deploy
```

### 4. Verify

```bash
curl https://chat-app-backend.<your-subdomain>.workers.dev/api/health
```

---

## Testing with curl

```bash
BASE="http://localhost:8787"

# Health check
curl "$BASE/api/health"

# Create owner
OWNER=$(curl -s -X POST "$BASE/api/owners" \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice","email":"alice@example.com"}')
OWNER_ID=$(echo "$OWNER" | jq -r '.data.id')

# Create agent
AGENT=$(curl -s -X POST "$BASE/api/agents" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"CodeBot\",\"description\":\"AI coding assistant\",\"owner_id\":\"$OWNER_ID\"}")
AGENT_ID=$(echo "$AGENT" | jq -r '.data.id')

# Create chat
CHAT=$(curl -s -X POST "$BASE/api/chats" \
  -H "Content-Type: application/json" \
  -d "{\"title\":\"Debug help\",\"agent_id\":\"$AGENT_ID\",\"owner_id\":\"$OWNER_ID\"}")
CHAT_ID=$(echo "$CHAT" | jq -r '.data.id')

# Send messages
curl -s -X POST "$BASE/api/chats/$CHAT_ID/messages" \
  -H "Content-Type: application/json" \
  -d "{\"sender_type\":\"owner\",\"sender_id\":\"$OWNER_ID\",\"content\":\"Can you help me?\"}"
curl -s -X POST "$BASE/api/chats/$CHAT_ID/messages" \
  -H "Content-Type: application/json" \
  -d "{\"sender_type\":\"agent\",\"sender_id\":\"$AGENT_ID\",\"content\":\"Sure!\"}"

# Get full chat
curl -s "$BASE/api/chats/$CHAT_ID" | jq .
```

---

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run the full check suite: `make fmt-check && make lint && make test`
5. Ensure the project compiles: `make check`
6. Submit a pull request

---

## What's Next?

This project is actively evolving. Check out [tasks.md](tasks.md) for the full roadmap including:

- **v0.3.0:** Pagination, filtering, and sorting for all list endpoints
- **v1.0.0:** Authentication, API keys, and rate limiting
- **v1.1.0:** WebSocket support for real-time chat via Durable Objects
- **v2.0.0:** Multi-tenancy with organizations and team management

## License

MIT
