# Development Guide

> **Project:** Chat App Backend — Rust + Cloudflare Workers + D1  
> **Version:** v0.2.0

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Project Setup](#project-setup)
- [Development Workflow](#development-workflow)
- [Tooling Configuration](#tooling-configuration)
- [Makefile Reference](#makefile-reference)
- [CI/CD Pipeline](#cicd-pipeline)
- [Coding Guidelines](#coding-guidelines)
- [Adding New Endpoints](#adding-new-endpoints)
- [Database Migrations](#database-migrations)
- [Testing](#testing)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Required

| Tool | Version | Installation |
|---|---|---|
| Rust | stable (2021 edition) | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| wasm32-unknown-unknown | — | `rustup target add wasm32-unknown-unknown` |
| Node.js | ≥ 18 | [nodejs.org](https://nodejs.org/) or use `nvm` |
| npm | ≥ 9 | Comes with Node.js |

### Cloudflare Account

Required for D1 database and deployment:

1. [Sign up for Cloudflare](https://dash.cloudflare.com/sign-up) (free tier works)
2. Install wrangler: `npm install` in the project root (installed as dev dependency)
3. Run `npx wrangler login` to authenticate

---

## Project Setup

```bash
# Clone the repository
git clone <repo-url> chat-app-backend
cd chat-app-backend

# Install Node.js dependencies (wrangler)
npm install

# Create the D1 database
npx wrangler d1 create chat-app-db

# Copy the database_id from the output into wrangler.toml
# Then apply migrations:
make migrate

# Start development server
make dev
```

---

## Development Workflow

### The inner loop

```
1. Edit Rust source files
2. Run make check   (fast: checks compilation for WASM target)
3. Run make test    (fast: runs unit tests natively)
4. Run make lint    (checks code quality)
5. Run make fmt     (auto-formats code)
6. Run make dev     (starts local server for manual testing)
```

### Common commands

```bash
# Fast compilation check (~5-10 seconds)
make check

# Run all tests (~1 second)
make test

# Full quality check (format + lint + compilation)
make fmt-check && make lint && make check
```

---

## Tooling Configuration

### rust-toolchain.toml

Pins the Rust toolchain and ensures the WASM target is always available:

```toml
[toolchain]
channel = "stable"
targets = ["wasm32-unknown-unknown"]
```

### .rustfmt.toml

```toml
max_width = 100
tab_spaces = 4
imports_granularity = "Module"
reorder_imports = true
format_code_in_doc_comments = true
wrap_comments = true
use_field_init_shorthand = true
force_explicit_abi = true
```

### .gitignore

Ignores: Rust build artifacts (`target/`, `Cargo.lock`), wrangler builds (`build/`, `dist/`, `.wrangler/`, `*.wasm`), Node modules, environment files, IDE files.

---

## Makefile Reference

| Command | What it does | When to use |
|---|---|---|
| `make dev` | Start wrangler dev server | Local development |
| `make build` | Build for WASM release | Before deploying |
| `make check` | Check WASM compilation | After writing code |
| `make fmt` | Format all Rust code | Before committing |
| `make fmt-check` | Verify formatting (CI) | CI pipeline |
| `make lint` | Run clippy (WASM, pedantic) | Before committing |
| `make test` | Run unit tests (native) | After writing code |
| `make test-check` | Verify tests compile (WASM) | CI pipeline |
| `make migrate` | Apply D1 migrations locally | After schema changes |
| `make migrate-remote` | Apply D1 migrations to prod | Before deployment |
| `make deploy` | Build + deploy to Cloudflare | Release |
| `make clean` | Remove build artifacts | When things get stale |

---

## CI/CD Pipeline

The GitHub Actions workflow (`.github/workflows/ci.yml`) runs on every push/PR to `main`/`master`:

### Job 1: Check & Lint

```yaml
steps:
  - cargo fmt --all -- --check       # Formatting
  - cargo clippy --target wasm32-unknown-unknown -- -D warnings  # Linting
  - cargo check --target wasm32-unknown-unknown  # Compilation
```

### Job 2: Test (depends on Check)

```yaml
steps:
  - cargo check --tests --target wasm32-unknown-unknown  # Tests compile on WASM
```

**Note:** Tests are compiled-checked for WASM (to catch `#[cfg(test)]` issues) but executed natively via `cargo test` for speed.

---

## Coding Guidelines

### Module Organization

```
src/
├── lib.rs              # Entry point (router creation)
├── router.rs           # Route definitions only
├── handlers.rs         # HTTP layer (parsing, CORS)
├── validation.rs       # Input validation
├── models.rs           # Data structures + AppError + tests
├── db.rs               # Database operations
└── prelude.rs          # Common imports + constants
```

### Import Convention

Always use `use crate::prelude::*;` for commonly-used types:

```rust
// In handlers.rs, db.rs, validation.rs:
use crate::prelude::*;
```

Only import directly from specific modules for new types:

```rust
use crate::validation::Validator;  // Only if needed
use crate::db;                     // For calling DB functions
```

### Naming Conventions

| Item | Convention | Example |
|---|---|---|
| Types/Structs | PascalCase | `CreateAgentRequest` |
| Enum variants | PascalCase | `AppError::NotFound` |
| Functions | snake_case | `validate_uuid` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_NAME_LENGTH` |
| Modules | snake_case | `validation.rs` |
| Variables | snake_case | `owner_id` |
| Type params | single uppercase | `T` in `ApiResponse<T>` |

### Error Handling Pattern

```rust
// Returning errors
AppError::NotFound(format!("Agent '{}' not found", id)).into_response()
AppError::Validation("name must not be empty".to_string()).into_response()
AppError::Database(format!("Failed to create: {}", e)).into_response()

// With the ? operator (converts worker::Error → AppError::Internal)
let existing = get_agent_by_id(d1, id).await?;
```

### Validation Pattern

Every new request type that accepts user input must:

1. Implement the `Validator` trait
2. Call existing field validators (`validate_name`, `validate_uuid`, etc.)
3. Be validated via `parse_and_validate::<T>()` in the handler

### Adding Comments

- Use `//!` for module-level documentation
- Use `///` for struct/enum/function documentation
- Use `// ─── Section separators ───` for grouping related code
- Avoid obvious comments (`// increment i`)

---

## Adding New Endpoints

### Step-by-step

1. **Define request/response types** in `models.rs`
2. **Implement `Validator`** for the request type in `validation.rs`
3. **Write DB function** in `db.rs`
4. **Write handler** in `handlers.rs` using `parse_and_validate`
5. **Register route** in `router.rs`
6. **Add tests** in the `#[cfg(test)]` module
7. **Document** in `docs/API_REFERENCE.md` and `README.md`

### Example: Adding a health endpoint (already done)

Since this is the simplest endpoint, here's what it looks like:

**router.rs** (route registration):
```rust
.get_async("/api/health", |_req, _ctx| async move {
    Response::from_json(&serde_json::json!({
        "status": "ok",
        "service": "chat-app-backend",
        "version": env!("CARGO_PKG_VERSION")
    }))
})
```

### Example: Adding a full CRUD endpoint (agents)

**models.rs** (request struct):
```rust
#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub owner_id: Option<String>,
}
```

**validation.rs** (validator):
```rust
impl Validator for CreateAgentRequest {
    fn validate(&self) -> Result<(), AppError> {
        validate_name(&self.name, "name")?;
        if let Some(desc) = &self.description {
            validate_content(desc, "description", MAX_DESCRIPTION_LENGTH)?;
        }
        // ...
        Ok(())
    }
}
```

**db.rs** (database operation):
```rust
pub async fn create_agent(d1: &D1Database, req: &CreateAgentRequest) -> Result<Response> {
    let id = generate_id();
    // ... prepare, bind, execute, respond
}
```

**handlers.rs** (HTTP handler):
```rust
pub async fn create_agent(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let body = parse_and_validate::<CreateAgentRequest>(&req)
        .map_err(|resp| with_cors(resp))?;
    let resp = db::create_agent(&d1, &body).await?;
    Ok(with_cors(resp))
}
```

**router.rs** (route):
```rust
.post_async("/api/agents", handlers::create_agent)
```

---

## Database Migrations

### Creating a new migration

```bash
npx wrangler d1 migrations create chat-app-db my_migration_name
```

This creates a new file like `migrations/0002_my_migration_name.sql`. Write your SQL changes there.

### Applying migrations

```bash
# Local (development)
make migrate

# Remote (production)
make migrate-remote
```

### Migration best practices

- **Never modify** an already-applied migration file
- Each migration should be **idempotent** where possible (use `IF NOT EXISTS`)
- Test migrations locally before applying to production
- Keep migrations small and focused

---

## Testing

### Running tests

```bash
# Run all tests (native target) - fast
make test

# Check tests compile for WASM
make test-check
```

### Writing tests

Tests are embedded inline in source files using `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        assert!(true);
    }
}
```

### Test coverage

| Module | Tests | What's tested |
|---|---|---|
| `models.rs` | 15 | SenderType enum, AppError variants, API response serialization, struct deserialization |
| `validation.rs` | 16 | Field validators (name, email, content, UUID), Validator impls for all request types |

### Writing good tests

- Test **success paths** (valid input produces expected output)
- Test **error paths** (invalid input produces expected error)
- Test **edge cases** (empty strings, max lengths, special characters)
- Use descriptive test names: `test_validate_email_no_at`, `test_app_error_not_found`
- Group related tests with comments

---

## Troubleshooting

### Rust doesn't compile

```bash
# Clear caches and retry
make clean
make check
```

### "target `wasm32-unknown-unknown` not installed"

```bash
rustup target add wasm32-unknown-unknown
```

### D1 database not found

```bash
# Check the database exists
npx wrangler d1 list

# If not, create it
npx wrangler d1 create chat-app-db

# Then update database_id in wrangler.toml
```

### "Worker exceeded size limit"

The release profile is optimized for size. If you still exceed the limit:

```bash
# Check current WASM binary size
ls -lh target/wasm32-unknown-unknown/release/*.wasm
```

Consider:
- Removing unnecessary dependencies
- Using `opt-level = "z"` instead of `"s"` (aggressive size optimization)
- Splitting into multiple workers

### Tests fail unexpectedly

```bash
# Run with verbose output
cargo test -- --nocapture
```

### Formatter or linter errors

```bash
# Fix formatting automatically
make fmt

# See lint warnings without treating them as errors
cargo clippy --target wasm32-unknown-unknown
```
