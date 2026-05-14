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

lint:
	cargo clippy --target wasm32-unknown-unknown -- -D warnings

# ─── Testing ──────────────────────────────────────────────────────────────────

test:
	cargo test

test-check:
	cargo check --tests --target wasm32-unknown-unknown

# ─── Cleanup ──────────────────────────────────────────────────────────────────

clean:
	cargo clean
	rm -rf build/ dist/ .wrangler/

# ─── Deployment ───────────────────────────────────────────────────────────────

deploy:
	npm run deploy
