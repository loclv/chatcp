# Chat CLI

A Rust terminal client for the [Chat App Backend](../README.md). Chat with agents and humans directly from your terminal — supports both quick commands and an interactive REPL.

## Quick Start

```bash
# Make sure the backend is running first
cd .. && make dev

# In another terminal:
cd cli
cargo run -- health

# List resources
cargo run -- list agents
cargo run -- list owners
cargo run -- list chats

# Create resources
cargo run -- create owner --name "Alice" --email "alice@example.com"
cargo run -- create agent --name "CodeBot" --description "AI assistant"
cargo run -- create chat --agent-id <agent-uuid> --owner-id <owner-uuid> --title "My Chat"

# Send a message
cargo run -- send --chat-id <chat-uuid> --as-type owner --sender-id <owner-uuid> --text "Hello!"

# Start interactive mode
cargo run -- repl
```

## Usage

### Configuration

Set the backend URL via environment variable or the `--api-url` flag:

```bash
# Default: http://localhost:8787
export CHAT_API_URL=http://localhost:8787
cargo run -- health

# Or use flag:
cargo run -- --api-url https://my-worker.workers.dev health
```

### Commands

| Command | Description |
|---|---|
| `health` | Check if the backend is reachable |
| `repl` | Start interactive chat mode |
| `list agents` | List all agents |
| `list owners` | List all owners |
| `list chats` | List all chats |
| `get agent <id>` | Show agent details |
| `get owner <id>` | Show owner details |
| `get chat <id>` | Show chat with messages |
| `create agent --name <n> [--description <d>] [--owner-id <id>]` | Create agent |
| `create owner --name <n> --email <e>` | Create owner |
| `create chat --agent-id <id> --owner-id <id> [--title <t>]` | Create chat |
| `update agent --id <id> [--name <n>] [--description <d>] [--owner-id <id>]` | Update agent |
| `update owner --id <id> [--name <n>] [--email <e>]` | Update owner |
| `update chat --id <id> --title <t>` | Update chat title |
| `delete agent <id>` | Delete agent |
| `delete owner <id>` | Delete owner |
| `delete chat <id>` | Delete chat |
| `send --chat-id <id> --as-type <type> --sender-id <id> --text <msg>` | Send a message |
| `messages --chat-id <id>` | View messages in a chat |

### Interactive REPL

Start the interactive mode:

```bash
cargo run -- repl
```

In REPL mode you get an interactive shell:

```
📋> /help
```

**REPL Commands:**

| Command | Short | Description |
|---|---|---|
| `/list` | `/l` | List all chats |
| `/select <n>` | `/s <n>` | Select a chat by number |
| `/new` | `/n` | Create a new chat (wizard) |
| `/refresh` | `/r` | Refresh the chat list |
| `/agents` | `/a` | List all agents |
| `/owners` | `/o` | List all owners |
| `/help` | `/h` | Show help |
| `/quit` | `/q` | Exit interactive mode |

When a chat is selected, type any text to send it as a message.

**Example REPL session:**

```
📋> /list
✓ Found 3 chat(s)
┌─────┬──────────┬─────────────────────┬──────────┬──────────┬────────────┐
│  #  │    ID    │        Title        │  Agent   │  Owner   │  Updated   │
├─────┼──────────┼─────────────────────┼──────────┼──────────┼────────────┤
│  1  │ c1d2e3f4 │ Debug Help          │ a1b2c3d4 │ o1p2q3r4 │ 2025-01-15 │
│  2  │ a2b3c4d5 │ Rust Project        │ b2c3d4e5 │ o2p3q4r5 │ 2025-01-14 │
│  3  │ b3c4d5e6 │ Web Scraper Design  │ c3d4e5f6 │ o3p4q5r6 │ 2025-01-13 │
└─────┴──────────┴─────────────────────┴──────────┴──────────┴────────────┘

📋> /select 1
✓ Selected: Debug Help
Chat: Debug Help
  ID: c1d2e3f4-a5b6-7890-cdef-012345678901
  ...

💬 Debug Help> Can you help me debug this?
Who is sending this message?
  1. Me (Owner)
  2. Agent
  Choice [1]: 1
✓ Message sent!
  [Owner] o1p2q3r4-s5t6-7890-uvwx-yz1234567890 2025-01-15 10:31:00
  Can you help me debug this?
```

## Development

```bash
# Run the CLI
cargo run -- health

# Build for release
cargo build --release

# Check compilation
cargo check
```

## Dependencies

- **clap** — CLI argument parsing with derive macros
- **reqwest** — HTTP client with TLS support
- **tokio** — Async runtime
- **serde / serde_json** — JSON serialization/deserialization
- **colored** — Terminal text colors and styling
- **prettytable-rs** — ASCII table formatting
- **uuid** — UUID validation

## Project Structure

```
cli/
├── Cargo.toml         # Rust dependencies and metadata
├── README.md          # This file
└── src/
    ├── main.rs        # Entry point and CLI argument parsing
    ├── client.rs      # HTTP API client for the backend
    ├── config.rs      # Configuration (backend URL, env vars)
    ├── display.rs     # Pretty-printing and terminal formatting
    ├── models.rs      # Response types matching the backend API
    └── repl.rs        # Interactive REPL mode
```
