# TUI Guide — `h-cli`

This document provides a detailed overview of the User Interface and the technical implementation of the `h-cli` terminal application.

## Screen Overview

### 1. Profile Selection (`SelectOwner`)
The first screen you see when launching the app for the first time (or after logging out).
- Lists all registered **Owners** from the backend.
- Selecting a profile persists its ID in your local configuration.

### 2. Conversation List (`ChatList`)
Displays all active chats for the currently selected owner.
- Shows the chat title and the associated agent's name.
- Highlights the currently selected chat for easy navigation.
- Automatically refreshes in the background every 2 seconds.

### 3. Chat Room (`ChatRoom`)
The main messaging interface.
- **Message Feed**: Displays the history of the conversation.
    - Messages from "Me" are highlighted in **Cyan**.
    - Messages from agents are highlighted in **Yellow** with their name.
- **Input Area**: A persistent input field at the bottom for typing your messages.
- **Auto-Update**: Fetches new messages automatically while the room is open.

## Technical Details

### Architecture
`h-cli` is built using a "loop-based" state machine architecture common in game development and TUI applications.

1.  **Event Loop**: Listens for terminal events (keypresses, window resizing) using `crossterm`.
2.  **State Management**: The `TuiApp` struct maintains the current screen, cached data, and input buffer.
3.  **Rendering**: The `ui` function (using `ratatui`) maps the current state to terminal widgets on every tick.
4.  **Async Integration**: Uses `tokio` to perform non-blocking API calls to the Cloudflare Worker backend.

### Background Polling
To provide a "real-time" experience without WebSockets, `h-cli` implements a smart polling mechanism:
- While on the **Chat List**, it polls the list of chats.
- While inside a **Chat Room**, it polls the specific chat's message history.
- This ensures that you see new agent responses without having to manually refresh the view.

### Code Reuse
`h-cli` is a separate crate but is part of the project workspace. It avoids duplication by depending on the [chat-cli](../cli) library for:
- API Client (`Client`)
- Backend Data Models (`Agent`, `Owner`, `Chat`, `Message`)
- Endpoint Configuration (`Config`)
