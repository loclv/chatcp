# Architecture

> **Project:** Chat App Backend — Rust + Cloudflare Workers + D1
> **Version:** v0.2.0

---

## System Overview

The backend is a **serverless REST API** deployed to Cloudflare's global edge network. It uses a layered architecture with clear separation of concerns between HTTP handling, business logic, and data access.

```
┌─────────────────────────────────────────────────────────────┐
│                     Client Application                       │
│          (Web App, Mobile App, CLI, AI Agent)                │
└────────────────────────┬────────────────────────────────────┘
                         │ HTTPS
                         ▼
┌─────────────────────────────────────────────────────────────┐
│             Cloudflare Edge Network (200+ locations)         │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              Cloudflare Worker (Rust WASM)            │  │
│  │                                                       │  │
│  │  ┌──────────┐  ┌──────────┐  ┌────────────────────┐  │  │
│  │  │  lib.rs  │─▶│router.rs │─▶│   handlers.rs      │  │  │
│  │  │ (entry)  │  │ (routes) │  │ (HTTP layer)       │  │  │
│  │  └──────────┘  └──────────┘  └─────────┬──────────┘  │  │
│  │                                        │              │  │
│  │  ┌─────────────────────────────────────▼──────────┐  │  │
│  │  │            validation.rs                        │  │  │
│  │  │  (Validator trait, field validators)            │  │  │
│  │  └─────────────────────────────────────┬──────────┘  │  │
│  │                                        │              │  │
│  │  ┌─────────────────────────────────────▼──────────┐  │  │
│  │  │               db.rs                             │  │  │
│  │  │  (D1 prepared statements, CRUD operations)      │  │  │
│  │  └─────────────────────────────────────┬──────────┘  │  │
│  │                                        │              │  │
│  │  ┌─────────────────────────────────────▼──────────┐  │  │
│  │  │   models.rs / prelude.rs                       │  │  │
│  │  │   (Data structures, constants, shared types)    │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────┘  │
└────────────────────────┬────────────────────────────────────┘
                         │ D1 Query API
                         ▼
┌─────────────────────────────────────────────────────────────┐
│               Cloudflare D1 (SQLite-based)                   │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐           │
│  │  agents    │  │  owners    │  │  chats     │           │
│  ├────────────┤  ├────────────┤  ├────────────┤           │
│  │ id (PK)    │  │ id (PK)    │  │ id (PK)    │           │
│  │ name       │  │ name       │  │ title      │           │
│  │ description│  │ email (UQ) │  │ agent_id   │──▶ agents │
│  │ owner_id   │──▶ owners     │  │ owner_id   │──▶ owners │
│  │ created_at │  │ created_at │  │ created_at │           │
│  │ updated_at │  └────────────┘  │ updated_at │           │
│  └────────────┘                  └─────┬──────┘           │
│                                        │                   │
│                               ┌────────▼────────┐         │
│                               │    messages     │         │
│                               ├─────────────────┤         │
│                               │ id (PK)         │         │
│                               │ chat_id ────────┤──▶ chats│
│                               │ sender_type     │         │
│                               │ sender_id       │         │
│                               │ content         │         │
│                               │ created_at      │         │
│                               └─────────────────┘         │
└─────────────────────────────────────────────────────────────┘
```

---

## Request Lifecycle

### Step-by-step flow of an API request:

```
1. HTTP Request arrives at Cloudflare Edge
       │
2. Worker runtime routes to Rust WASM binary
       │
3. lib.rs::main() → calls router::build_router()
       │
4. Router matches request path + method
       │
5. handlers::handler_fn() is invoked
       │
   ├── 5a. Extracts path params (ctx.param("id"))
   ├── 5b. Parses JSON body (req.json::<T>())
   ├── 5c. Validates input (T::validate())          ← validation.rs
   └── 5d. Calls db::function()                     ← db.rs
       │
6. DB function prepares SQL query + binds params
       │
7. D1 executes query against SQLite storage
       │
8. Result is mapped to response struct             ← models.rs
       │
9. Response is serialized to JSON
       │
10. CORS headers applied → HTTP response returned
```

### Error path (any step can fail):

```
Step 5b (parse error)    → AppError::BadRequest  → 400
Step 5c (invalid data)   → AppError::Validation  → 400
Step 6 (DB error)        → AppError::Database    → 500 (logged)
Step 8 (not found)       → AppError::NotFound    → 404
Any unexpected error     → AppError::Internal    → 500 (logged)
```

---

## Module Architecture

### Module Dependency Graph

```
lib.rs
  ├── router.rs ───→ handlers.rs
  │                    ├── db.rs ───→ prelude.rs
  │                    │              ├── models.rs
  │                    │              └── constants
  │                    └── validation.rs
  │                         └── prelude.rs
  ├── models.rs (self-contained)
  ├── prelude.rs ───→ models.rs
  └── validation.rs ───→ prelude.rs
```

**Key design decisions:**

1. **No circular dependencies** — `prelude.rs` only depends on `models.rs`, `validation.rs` depends on `prelude.rs` (one-way)
2. **Router is isolated** — `router.rs` has no knowledge of validation or DB internals, only handler function signatures
3. **Handlers bridge two worlds** — `handlers.rs` knows about HTTP (parsing, CORS) and DB (calling `db.rs` functions)
4. **prelude.rs centralizes imports** — reduces repetitive `use crate::models::*` across modules

---

## Module Details

### lib.rs — Entry Point

The smallest file in the project (12 lines). Responsibilities:

- Declares all modules (`mod db; mod handlers; mod models; mod prelude; mod router; mod validation;`)
- Defines the `#[event(fetch)]` entry point
- Creates the router via `router::build_router()` and runs it

```rust
#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = router::build_router();
    router.run(req, env).await
}
```

### router.rs — Route Definitions

Uses the `worker::Router` to define all HTTP routes:

```rust
pub fn build_router() -> Router<()> {
    Router::new()
        .options_async("/*", |_req, _ctx| async move { ... })
        .get_async("/api/health", |_req, _ctx| async move { ... })
        .post_async("/api/agents", handlers::create_agent)
        .get_async("/api/agents", handlers::list_agents)
        .get_async("/api/agents/:id", handlers::get_agent)
        // ... remaining 14 routes
        .or_else(|_req, _ctx| async move { Response::error("Not Found", 404) })
}
```

**Route conventions:**
- All API paths prefixed with `/api/`
- Path params denoted with `:param_name` syntax (e.g., `/api/agents/:id`)
- CORS preflight handled via `options_async("/*", ...)`
- Wildcard 404 fallback via `or_else`

### handlers.rs — HTTP Layer

Bridges the HTTP and database worlds. Each handler:

1. Gets the D1 binding from the environment
2. Extracts path params (if any)
3. Parses and validates the JSON body (if applicable)
4. Calls the appropriate `db::` function
5. Wraps the response with CORS headers

**Key pattern — `parse_and_validate` helper:**

```rust
fn parse_and_validate<T>(req: &Request) -> Result<T, Response>
where
    T: DeserializeOwned + Validator,
{
    let body: T = req.json::<T>().map_err(|e| {
        AppError::BadRequest(format!("Invalid request body: {}", e))
            .into_response().unwrap()
    })?;
    body.validate().map_err(|e| e.into_response().unwrap())?;
    Ok(body)
}
```

This ensures every mutating endpoint validates input before reaching the database layer.

### validation.rs — Input Validation

Implements the `Validator` trait for all 7 request types:

| Request Type | Validated Fields |
|---|---|
| `CreateAgentRequest` | name, description, owner_id |
| `UpdateAgentRequest` | name (optional), description (optional), owner_id (optional) |
| `CreateOwnerRequest` | name, email |
| `UpdateOwnerRequest` | name (optional), email (optional) |
| `CreateChatRequest` | agent_id, owner_id, title (optional) |
| `UpdateChatRequest` | title (optional) |
| `SendMessageRequest` | sender_type, sender_id, content |

**Validation functions:**

| Function | What it checks |
|---|---|
| `validate_name(name, field)` | Non-empty, 1–200 chars, trimmed |
| `validate_email(email, field)` | Non-empty, ≤320 chars, contains `@` with domain |
| `validate_content(content, field, max_len)` | Non-empty, ≤max_len chars, trimmed |
| `validate_uuid(id, field)` | 36 chars, dashes at positions 8/13/18/23, version nibble `4`, all hex |

### models.rs — Data Structures

Contains all serializable types:

- **Enums:** `SenderType` (Agent, Owner), `AppError` (5 variants)
- **Database entities:** `Agent`, `Owner`, `Chat`, `Message`
- **Request types:** `CreateAgentRequest`, `UpdateAgentRequest`, `CreateOwnerRequest`, `UpdateOwnerRequest`, `CreateChatRequest`, `UpdateChatRequest`, `SendMessageRequest`
- **Response types:** `ApiResponse<T>`, `PaginatedResponse<T>`, `ChatWithMessages`, `ErrorResponse`

### db.rs — Database Layer

Contains all D1 SQL queries as async functions. Each function:

1. Takes a `&D1Database` reference and request-specific params
2. Prepares a SQL statement with `?` placeholders
3. Binds parameters using `.bind(&[...])`
4. Executes with `.run().await`, `.all::<T>().await`, or `.first::<T>().await`
5. Returns `worker::Result<Response>` with proper error handling

**Key helper — `optional_d1_value`:**

```rust
fn optional_d1_value(opt: &Option<String>) -> D1Value {
    match opt {
        Some(val) => val.as_str().into(),
        None => D1Value::Null,
    }
}
```

### prelude.rs — Shared Imports & Constants

Centralizes commonly-used imports and defines shared constants:

| Constant | Value | Used In |
|---|---|---|
| `DEFAULT_CHAT_TITLE` | `"New Chat"` | `create_chat` |
| `MIN_NAME_LENGTH` | `1` | Name validation |
| `MAX_NAME_LENGTH` | `200` | Name validation |
| `MAX_TITLE_LENGTH` | `500` | Chat title validation |
| `MAX_CONTENT_LENGTH` | `10_000` | Message content validation |
| `MAX_DESCRIPTION_LENGTH` | `2_000` | Agent description validation |
| `MAX_EMAIL_LENGTH` | `320` | Email validation |
| `DEFAULT_PAGE_LIMIT` | `50` | (reserved for pagination) |
| `MAX_PAGE_LIMIT` | `1_000` | (reserved for pagination) |

---

## WASM Compatibility Considerations

The entire binary runs as WebAssembly on Cloudflare's `wasm32-unknown-unknown` target. This imposes constraints:

### Compatible Patterns ✅

| Pattern | Example |
|---|---|
| Pure Rust crates | `serde`, `serde_json` |
| Crates with `js` feature flag | `uuid = { version = "1", features = ["v4", "js"] }` |
| SQL-level timestamps | `datetime('now')` in D1 queries |
| Worker crate SDK | `worker`, `worker-sys` |

### Incompatible Patterns ❌

| Pattern | Why | Alternative |
|---|---|---|
| System crates | `chrono` reads system timezone | Use SQL `datetime('now')` |
| File I/O | WASM has no file system | N/A (serverless) |
| Threading | WASM has no threads | Use async/await |
| Raw syscalls | Not available in WASM | Use worker crate APIs |

---

## CORS Configuration

All responses include CORS headers for development:

```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization
Access-Control-Max-Age: 86400
```

Preflight (`OPTIONS`) requests return `204 No Content`.

> **For production:** Restrict `Access-Control-Allow-Origin` to specific domains by modifying the `with_cors()` function in `src/handlers.rs`.

---

## Testing Architecture

Unit tests are embedded inline using `#[cfg(test)]` modules:

```
src/
├── models.rs ─── #[cfg(test)] mod tests ─── 15 tests
├── validation.rs ─── #[cfg(test)] mod tests ─── 16 tests
```

Tests run on the **native x86_64 target** (not WASM) via `cargo test` for fast execution. The CI pipeline verifies tests compile for WASM but executes them natively.

---

## Build Pipeline

```
Source (Rust .rs files)
    │
    ▼
cargo check (WASM target)  ← compilation validation
    │
    ▼
cargo clippy (WASM target)  ← linting
    │
    ▼
wasm-pack build --release   ← WASM binary generation
    │
    ▼
wrangler deploy              ← Edge deployment
```
