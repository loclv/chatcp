# Implementation Notes - Phase 3: Security & Authentication

## Initial Assessment
- The backend authentication (JWT & API Keys) and ownership checking logic is already written in `src/auth.rs` and `src/handlers.rs` but is uncommitted and untested.
- The unit tests do not cover JWT token generation and verification because `Date::now()` from the `worker` crate relies on WASM bindings.
- The CLI crate (`cli/src/models.rs`, `cli/src/client.rs`, `cli/src/main.rs`) does not have the parity fields or authentication support, violating `cli.md`'s rule on parity.

## Decisions & Design
1. **Conditional compilation for time**: We will use conditional compilation (`#[cfg(target_arch = "wasm32")]` vs `#[cfg(not(target_arch = "wasm32"))]`) to retrieve time for JWT. This lets `cargo test` run successfully in native environments.
2. **CLI Parity**: Update CLI structures and Client to automatically read and attach `CHAT_API_KEY` and `CHAT_JWT_TOKEN` environment variables as `Authorization` headers.
3. **CLI Owner Creation**: Add a `--password` parameter to `owner create` subcommand.
