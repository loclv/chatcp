---
trigger: always_on
---

# CLI Client Rules

Guidelines for developing and using the Rust terminal client.

## CLI Development
- **Crate**: Located in the `cli/` directory.
- **Argument Parsing**: Use `clap` with the derive macro.
- **Modes**:
    - **Quick Commands**: For one-off operations (e.g., `list agents`).
    - **Interactive REPL**: For immersive chat. Ensure the REPL (`repl.rs`) provides clear feedback and a smooth UX.
- **Parity**: Keep the CLI models (`cli/src/models.rs`) in sync with the backend models (`src/models.rs`).
- **Pretty Printing**: Use `display.rs` for tables, colors, and consistent formatting.

## CLI Usage (for Agents)
- When testing the API, prefer using the CLI over raw `curl` for better readability and to verify the CLI's correctness.
- Backend URL defaults to `http://localhost:8787`. Override with `CHAT_API_URL` if needed.
