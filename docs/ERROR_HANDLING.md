# Error Handling & Validation

> **Version:** v0.2.0 — Structured error handling with AppError enum

---

## Overview

The API uses a unified error handling system built around the `AppError` enum. Every error — whether from invalid input, missing resources, database failures, or unexpected conditions — is converted into a consistent JSON response with an HTTP status code, human-readable message, and machine-readable error code.

---

## AppError Enum

Defined in `src/models.rs`:

```rust
pub enum AppError {
    NotFound(String),      // Resource doesn't exist
    BadRequest(String),    // Malformed request (invalid JSON)
    Validation(String),    // Input validation failure
    Database(String),      // D1 query execution failure
    Internal(String),      // Unexpected server error
}
```

### Variant Details

| Variant | HTTP Status | Error Code | When It's Used | Logged? |
|---|---|---|---|---|
| `NotFound` | 404 | `ERR_NOT_FOUND` | Resource by ID not found | No |
| `BadRequest` | 400 | `ERR_BAD_REQUEST` | Invalid JSON, duplicate email | No |
| `Validation` | 400 | `ERR_VALIDATION` | Field validation failure | No |
| `Database` | 500 | `ERR_DATABASE` | D1 query error | Yes (`console_error!`) |
| `Internal` | 500 | `ERR_INTERNAL` | Unexpected errors | Yes (`console_error!`) |

---

## Error Response Format

All error responses follow the same JSON structure:

```json
{
  "success": false,
  "data": null,
  "error": "Human-readable error message describing the issue",
  "code": "ERR_VALIDATION"
}
```

### Real Examples

**404 — Agent not found:**
```json
{
  "success": false,
  "data": null,
  "error": "Agent 'abc-123' not found",
  "code": "ERR_NOT_FOUND"
}
```

**400 — Validation:**
```json
{
  "success": false,
  "data": null,
  "error": "name must not be empty",
  "code": "ERR_VALIDATION"
}
```

**400 — Bad request:**
```json
{
  "success": false,
  "data": null,
  "error": "Invalid request body: missing field `name` at line 1 column 42",
  "code": "ERR_BAD_REQUEST"
}
```

**400 — Duplicate email:**
```json
{
  "success": false,
  "data": null,
  "error": "An owner with this email already exists",
  "code": "ERR_BAD_REQUEST"
}
```

**500 — Database error (logged to Cloudflare console):**
```json
{
  "success": false,
  "data": null,
  "error": "Failed to create agent: D1_ERROR: constraint failed",
  "code": "ERR_DATABASE"
}
```

---

## Error Flow

```
Request arrives
    │
    ▼
Parse JSON body ──── Err ──▶ AppError::BadRequest ──▶ 400 response
    │
    OK
    ▼
Validate input ────── Err ──▶ AppError::Validation ──▶ 400 response
    │
    OK
    ▼
Execute D1 query ──── Err ──▶ AppError::Database ────▶ 500 response (logged)
    │
    OK
    ▼
Resource exists? ──── No ──▶ AppError::NotFound ─────▶ 404 response
    │
    Yes
    ▼
Return success response (200/201)
```

---

## Conversion Traits

`AppError` implements several `From` traits, enabling idiomatic Rust error handling with the `?` operator:

### From<worker::Error>

Converts `worker::Error` into `AppError::Internal`, so D1 operations can use `?`:

```rust
// Instead of:
let result = d1.prepare("SELECT ...").bind(&[...])?.run().await.map_err(|e| AppError::Database(e.to_string()))?;

// You can write:
let result = d1.prepare("SELECT ...").bind(&[...])?.run().await?;
// Error is automatically converted to AppError::Internal
```

### From<String> and From<&str>

```rust
let err: AppError = "something went wrong".into();
// Results in AppError::Internal("something went wrong")
```

### into_response()

The `into_response()` method on `AppError` converts the error into a `worker::Result<Response>`:

```rust
// In handler code:
if not_found {
    return AppError::NotFound(format!("Chat '{}' not found", id)).into_response();
}
```

---

## Input Validation

### Validator Trait

Defined in `src/validation.rs`:

```rust
pub trait Validator {
    fn validate(&self) -> Result<(), AppError>;
}
```

Implemented for all 7 request types. The `parse_and_validate` helper in `handlers.rs` uses it automatically:

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

### Field Validators

#### validate_name(name, field)

| Rule | Value |
|---|---|
| Minimum length | 1 character (after trimming) |
| Maximum length | 200 characters |
| Trimming | Leading/trailing whitespace is trimmed before check |
| Error on | Empty, whitespace-only, or too long |

**Error examples:**
- `"name must not be empty"`
- `"name must be at most 200 characters (got 250)"`
- `"name must be at least 1 character(s) (got 0)"`

---

#### validate_email(email, field)

| Rule | Value |
|---|---|
| Maximum length | 320 characters |
| `@` presence | Exactly one `@` character |
| Local part | Non-empty (before `@`) |
| Domain part | Contains at least one `.` after `@` |

**Note:** This is a **basic format check**, not a full RFC 5322 validation. It catches obvious mistakes (missing `@`, no TLD) but won't catch all edge cases.

**Error examples:**
- `"email must not be empty"`
- `"email must be a valid email address"`
- `"email must be at most 320 characters (got 350)"`

---

#### validate_content(content, field, max_len)

| Rule | Value |
|---|---|
| Minimum length | 1 character (after trimming) |
| Maximum length | Configurable via `max_len` parameter |
| Trimming | Leading/trailing whitespace is trimmed before check |

**Constants:**
| Context | max_len |
|---|---|
| Message content | `MAX_CONTENT_LENGTH` = 10,000 |
| Chat title | `MAX_TITLE_LENGTH` = 500 |
| Agent description | `MAX_DESCRIPTION_LENGTH` = 2,000 |

**Error examples:**
- `"content must not be empty"`
- `"content must be at most 10000 characters (got 15000)"`

---

#### validate_uuid(id, field)

| Rule | Description |
|---|---|
| Length | Exactly 36 characters |
| Dashes | At positions 8, 13, 18, 23 |
| Version | Character at position 14 must be `4` (UUID v4) |
| Characters | All non-dash characters must be hexadecimal (0-9, a-f, A-F) |

**Note:** This validates the **format** of the UUID, not that it exists in the database. A well-formatted UUID for a non-existent resource will pass validation but receive a 404 from the database layer.

**Error examples:**
- `"agent_id must not be empty"`
- `"agent_id must be a valid UUID (36 characters, got 8)"`
- `"agent_id must be a UUID v4"` (when version nibble is not `4`)
- `"agent_id must be a valid UUID format"` (when dashes are missing)
- `"agent_id must be a valid UUID (invalid character at position 0)"`

---

## Validation by Endpoint

| Endpoint | Request Type | Validated Fields |
|---|---|---|
| `POST /api/agents` | `CreateAgentRequest` | `name`, `description` (optional), `owner_id` (optional) |
| `PUT /api/agents/:id` | `UpdateAgentRequest` | `name` (optional), `description` (optional), `owner_id` (optional) |
| `POST /api/owners` | `CreateOwnerRequest` | `name`, `email` |
| `PUT /api/owners/:id` | `UpdateOwnerRequest` | `name` (optional), `email` (optional) |
| `POST /api/chats` | `CreateChatRequest` | `agent_id`, `owner_id`, `title` (optional) |
| `PUT /api/chats/:id` | `UpdateChatRequest` | `title` (optional) |
| `POST /api/chats/:id/messages` | `SendMessageRequest` | `sender_type`, `sender_id`, `content` |

---

## Error Handling in Code

### Pattern: Simple not-found check

```rust
async fn get_agent(d1: &D1Database, id: &str) -> Result<Response> {
    match get_agent_by_id(d1, id).await {
        Ok(Some(agent)) => {
            let resp = ApiResponse::success(agent);
            Response::from_json(&resp)
        }
        Ok(None) => AppError::NotFound(format!("Agent '{}' not found", id)).into_response(),
        Err(e) => AppError::Database(e.to_string()).into_response(),
    }
}
```

### Pattern: Database operation with error mapping

```rust
let result = d1
    .prepare("INSERT INTO agents (...) VALUES (...)")
    .bind(&[...])?
    .run()
    .await;

match result {
    Ok(_) => { /* success path */ }
    Err(e) => {
        AppError::Database(format!("Failed to create agent: {}", e)).into_response()
    }
}
```

### Pattern: Special error handling (duplicate email)

```rust
Err(e) => {
    let msg = format!("{}", e);
    if msg.contains("UNIQUE") {
        return AppError::BadRequest("An owner with this email already exists".to_string())
            .into_response();
    }
    AppError::Database(format!("Failed to create owner: {}", e)).into_response()
}
```

### Pattern: Validation + handler combo

```rust
pub async fn create_agent(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let body = match parse_and_validate::<CreateAgentRequest>(&req) {
        Ok(b) => b,
        Err(resp) => return Ok(with_cors(resp)),
    };
    let resp = db::create_agent(&d1, &body).await?;
    Ok(with_cors(resp))
}
```

---

## Best Practices

When extending the API with new endpoints:

1. **Define a request struct** in `models.rs` with `#[derive(Deserialize)]`
2. **Implement the `Validator` trait** in `validation.rs` for that struct
3. **Use `parse_and_validate::<T>()`** in the handler to get validated input
4. **Return `AppError::Foo(msg).into_response()`** for error paths
5. **Log internal errors** with `console_error!()` (already done for `Internal` and `Database` variants)

---

## Technical Debt

- Email validation is basic — does not catch all edge cases per RFC 5322
- No request ID tracking in error responses (planned for future)
- No rate limiting error responses (planned for Phase 3)
