# Tasks & Roadmap

> **Project:** Chat App Backend — Rust + Cloudflare Workers + D1
> **Status:** v0.2.0 — Foundation complete (validation, error handling, CI, tests)

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

| # | Task | Priority | Effort | Status |
|---|------|----------|--------|--------|
| 1.1 | Verify code compilation with cargo check | 🟢 P0 | 🟢 Small | ✅ Done |
| 1.2 | Add pagination (limit/offset) to list endpoints | 🟡 P1 | 🟢 Small | ⬜ Open |
| 1.3 | Add "me" endpoints (get agents by owner, chats by participant) | 🟡 P1 | 🟢 Small | ⬜ Open |
| 2.1 | JWT / API key authentication | 🟡 P1 | 🟡 Medium | ⬜ Open |
| 1.4 | Input validation (required fields, string lengths) | 🟢 P0 | 🟢 Small | ✅ Done |
| 1.5 | Better error handling with structured error types | 🟢 P0 | 🟢 Small | ✅ Done |

---

## Phase 1: Foundation & Polish ✅ (Completed)

### 1.1 Fix Compilation & Build Pipeline

- [x] **Run `cargo check`** to verify compilation with the `worker` crate
- [x] **Add `rust-toolchain.toml`** to pin the Rust version and WASM target
- [x] **Add a Makefile** with common commands (dev, build, check, migrate, fmt, lint, test, deploy)
- [x] **Dependency cleanup:** Audit dependencies — only 4 crates (worker, serde, serde_json, uuid)
- [x] **CI Pipeline:** GitHub Actions workflow for cargo fmt, clippy, check, and tests

### 1.2 Input Validation

- [x] **Name validation:** Minimum/maximum length (1–200), reject empty strings
- [x] **Email validation:** Basic format check on CreateOwnerRequest / UpdateOwnerRequest
- [x] **Content validation:** Message content must not be empty, max 10,000 chars
- [x] **Title validation:** Chat title max length (500 chars)
- [x] **UUID format check:** Validate agent_id, owner_id, sender_id look like UUID v4
- [x] **Sender type enum:** sender_type validated against "agent" or "owner" enum
- [x] **Consolidated validation module:** Created `src/validation.rs` with `Validator` trait + field validators

### 1.3 Better Error Handling & Error Types

- [x] **Create `AppError` enum:** 5 variants (NotFound, BadRequest, Validation, Database, Internal)
- [x] **Implement `into_response()` for `AppError`** to unify error responses
- [x] **Add structured error codes** (ERR_NOT_FOUND, ERR_VALIDATION, ERR_DATABASE, etc.)
- [x] **Log all internal errors** with `console_error!` for Internal and Database variants
- [x] **Implement `From<worker::Error>`** so `?` operator works with AppError

### 1.4 Code Quality & Architecture

- [x] **Add `rustfmt` configuration** (`.rustfmt.toml`) — 100-char lines, module imports
- [x] **Add clippy linting** — configured for WASM target with `-D warnings`
- [x] **Separate the router into its own module** (`src/router.rs`) — lib.rs is 12 lines
- [x] **Extract constants:** 10 shared constants in `prelude.rs` (DEFAULT_CHAT_TITLE, MAX_*_LENGTH, etc.)
- [x] **Add `prelude.rs` pattern** for commonly-used imports
- [x] **CI Pipeline:** GitHub Actions with check+lint and test jobs

### 1.5 Testing Infrastructure

- [x] **Unit tests for models:** 15 tests — SenderType (4), AppError (6), API responses (5)
- [x] **Unit tests for validation:** 16 tests — Name (5), Email (6), Content (4), UUID (6), Validator impls (7)
- [x] **Total:** 31 unit tests across 2 modules
- [x] **CI test job:** Verifies tests compile for WASM target

---

## Phase 2: API Enhancements

### 2.1 Pagination (High Impact)

- [ ] **Add query parameter support** (`limit`, `offset`) to list handlers
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
- [ ] **Set default limit** (50) and enforce max limit (1,000) — constants already defined in prelude.rs
- [ ] **Add `PaginationParams` struct** to `models.rs`

### 2.2 Filtering & Search

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

- [ ] **Sort direction:** Add `sort_order` param (`asc` / `desc`, default to `desc` for most)
- [ ] **Sort by field:** Add `sort_by` param (`created_at`, `updated_at`, `name`, `email`, `title`)
- [ ] **Whitelist sort fields** to prevent SQL injection via column names
- [ ] **Add `SortParams` struct** to `models.rs`

### 2.4 Endpoint Expansion

- [ ] **`GET /api/agents/:id/chats`** — List all chats for a specific agent
- [ ] **`GET /api/owners/:id/chats`** — List all chats for a specific owner
- [ ] **`GET /api/agents/:id/owner`** — Get the owner of an agent (if any)
- [ ] **`GET /api/owners/:id/agents`** — List all agents owned by an owner
- [ ] **`GET /api/agents/:id/messages`** — Get all messages sent by a specific agent across all chats
- [ ] **`PATCH /api/agents/:id`** — Add partial update support (currently `PUT` works like PATCH)
- [ ] **`POST /api/agents/batch`** — Batch create agents

### 2.5 Response Improvements

- [ ] **Include related resource summaries** — e.g., include agent name and owner name in chat responses
- [ ] **Add `ETag` / `Last-Modified` headers** for caching
- [ ] **Add `Cache-Control` headers** for cacheable responses (list endpoints)
- [ ] **Compress responses** with Gzip/Brotli (Cloudflare Workers does this automatically at the edge)
- [ ] **Add `X-Request-Id`** header to all responses
- [ ] **Add `X-Response-Time`** header for performance monitoring

---

## Phase 3: Security & Authentication

### 3.1 Authentication (Critical for Production)

- [ ] **JWT authentication** using `jsonwebtoken` crate
- [ ] **API key authentication** as an alternative to JWT
- [ ] **Auth middleware:** Create a reusable auth layer
- [ ] **Protected routes:** Require auth for all mutating endpoints
- [ ] **Public routes:** Keep `GET /api/health` and `OPTIONS /*` open
- [ ] **Owner-only actions:** Verify authenticated owner can only modify their own resources
- [ ] **Agent auth:** Support agents authenticating to send messages on their own behalf

### 3.2 Authorization & Resource Ownership

- [ ] **Ownership model:** Each resource is owned by a user (owner or agent)
- [ ] **Access control middleware**
- [ ] **Admin role:** Super-admin users who can access all resources
- [ ] **Permission table:** Add a `permissions` table for fine-grained access control

### 3.3 Rate Limiting

- [ ] **Rate limit by IP** using Cloudflare's built-in rate limiting
- [ ] **Rate limit by API key** — track request counts per key in D1
- [ ] **Rate limit tiers:** Different limits for different endpoint types
- [ ] **Rate limit headers** in responses (`X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`)
- [ ] **429 response** when rate limited

### 3.4 Input Sanitization

- [ ] **SQL injection:** Already prevented through D1 prepared statements ✅
- [ ] **XSS sanitization:** Sanitize message content before storing
- [ ] **Size limits:** Enforce maximum sizes for all string fields ✅
- [ ] **Whitespace trimming:** Auto-trim leading/trailing whitespace on names, emails, titles ✅

---

## Phase 4: Real-time & Advanced Features

### 4.1 WebSocket Support (Real-time Chat)

- [ ] **Create Durable Object** for managing WebSocket connections per chat
- [ ] **WebSocket upgrade endpoint:** `GET /api/chats/:id/ws`
- [ ] **Broadcast messages:** When a message is sent, broadcast to all connected clients
- [ ] **Connection state management:** Handle connect, disconnect, reconnect
- [ ] **Online presence:** Show which users/agents are currently online
- [ ] **Typing indicators:** Broadcast typing events (not persisted to DB)
- [ ] **Read receipts:** Track and expose which messages have been read

### 4.2 Server-Sent Events (SSE)

- [ ] **SSE endpoint:** `GET /api/chats/:id/events` — stream new messages as they arrive
- [ ] **Auto-reconnect support** using standard SSE protocol

### 4.3 Notifications & Webhooks

- [ ] **Webhook registration:** `POST /api/webhooks`
- [ ] **Event types:** `message.created`, `chat.created`, `agent.created`, etc.
- [ ] **Webhook delivery:** Queue and deliver webhooks with retry logic
- [ ] **Webhook signatures:** Sign payloads with HMAC-SHA256 for verification
- [ ] **Email notifications** — Send email to owner when agent sends a message

### 4.4 Message Features

- [ ] **Message editing:** `PUT /api/chats/:id/messages/:msg_id`
- [ ] **Message deletion:** `DELETE /api/chats/:id/messages/:msg_id`
- [ ] **Reply threads:** Support replies to specific messages (`parent_message_id` field)
- [ ] **Message reactions:** Emoji reactions on messages (new `reactions` table)
- [ ] **File attachments:** Integrate with Cloudflare R2 for file uploads
- [ ] **Message types:** Extend beyond plain text — support markdown, code blocks, rich text
- [ ] **Streaming responses:** Support streaming message content (useful for AI agent responses)

---

## Phase 5: Observability & Production Readiness

### 5.1 Logging & Monitoring

- [ ] **Structured JSON logging** using the `tracing` crate
- [ ] **Request tracing:** Trace a request through all layers
- [ ] **Error tracking:** Collect and report errors
- [ ] **Metrics endpoint:** `GET /api/metrics` with Prometheus-style metrics
- [ ] **Health check** → add more details: database connectivity, latency percentiles
- [ ] **Setup Sentry** or similar error tracking integration

### 5.2 Performance Optimization

- [ ] **Query optimization:** Add `EXPLAIN QUERY PLAN` analysis on slow queries
- [ ] **D1 query batching:** Batch multiple queries where possible
- [ ] **Response compression** (handled by Cloudflare Workers at the edge)
- [ ] **Caching layer:** Add a cache (Cloudflare KV) for frequently accessed resources
- [ ] **Cache invalidation:** Invalidate cache entries on resource updates
- [ ] **Pagination** — critical for performance at scale (Phase 2)
- [ ] **N+1 query prevention:** When returning lists, avoid querying for each item individually
- [ ] **Cold start optimization:** Reduce WASM binary size further

### 5.3 Backup & Data Management

- [ ] **Automated D1 backups** — Cloudflare D1 has built-in backups via the dashboard
- [ ] **Export endpoint:** `GET /api/export/chats/:id` — export chat as JSON
- [ ] **Export all data:** `GET /api/export/user/:owner_id` — export all user data (GDPR compliance)
- [ ] **Data retention policy:** Auto-delete messages older than N days
- [ ] **Soft delete** — mark resources as deleted instead of actually deleting them
- [ ] **Audit log:** Track all mutations with who made them and when

---

## Phase 6: Ecosystem & Extensions

### 6.1 Client SDKs

- [ ] **TypeScript / JavaScript SDK** with full type definitions
- [ ] **Python SDK** — for AI agent integrations
- [ ] **OpenAPI / Swagger spec:** Auto-generate from Rust types
- [ ] **API documentation page:** Deploy a Swagger UI or Redoc page

### 6.2 Frontend Application

- [ ] **React dashboard app** with chat list sidebar, message view, composer
- [ ] **WebSocket integration** for real-time updates
- [ ] **File upload UI** with drag-and-drop

### 6.3 Integrations

- [ ] **Slack integration:** Bridge chat conversations to Slack channels
- [ ] **Discord integration:** Bridge to Discord
- [ ] **Email integration:** Send/receive messages via email
- [ ] **Webhook triggers** — allow external services to send messages via webhooks
- [ ] **AI agent integration** — connect agents to LLM APIs (OpenAI, Anthropic, etc.)

### 6.4 Multi-Tenancy

- [ ] **Organization model:** Add `organizations` table
- [ ] **Team management:** Multiple owners per organization
- [ ] **Role-based access control:** Admin, member, viewer roles per organization
- [ ] **Isolation:** Data is separated by organization (add `org_id` to all tables)
- [ ] **Billing / usage tracking** — track API usage per organization
- [ ] **Custom domains** — support custom domains per organization

---

## Version Roadmap

| Version | Focus | Key Deliverables | Status |
|---------|-------|------------------|--------|
| **v0.1.0** | MVP | Basic CRUD API, D1 schema, 17 endpoints | ✅ Complete |
| **v0.2.0** | Foundation | Input validation, AppError, router separation, CI, 31 tests, Makefile, docs | ✅ Complete |
| **v0.3.0** | Pagination & Filtering | Paginated list endpoints, query param filtering, sort support | ⬜ Upcoming |
| **v1.0.0** | Production | JWT auth, API keys, rate limiting, observability | ⬜ Planned |
| **v1.1.0** | Real-time | WebSocket support (Durable Objects) | ⬜ Future |
| **v1.2.0** | Rich messages | Attachments, reactions, message editing/deletion | ⬜ Future |
| **v1.3.0** | Integrations | Webhooks, SDKs, Slack/Discord bridges | ⬜ Future |
| **v2.0.0** | Multi-tenant | Organizations, team management, billing | ⬜ Future |

---

## Detailed Task Breakdown: v0.3.0 Sprint (Pagination & Filtering)

### Sprint Goal
Deliver pagination and filtering for all list endpoints, making the API suitable for production use at moderate scale.

### Sprint Backlog

| Task | Est. Effort | Dependencies | Status |
|------|-------------|--------------|--------|
| Add `PaginationParams` struct to models.rs | 30m | None | ⬜ |
| Parse `limit` and `offset` query params in handlers | 1h | PaginationParams | ⬜ |
| Update all list DB queries to use LIMIT/OFFSET | 3h | PaginationParams | ⬜ |
| Update response format with pagination metadata | 1h | DB updates | ⬜ |
| Add `?owner_id=` filter to GET /api/agents | 2h | Pagination | ⬜ |
| Add `?agent_id=` and `?owner_id=` filter to GET /api/chats | 3h | Pagination | ⬜ |
| Write integration tests for pagination & filtering | 4h | All above | ⬜ |

**Total estimated effort:** ~14.5 hours

### Acceptance Criteria
- [ ] All list endpoints accept `limit` and `offset` query params
- [ ] Default limit of 50, max limit of 1,000 enforced
- [ ] Response includes `pagination` object with `limit`, `offset`, `total`, `has_more`
- [ ] `GET /api/agents` supports `?owner_id=<uuid>` filter
- [ ] `GET /api/chats` supports `?agent_id=<uuid>&owner_id=<uuid>` filter
- [ ] Invalid pagination params return 400 with descriptive error

---

## Decision Log

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|------------------------|
| 2025-01 | Use UUID v4 as primary keys | Works well with D1, no auto-increment issues in distributed systems | Auto-increment integers, ULID, CUID |
| 2025-01 | Use `datetime('now')` in SQL rather than Rust timers | WASM has limited time support, D1 handles timezone consistently | `chrono` crate, `js_sys::Date` |
| 2025-01 | All timestamps stored as TEXT in ISO format | D1 returns TEXT from `datetime()`, simplifies parsing | INTEGER Unix timestamps |
| 2025-01 | Flatten `ChatWithMessages` with `#[serde(flatten)]` | Cleaner API response — keeps `chat` fields at top level alongside `messages` | Nested `{ chat: {...}, messages: [...] }` |
| 2025-01 | Single `sender_type` field on messages | Simple and extensible — both agent and owner use the same messages table | Separate tables for agent vs owner messages |
| 2025-01 | Extract router into separate module | Keeps lib.rs minimal (12 lines), improves testability and readability | All routes in lib.rs |
| 2025-01 | Use `prelude.rs` for shared imports | Reduces repetitive imports across modules, centralizes constants | Import from each module individually |
| 2025-01 | `AppError::into_response()` returns `Result<Response>` | Enables `?` in handlers and clean error propagation | Returning `Response` directly without Result |

---

## Technical Debt Register

| Issue | Severity | Created | Status |
|-------|----------|---------|--------|
| No pagination on list endpoints | Medium | 2025-01 | Open — Phase 2 |
| No authentication / authorization | High | 2025-01 | Open — Phase 3 |
| Email validation is basic (not RFC 5322) | Low | 2025-01 | Open — acceptable for MVP |
| No request ID tracking in responses | Low | 2025-01 | Open — Phase 2 |
| No integration tests for API endpoints | Medium | 2025-01 | Open — Phase 3 |

---

*This roadmap is a living document. Update it as priorities shift and new ideas emerge.*
