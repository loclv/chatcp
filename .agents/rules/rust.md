---
trigger: always_on
---

# Rust Best Practices

When working on the Rust backend or CLI, follow these guidelines:

## Code Quality
- **Formatting**: Always run `cargo fmt` before committing. Use the configuration in `.rustfmt.toml`.
- **Linting**: Use `cargo clippy`. For the backend, ensure it passes for the `wasm32-unknown-unknown` target:
  ```bash
  cargo clippy --target wasm32-unknown-unknown -- -D warnings
  ```
- **Structure**:
    - Backend: Keep `lib.rs` minimal. Use `router.rs`, `handlers.rs`, `validation.rs`, `models.rs`, and `db.rs`.
    - CLI: Maintain separation between `main.rs`, `client.rs`, `display.rs`, and `repl.rs`.

## Architecture
- **Prelude**: Use `use crate::prelude::*;` to import commonly used types and constants.
- **Constants**: Store all magic numbers, strings, and configuration defaults in `src/prelude.rs`.
- **Error Handling**: Use the `AppError` enum for structured errors. Use the `?` operator with `worker::Result` in the backend.

## Dependencies
- **WASM Compatibility**: Before adding a dependency to the backend, ensure it is compatible with `wasm32-unknown-unknown`. Avoid system calls, threading, and non-WASM compatible I/O.
- **CLI**: The CLI is a native binary and can use standard crates.

## Development Workflow
- Use the `Makefile` for common tasks:
    - `make check`: Verify WASM compilation.
    - `make test`: Run native unit tests.
    - `make dev`: Start local wrangler server.
    - `make migrate`: Apply local migrations.
