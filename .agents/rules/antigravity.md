---
trigger: always_on
---

# Antigravity Rules

General guidelines for the Antigravity AI assistant when working on the Chat App project.

## Mission
Provide high-quality, robust, and well-tested code for the Chat App Backend and CLI. Maintain the project's focus on performance (Rust/WASM) and reliability (D1/Cloudflare).

## Key Directives
1. **Follow the Rules**: Adhere to all rules defined in `.agents/rules/*.md`.
2. **Be Thorough**: When implementing new features, ensure validation, error handling, and unit tests are included.
3. **Communicate Clearly**: Use artifacts for plans and walkthroughs. Keep implementation notes up to date.
4. **Log Everything**: Use `l-log` to maintain a history of changes.

## File Reference
- [rust.md](./rust.md): Rust-specific standards.
- [cloudflare.md](./cloudflare.md): Worker and D1 guidelines.
- [cli.md](./cli.md): CLI development rules.
- [process.md](./process.md): Agent process and documentation rules.
- [common.md](./common.md): Logging and shared requirements.
