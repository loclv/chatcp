# Tasks & Roadmap

> **Project:** Chat App Backend — Rust + Cloudflare Workers + D1
> **Status:** v0.1.0 — Initial MVP complete with full CRUD API

---

## Legend

| Icon | Meaning |
|------|---------|
| 🟢 **P0** | Critical — blocking / must-have |
| 🟡 **P1** | Important — high value, should do soon |
| 🔵 **P2** | Nice to have — good improvement, moderate effort |
| ⚪ **P3** | Future — interesting idea, no urgency |
| 🔴 **Breaking** | May require schema migration or API changes |
| 🛡️ **Security** | Security-relevant |
| ⚡ **Performance** | Performance-related |
| 📚 **Docs** | Documentation |

**Effort:** 🟢 Small (< 1 day) · 🟡 Medium (1–3 days) · 🔴 Large (3–7 days)

---

## Quick Reference: High-Impact, Low-Effort Wins

These are the items that deliver the most value for the least effort. Start here.

| # | Task | Priority | Effort | Impact |
|---|------|----------|--------|--------|
| 1.1 | Verify code compilation with cargo check | 🟢 P0 | 🟢 Small | Unlocks development |
| 1.2 | Add pagination (limit/offset) to list endpoints | 🟡 P1 | 🟢 Small | High — needed for any real usage |
| 1.3 | Add "me" endpoints (get agents by owner, chats by participant) | 🟡 P1 | 🟢 Small | High — common query pattern |
| 2.1 | JWT / API key authentication | 🟡 P1 | 🟡 Medium | Critical for production deployment |
| 1.4 | Input validation (required fields, string lengths) | 🟢 P0 | 🟢 Small | Prevents bad data |

---

## Phase 1: Foundation & Polish

### 1.1 Fix Compilation & Build Pipeline
*Get the project compiling end-to-end, ensure the toolchain is correct.*

- [ ] **Run `cargo check`** to verify compilation with the `worker` crate
- [ ] **Add `rust-toolchain.toml`** to pin the Rust version and WASM target:
  ```toml
  [toolchain]
  channel = "stable"
  targets = ["wasm32-unknown-unknown"]
  ```
- [ ] **Add a Makefile or Justfile** with common commands:
  ```makefile
  dev:
      npm run dev
  build:
      wasm-pack build --target web --release
  check:
      cargo check --target wasm32-unknown-unknown
  migrate:
      npx wrangler d1 migrations apply chat-app-db --local
  ```
- [ ] **Dependency cleanup:** Audit dependencies — remove unused, pin exact versions
- [ ] **Add `build.rs`** if needed for WASM compatibility quirks

### 1.2 Input Validation
*Currently the API accepts any data — add validation to prevent bad data from entering the system.*

- [ ] **Name validation:** Minimum/maximum length, reject empty strings
- [ ] **Email validation:** Basic format check on `CreateOwnerRequest` / `UpdateOwnerRequest`
- [ ] **Content validation:** Message content must not be empty, max length (e.g. 10,000 chars)
- [ ] **Title validation:** Chat title max length
- [ ] **UUID format check:** Validate that `agent_id`, `owner_id`, `sender_id` look like UUIDs before querying
- [ ] **Sender type enum:** Change `sender_type` from `String` to a Rust enum for type safety
- [ ] **Consolidated validation module:** Create `src/validation.rs` with reusable validators

### 1.3 Better Error Handling & Error Types
*Replace ad-hoc error helpers with a structured error system.*

- [ ] **Create `AppError` enum:**
  ```rust
  pub enum AppError {
      NotFound(String),
      BadRequest(String),
      Validation(String),
      Database(String),
      Internal(String),
  }
  ```
- [ ] **Implement `Into<Response>` for `AppError`** to unify error responses
- [ ] **Add structured error codes** (e.g. `ERR_NOT_FOUND`, `ERR_VALIDATION`) to JSON error responses
- [ ] **Log all internal errors** with request context (already partially done with `console_error!`)
- [ ] **Add request ID to error responses** for debugging

### 1.4 Code Quality & Architecture
*Improve the internal structure and add tooling.*

- [ ] **Add `rustfmt` configuration** (`.rustfmt.toml`) — 4-space tabs, 100-char lines
- [ ] **Add `clippy` linting** — fix all clippy warnings
- [ ] **Separate the router into its own module** (`src/router.rs`) to keep `lib.rs` clean
- [ ] **Extract constants:** Move magic strings to constants (e.g., `DEFAULT_CHAT_TITLE`, `MAX_CONTENT_LENGTH`)
- [ ] **Add `prelude.rs`** pattern for commonly-used imports
- [ ] **CI Pipeline:** Add GitHub Actions workflow for `cargo check`, `clippy`, `fmt` checks

### 1.5 Testing Infrastructure
*Currently there are zero tests. Fix this as early as possible.*

- [ ] **Unit tests for models:** Test serialization/deserialization of all request/response types
- [ ] **Unit tests for validation:** Test input validation logic
- [ ] **Integration test setup:** Use `worker` crate's test utilities to create a test D1 instance
- [ ] **API integration tests:** Test each endpoint with valid and invalid inputs
  - Test 200/201 responses
  - Test 400 responses (bad requests)
  - Test 404 responses (not found)
  - Test 500 error paths
- [ ] **End-to-end workflow test:** Create owner → create agent → create chat → send messages → verify
- [ ] **Add test database fixture** — a seed script for the test D1 database

---

## Phase 2: API Enhancements

### 2.1 Pagination (High Impact)
*All list endpoints return ALL results — add pagination before real usage.*

- [ ] **Add query parameter support** (`limit`, `offset`, `cursor`) to list handlers
- [ ] **Update `db.rs` list functions** to accept pagination params and modify SQL:
  ```sql
  SELECT * FROM agents ORDER BY created_at DESC LIMIT ? OFFSET ?
  ```
- [ ] **Update response format** to include pagination metadata:
  ```json
  {
    "success": true,
    "data": [...],
    "pagination": {
      "limit": 20,
      "offset": 0,
      "total": 142,
      "has_more": true
    }
  }
  ```
- [ ] **Set default limit** (e.g. 50) and enforce max limit (e.g. 1000)
- [ ] **Add `PaginationParams` struct** to `models.rs`:
  ```rust
  pub struct PaginationParams {
      pub limit: Option<u32>,
      pub offset: Option<u32>,
  }
  ```

### 2.2 Filtering & Search
*Allow clients to filter resources by common criteria.*

- [ ] **Filtering, agents list:** Filter by `owner_id` query parameter
  ```http
  GET /api/agents?owner_id=<uuid>
  ```
- [ ] **Filtering, chats list:** Filter by `agent_id` and/or `owner_id`
  ```http
  GET /api/chats?agent_id=<uuid>&owner_id=<uuid>
  ```
- [ ] **Full-text search on messages:** Use D1's built-in FTS5 extension:
  ```sql
  CREATE VIRTUAL TABLE messages_fts USING fts5(content, chat_id);
  ```
- [ ] **Search endpoint:** `GET /api/chats/:id/messages?q=<search-term>`
- [ ] **Date range filtering:** Support `created_after` and `created_before` query params

### 2.3 Sorting
*Control the sort order of list responses.*

- [ ] **Sort direction:** Add `sort_order` param (`asc` / `desc`, default to `desc` for most)
- [ ] **Sort by field:** Add `sort_by` param (`created_at`, `updated_at`, `name`, `email`, `title`)
- [ ] **Whitelist sort fields** to prevent SQL injection via column names
- [ ] **Add `SortParams` struct** to `models.rs`

### 2.4 Endpoint Expansion
*New API endpoints to fill gaps.*

- [ ] **`GET /api/agents/:id/chats`** — List all chats for a specific agent
- [ ] **`GET /api/owners/:id/chats`** — List all chats for a specific owner
- [ ] **`GET /api/agents/:id/owner`** — Get the owner of an agent (if any)
- [ ] **`GET /api/owners/:id/agents`** — List all agents owned by an owner
- [ ] **`GET /api/agents/:id/messages`** — Get all messages sent by a specific agent across all chats
- [ ] **`PATCH /api/agents/:id`** — Add partial update support (already have `PUT` which works like PATCH)
- [ ] **`POST /api/agents/batch`** — Batch create agents

### 2.5 Response Improvements
*Make responses more informative and useful.*

- [ ] **Include related resource summaries** — e.g., include agent name and owner name in chat responses
- [ ] **Add `ETag` / `Last-Modified` headers** for caching
- [ ] **Add `Cache-Control` headers** for cacheable responses (list endpoints)
- [ ] **Compress responses** with Gzip/Brotli (Cloudflare Workers does this automatically at the edge)
- [ ] **Add `X-Request-Id`** header to all responses
- [ ] **Add `X-Response-Time`** header for performance monitoring

---

## Phase 3: Security & Authentication

### 3.1 Authentication (Critical for Production)
*Currently the API is completely open — anyone can create/delete resources.*

- [ ] **JWT authentication** using `jsonwebtoken` crate:
  ```rust
  // Pseudo-implementation
  let token = extract_bearer_token(req);
  let claims = jwt::decode(token, secret, &algorithms)?;
  ctx.data(claims); // Pass claims to handlers
  ```
- [ ] **API key authentication** as an alternative to JWT:
  - Store API keys in a new `api_keys` D1 table
  - Support key-per-agent and key-per-owner
- [ ] **Auth middleware:** Create a reusable auth layer that validates tokens before request processing
- [ ] **Protected routes:** Require auth for all mutating endpoints (POST, PUT, DELETE)
- [ ] **Public routes:** Keep `GET /api/health` and `OPTIONS /*` open
- [ ] **Owner-only actions:** Verify that an authenticated owner can only modify their own resources
- [ ] **Agent auth:** Support agents authenticating to send messages on their own behalf

### 3.2 Authorization & Resource Ownership
*Ensuring users can only access their own resources.*

- [ ] **Ownership model:** Each resource is owned by a user (owner or agent)
- [ ] **Access control middleware:**
  - Owners can CRUD their own agents
  - Owners can CRUD their own chats
  - Agents can read/write messages in their chats
- [ ] **Admin role:** Super-admin users who can access all resources
- [ ] **Permission table:** Add a `permissions` table for fine-grained access control

### 3.3 Rate Limiting
*Protect the API from abuse.*

- [ ] **Rate limit by IP** using Cloudflare's built-in rate limiting (configure in Cloudflare dashboard)
- [ ] **Rate limit by API key** — track request counts per key in D1
- [ ] **Rate limit tiers:** Different limits for different endpoint types (e.g., 1000/hr for reads, 100/hr for writes)
- [ ] **Rate limit headers** in responses:
  ```
  X-RateLimit-Limit: 100
  X-RateLimit-Remaining: 42
  X-RateLimit-Reset: 1645567842
  ```
- [ ] **429 response** when rate limited:
  ```json
  {
    "success": false,
    "data": null,
    "error": "Rate limit exceeded. Try again in 47 seconds."
  }
  ```

### 3.4 Input Sanitization
*Prevent injection and XSS attacks.*

- [ ] **SQL injection:** Already prevented through D1 prepared statements ✅
- [ ] **XSS sanitization:** Sanitize message content before storing (strip HTML tags if needed)
- [ ] **NoSQL** — N/A (SQLite-based)
- [ ] **Size limits:** Enforce maximum sizes for all string fields
- [ ] **Whitespace trimming:** Auto-trim leading/trailing whitespace on names, emails, titles

---

## Phase 4: Real-time & Advanced Features

### 4.1 WebSocket Support (Real-time Chat)
*Add real-time messaging using Cloudflare Durable Objects.*

- [ ] **Create Durable Object** for managing WebSocket connections per chat:
  ```rust
  // Durable Object class for chat sessions
  pub struct ChatRoom {
      sessions: Vec<WebSocket>,
      // ...
  }
  ```
- [ ] **WebSocket upgrade endpoint:** `GET /api/chats/:id/ws`
- [ ] **Broadcast messages:** When a message is sent via REST or WebSocket, broadcast to all connected clients
- [ ] **Connection state management:** Handle connect, disconnect, reconnect
- [ ] **Online presence:** Show which users/agents are currently online in a chat
- [ ] **Typing indicators:** Broadcast typing events (not persisted to DB)
- [ ] **Read receipts:** Track and expose which messages have been read

### 4.2 Server-Sent Events (SSE)
*A simpler alternative to WebSockets for one-way real-time updates.*

- [ ] **SSE endpoint:** `GET /api/chats/:id/events` — stream new messages as they arrive
- [ ] **Auto-reconnect support** using standard SSE protocol
- [ ] **Less complex than WebSockets** — good for read-only real-time updates

### 4.3 Notifications & Webhooks
*Notify external systems of events.*

- [ ] **Webhook registration:** `POST /api/webhooks` — register a URL to receive events:
  ```json
  {
    "url": "https://my-app.com/webhooks/chat-events",
    "events": ["message.created", "chat.created", "agent.created"],
    "secret": "whsec_..."
  }
  ```
- [ ] **Event types:** Define a set of event types with payload schemas:
  - `message.created` — new message sent
  - `chat.created` — new chat created
  - `agent.created` / `agent.updated` / `agent.deleted`
  - `owner.created` / `owner.updated` / `owner.deleted`
- [ ] **Webhook delivery:** Queue and deliver webhooks with retry logic
- [ ] **Webhook signatures:** Sign payloads with HMAC-SHA256 for verification
- [ ] **Email notifications** — Send email to owner when agent sends a message (via Cloudflare Email Routing or SendGrid)

### 4.4 Message Features
*Richer message capabilities.*

- [ ] **Message editing:** `PUT /api/chats/:id/messages/:msg_id` — update message content
- [ ] **Message deletion:** `DELETE /api/chats/:id/messages/:msg_id` — soft or hard delete
- [ ] **Reply threads:** Support replies to specific messages (`parent_message_id` field)
- [ ] **Message reactions:** Emoji reactions on messages (new `reactions` table)
- [ ] **File attachments:** Integrate with Cloudflare R2 for file uploads
  - Upload endpoint: `POST /api/chats/:id/attachments`
  - Store R2 object key in messages table
- [ ] **Message types:** Extend beyond plain text — support markdown, code blocks, rich text
- [ ] **Streaming responses:** Support streaming message content (useful for AI agent responses)

---

## Phase 5: Observability & Production Readiness

### 5.1 Logging & Monitoring
*See what's happening in production.*

- [ ] **Structured JSON logging** using the `tracing` crate:
  ```json
  {
    "timestamp": "2025-01-15T10:30:00Z",
    "level": "INFO",
    "request_id": "req_abc123",
    "method": "POST",
    "path": "/api/agents",
    "status": 201,
    "duration_ms": 42
  }
  ```
- [ ] **Request tracing:** Trace a request through all layers (handler → db → response)
- [ ] **Error tracking:** Collect and report errors (Cloudflare Workers dashboard already captures `console_error!`)
- [ ] **Metrics endpoint:** `GET /api/metrics` with Prometheus-style metrics:
  ```
  # HELP chat_requests_total Total number of API requests
  # TYPE chat_requests_total counter
  chat_requests_total{method="GET",path="/api/agents",status="200"} 156
  ```
- [ ] **Health check** → add more details: database connectivity, latency percentiles
- [ ] **Setup Sentry** or similar error tracking integration

### 5.2 Performance Optimization
*Make the API faster and more efficient.*

- [ ] **Query optimization:** Add `EXPLAIN QUERY PLAN` analysis on slow queries
- [ ] **D1 query batching:** Batch multiple queries where possible
- [ ] **Response compression** (handled by Cloudflare Workers at the edge)
- [ ] **Caching layer:** Add a cache (Cloudflare KV) for frequently accessed resources:
  - Agent profiles (change infrequently)
  - Owner profiles
- [ ] **Cache invalidation:** Invalidate cache entries on resource updates
- [ ] **Pagination** — already in Phase 2, but critical for performance at scale
- [ ] **N+1 query prevention:** When returning lists, avoid querying for each item individually
- [ ] **Cold start optimization:** Reduce WASM binary size further

### 5.3 Backup & Data Management
*Protect against data loss.*

- [ ] **Automated D1 backups** — Cloudflare D1 has built-in backups via the dashboard
- [ ] **Export endpoint:** `GET /api/export/chats/:id` — export chat as JSON
- [ ] **Export all data:** `GET /api/export/user/:owner_id` — export all user data (GDPR compliance)
- [ ] **Data retention policy:** Auto-delete messages older than N days (configurable)
- [ ] **Soft delete** — mark resources as deleted instead of actually deleting them
- [ ] **Audit log:** Track all mutations with who made them and when

---

## Phase 6: Ecosystem & Extensions

### 6.1 Client SDKs
*Make it easy for other applications to use this API.*

- [ ] **TypeScript / JavaScript SDK:**
  ```typescript
  import { ChatAppClient } from 'chat-app-client';
  
  const client = new ChatAppClient({ apiKey: '...', baseUrl: '...' });
  const agents = await client.agents.list();
  const chat = await client.chats.create({ agentId, ownerId });
  ```
- [ ] **Python SDK** — for AI agent integrations
- [ ] **OpenAPI / Swagger spec:** Auto-generate from Rust types (or maintain manually)
- [ ] **API documentation page:** Deploy a Swagger UI or Redoc page alongside the API

### 6.2 Frontend Application
*A simple chat UI that consumes this API.*

- [ ] **React dashboard app** (or similar):
  - Owner login / agent selection
  - Chat list sidebar
  - Message view
  - Message composer
- [ ] **WebSocket integration** for real-time updates
- [ ] **File upload UI** with drag-and-drop

### 6.3 Integrations
*Connect with external platforms.*

- [ ] **Slack integration:** Bridge chat conversations to Slack channels
- [ ] **Discord integration:** Bridge to Discord
- [ ] **Email integration:** Send/receive messages via email (owner replies via email)
- [ ] **Webhook triggers** — allow external services to send messages via webhooks
- [ ] **AI agent integration** — connect agents to LLM APIs (OpenAI, Anthropic, etc.)

### 6.4 Multi-Tenancy
*Support multiple organizations.*

- [ ] **Organization model:** Add `organizations` table
- [ ] **Team management:** Multiple owners per organization
- [ ] **Role-based access control:** Admin, member, viewer roles per organization
- [ ] **Isolation:** Data is separated by organization (add `org_id` to all tables)
- [ ] **Billing / usage tracking** — track API usage per organization
- [ ] **Custom domains** — support custom domains per organization

---

## Version Roadmap

| Version | Focus | Key Deliverables |
|---------|-------|------------------|
| **v0.1.0** ✅ | MVP | Basic CRUD API, D1 schema, 17 endpoints |
| **v0.2.0** | Foundation | Pagination, filtering, input validation, tests, CI |
| **v0.3.0** | Authentication | JWT auth, API keys, rate limiting, access control |
| **v1.0.0** | Production | Observability, error handling, documentation, deployment scripts |
| **v1.1.0** | Real-time | WebSocket support (Durable Objects) |
| **v1.2.0** | Rich messages | Attachments, reactions, message editing/deletion |
| **v1.3.0** | Integrations | Webhooks, SDKs, Slack/Discord bridges |
| **v2.0.0** | Multi-tenant | Organizations, team management, billing |

---

## Detailed Task Breakdown: v0.2.0 Sprint

Here's a concrete sprint plan for the next version.

### Sprint Goal
Deliver a production-quality foundation with pagination, validation, testing, and CI.

### Sprint Backlog

| Task | Est. Effort | Dependencies | Assignee |
|------|-------------|--------------|----------|
| Add pagination (limit/offset) to all list endpoints | 4h | None | — |
| Add query parameter parsing to handlers | 2h | None | — |
| Implement input validation module | 3h | None | — |
| Add `AppError` enum and integrate error handling | 2h | — | — |
| Add HTTP endpoint tests with local D1 | 6h | Error handling | — |
| Add GitHub Actions CI | 2h | Tests | — |
| Add `rust-toolchain.toml` | 15m | None | — |
| Add Makefile with common commands | 30m | None | — |
| Rustfmt + Clippy configuration | 30m | None | — |
| Filtering: filter agents by owner_id | 2h | Pagination | — |
| Filtering: filter chats by agent_id, owner_id | 3h | Pagination | — |

**Total estimated effort:** ~25 hours

### Acceptance Criteria
- [ ] All list endpoints accept `limit` and `offset` query params
- [ ] Invalid input returns 400 with descriptive error message
- [ ] All error responses follow a consistent format
- [ ] GitHub Actions runs `cargo check`, `clippy`, and `fmt` on every PR
- [ ] Minimum 80% of handlers covered by integration tests
- [ ] Agents endpoint supports `?owner_id=` filter
- [ ] Chats endpoint supports `?agent_id=` and `?owner_id=` filters

---

## Decision Log

Track architectural decisions made during development.

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|------------------------|
| 2025-01 | Use UUID v4 as primary keys | Works well with D1, no auto-increment issues in distributed systems | Auto-increment integers, ULID, CUID |
| 2025-01 | Use `datetime('now')` in SQL rather than Rust timers | WASM has limited time support, D1 handles timezone consistently | `chrono` crate, `js_sys::Date` |
| 2025-01 | All timestamps stored as TEXT in ISO format | D1 returns TEXT from `datetime()`, simplifies parsing | INTEGER Unix timestamps |
| 2025-01 | Flatten `ChatWithMessages` with `#[serde(flatten)]` | Cleaner API response — keeps `chat` fields at top level alongside `messages` | Nested `{ chat: {...}, messages: [...] }` |
| 2025-01 | Single `sender_type` field on messages | Simple and extensible — both agent and owner use the same messages table | Separate tables for agent vs owner messages |

---

## Technical Debt Register

Track known issues that need attention.

| Issue | Severity | Created | Status |
|-------|----------|---------|--------|
| `or_else` method may need prefix path filtering | Low | 2025-01 | Open |
| No unit tests for any modules | High | 2025-01 | Open |
| No input validation on any fields | High | 2025-01 | Open |
| All list endpoints return unlimited results | Medium | 2025-01 | Open |
| Error responses inconsistent between modules | Medium | 2025-01 | Open |

---

*This roadmap is a living document. Update it as priorities shift and new ideas emerge.*
