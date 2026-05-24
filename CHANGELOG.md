# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.3.0] - 2026-05-25

### Added
- **JWT & API Key Authentication**: Integrated secure, WASM-compatible cryptographic module (`src/auth.rs`) using SHA-256 with salt for owner passwords and JWT signing/verification.
- **Key Rotation**: Added support for rotating API keys for both owners (`/api/owners/:id/key`) and agents (`/api/agents/:id/key`).
- **Resource Ownership Checks**: Implemented strict validation for resource modification:
  - Owners can only update/delete agents, chats, and messages that belong to them.
  - Agents can only send messages to chats they are assigned to.
- **CLI Authentication Parity**:
  - Automatically reads and injects `Authorization` header (`ApiKey <KEY>` if `CHAT_API_KEY` is present, or `Bearer <TOKEN>` if `CHAT_JWT_TOKEN` is present) into all client HTTP requests.
  - Added CLI client methods for `/api/auth/login`, `/api/auth/me`, and key rotations.
  - Added an optional `--password` option for creating owners (`owner create`).
- **List Pagination, Filtering, and Sorting** (Sprint v0.3.0):
  - Supported `limit` and `offset` query parameters for pagination on list endpoints.
  - Added filtering by `owner_id` for agents, and by `agent_id` / `owner_id` for chats.
  - Added sorting by field (e.g., `created_at`, `updated_at`, `name`, `email`) and sort order (`asc` / `desc`).
  - Formatted lists with dedicated `PaginationMetadata` containing limits, offsets, totals, and `has_more` indicators.

### Changed
- Refactored CLI display module and test suite to keep mock structures (`Agent`, `Owner`) in sync with new database models.
- Conditionally compile timestamp generation in `src/auth.rs` using `std::time::SystemTime` on native targets and `worker::Date` on WASM target, allowing native host execution of unit tests for token verification.

### Fixed
- Handled CORS preflight and response headers consistently on all new authenticated and key rotation routes.
