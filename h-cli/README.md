# Human Chat CLI (`h-cli`)

A beautiful, human-centric Terminal User Interface (TUI) for the [Chat App Backend](../README.md). Built with **Rust**, **Ratatui**, and **Crossterm**.

Unlike the management-focused `chat-cli`, this application is designed specifically for humans to have seamless conversations with their agents.

## Features

- **Polished TUI**: Multi-panel interface with clear status indicators and vibrant colors.
- **Persistent Identity**: Remembers your selected profile, so you can just jump in and chat.
- **Auto-Refresh**: Background polling ensures agent messages appear automatically without manual refreshes.
- **Code Reuse**: Leverages the core `chat-cli` library for robust API interaction.

## Quick Start

1.  **Ensure the backend is running**:
    ```bash
    cd .. && make dev
    ```

2.  **Launch the TUI**:
    ```bash
    cd h-cli
    cargo run
    ```

## Navigation & Keys

| Key | Action |
|---|---|
| **Arrow Up/Down** | Navigate through profiles or chats |
| **Enter** | Select profile / Open chat / Send message |
| **Esc** | Go back to chat list |
| **l** | Logout (return to profile selection) |
| **q** | Quit application |

## Configuration

The application stores your current session (Owner ID and API URL) in a local configuration file using the `confy` crate. 

- **Linux**: `~/.config/chat-app/human-session.toml`
- **macOS**: `~/Library/Application Support/chat-app/human-session.toml`
- **Windows**: `%AppData%\chat-app\config\human-session.toml`

## Documentation

For a detailed guide on the TUI screens and internal architecture, see the [TUI Guide](./docs/TUI_GUIDE.md).
