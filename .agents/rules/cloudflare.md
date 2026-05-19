---
trigger: always_on
---

# Cloudflare Workers & D1 Rules

Guidelines for developing the serverless backend.

## Cloudflare Workers
- **Runtime**: The backend runs in a WASM environment. Use the `worker` crate (v0.4).
- **Entry Point**: `src/lib.rs` initializes the worker and dispatches to the router.
- **Routing**: Define all endpoints in `src/router.rs`. Use the `Router` provided by the `worker` crate.

## Cloudflare D1 (Database)
- **Queries**: All database interactions must be in `src/db.rs`.
- **Prepared Statements**: Always use prepared statements to prevent SQL injection.
- **Migrations**: Database schema changes must be added as SQL files in the `migrations/` directory. Use `make migrate` to apply locally.
- **Timestamps**: Use SQL `datetime('now')` for consistency. Timestamps are stored as `TEXT` in ISO format.

## Error Handling & Validation
- **Validation**: Every mutating request must be validated using the `Validator` trait in `src/validation.rs`.
- **AppError**: Map database and worker errors to `AppError`. Use `into_response()` to return structured JSON errors with correct HTTP status codes.
- **CORS**: Ensure CORS headers are handled correctly in `handlers.rs` and `router.rs` (preflight).
