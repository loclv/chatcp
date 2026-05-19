.PHONY: dev build check migrate fmt lint test clean deploy

# ─── Development ──────────────────────────────────────────────────────────────

dev:
	npm run dev

build:
	cargo build --target wasm32-unknown-unknown --release

check:
	cargo check --target wasm32-unknown-unknown

# ─── Database ─────────────────────────────────────────────────────────────────

migrate:
	npx wrangler d1 migrations apply chat-app-db --local

migrate-remote:
	npx wrangler d1 migrations apply chat-app-db --remote

# ─── Code Quality ─────────────────────────────────────────────────────────────

fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

# Backend lint (WASM target — excludes CLI)
lint-backend:
	cargo clippy --target wasm32-unknown-unknown -- -D warnings

# CLI lint (native target)
lint-cli:
	cd cli && cargo clippy -- -D warnings

# Human CLI lint (native target)
lint-h-cli:
	cd h-cli && cargo clippy -- -D warnings

# Lint all
lint: lint-backend lint-cli lint-h-cli

# ─── Testing ──────────────────────────────────────────────────────────────────

test:
	cargo test --workspace --exclude chat-app-backend

test-backend:
	cargo test

test-cli:
	cd cli && cargo test

test-h-cli:
	cd h-cli && cargo test

test-check:
	cargo check --tests --target wasm32-unknown-unknown

# ─── Cleanup ──────────────────────────────────────────────────────────────────

clean:
	cargo clean
	rm -rf build/ dist/ .wrangler/

# ─── Deployment ───────────────────────────────────────────────────────────────

deploy:
	npm run deploy
