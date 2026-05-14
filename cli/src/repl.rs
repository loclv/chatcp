//! Interactive REPL mode for the chat CLI.
//!
//! Provides an interactive shell where users can:
//! - List and select chats
//! - View message history
//! - Send messages
//! - Create new chats on the fly

use std::io::{self, Write};

use colored::*;

use crate::client::Client;
use crate::display;
use crate::models::{Chat, ChatWithMessages};

// ─── Command Parsing (extracted for testability) ─────────────────────────────

/// Parsed REPL command — extracted for testability.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) enum ParsedCommand {
    Help,
    List,
    Select(Option<usize>),
    New,
    Refresh,
    Agents,
    Owners,
    Quit,
    Unknown(String),
}

/// Parse a raw input string into a `ParsedCommand`.
/// Returns `None` for empty input.
pub(crate) fn parse_command(input: &str) -> Option<ParsedCommand> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    if !trimmed.starts_with('/') {
        return None; // Not a command — it's a message to send
    }

    let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
    let cmd = parts[0].to_lowercase();
    let arg = parts.get(1).map(|s| s.trim().to_string());

    match cmd.as_str() {
        "/help" | "/h" | "/?" => Some(ParsedCommand::Help),
        "/list" | "/l" => Some(ParsedCommand::List),
        "/select" | "/s" => {
            let num = arg.as_ref().and_then(|s| s.parse::<usize>().ok());
            Some(ParsedCommand::Select(num))
        }
        "/new" | "/n" => Some(ParsedCommand::New),
        "/refresh" | "/r" => Some(ParsedCommand::Refresh),
        "/agents" | "/a" => Some(ParsedCommand::Agents),
        "/owners" | "/o" => Some(ParsedCommand::Owners),
        "/quit" | "/q" | "/exit" => Some(ParsedCommand::Quit),
        other => Some(ParsedCommand::Unknown(other.to_string())),
    }
}

// ─── State ───────────────────────────────────────────────────────────────────

/// State for the REPL session.
struct ReplState {
    /// Current chat being viewed (None = no chat selected).
    current_chat: Option<ChatWithMessages>,
    /// All available chats (cached for selection).
    chats: Vec<Chat>,
}

// ─── REPL Main Loop ─────────────────────────────────────────────────────────

/// Run the interactive REPL.
pub async fn run(client: &Client) {
    display::print_info(format!(
        "Connected to backend at {}",
        client.config().base_url
    ));

    let mut state = ReplState {
        current_chat: None,
        chats: Vec::new(),
    };

    // Refresh chat list on start
    refresh_chats(client, &mut state).await;

    println!();
    println!("{}", "╔══════════════════════════════════════════════════════════╗".bright_blue());
    println!("{}", "║           Chat App CLI — Interactive Mode              ║".bright_blue().bold());
    println!("{}", "╠══════════════════════════════════════════════════════════╣".bright_blue());
    println!("{}", "║  Commands:                                             ║".bright_blue());
    println!("{}", "║  /list         — List all chats                        ║".bright_blue());
    println!("{}", "║  /select <n>   — Select chat by number                 ║".bright_blue());
    println!("{}", "║  /new          — Create a new chat                     ║".bright_blue());
    println!("{}", "║  /refresh      — Refresh chat list                     ║".bright_blue());
    println!("{}", "║  /agents       — List agents                           ║".bright_blue());
    println!("{}", "║  /owners       — List owners                           ║".bright_blue());
    println!("{}", "║  /help         — Show this help                        ║".bright_blue());
    println!("{}", "║  /quit         — Exit interactive mode                 ║".bright_blue());
    println!("{}", "║                                                         ║".bright_blue());
    println!("{}", "║  Type any text to send it as a message in the          ║".bright_blue());
    println!("{}", "║  currently selected chat.                              ║".bright_blue());
    println!("{}", "╚══════════════════════════════════════════════════════════╝".bright_blue());
    println!();

    loop {
        // Show current chat context in prompt
        let prompt = match &state.current_chat {
            Some(chat) => format!(
                "{} {}> ",
                "\u{1f4ac}",
                chat.title.as_str().green().bold()
            ),
            None => format!("{}> ", "\u{1f4cb}"),
        };

        print!("{}", prompt);
        io::stdout().flush().ok();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() || input.trim().to_lowercase() == "/quit" {
            break;
        }

        let input = input.trim().to_string();
        if input.is_empty() {
            continue;
        }

        // Try parsing as a command
        if let Some(cmd) = parse_command(&input) {
            match cmd {
                ParsedCommand::Quit => break,
                _ => handle_command(client, &mut state, cmd).await,
            }
        } else if state.current_chat.is_some() {
            // Send as message in current chat
            send_as_message(client, &state, &input).await;
        } else {
            display::print_warning("No chat selected. Use /list then /select <n> to choose a chat.");
        }
    }

    display::print_info("Goodbye!");
}

/// Dispatch a parsed command to the appropriate handler.
async fn handle_command(client: &Client, state: &mut ReplState, cmd: ParsedCommand) {
    match cmd {
        ParsedCommand::Help => {
            print_help();
        }
        ParsedCommand::List => {
            refresh_chats(client, state).await;
        }
        ParsedCommand::Select(num) => {
            if let Some(n) = num {
                if n > 0 && n <= state.chats.len() {
                    select_chat(client, state, n - 1).await;
                } else {
                    display::print_error(format!(
                        "Invalid chat number. Choose 1-{}",
                        state.chats.len()
                    ));
                }
            } else {
                // Show current selection
                match &state.current_chat {
                    Some(chat) => {
                        display::print_success(format!("Currently in chat: {}", chat.title));
                    }
                    None => {
                        display::print_warning("No chat selected. Use /list then /select <n>");
                    }
                }
            }
        }
        ParsedCommand::New => {
            create_new_chat(client, state).await;
        }
        ParsedCommand::Refresh => {
            refresh_chats(client, state).await;
        }
        ParsedCommand::Agents => {
            client.print_agents().await;
        }
        ParsedCommand::Owners => {
            client.print_owners().await;
        }
        ParsedCommand::Unknown(other) => {
            display::print_error(format!("Unknown command: {}", other));
            display::print_info("Type /help to see available commands");
        }
        ParsedCommand::Quit => {}
    }
}

/// Print the help text.
fn print_help() {
    println!();
    println!("{}", "Available Commands:".bold().underline());
    println!("  {:<20} List all chats", "/list, /l".cyan());
    println!("  {:<20} Select chat by number", "/select <n>, /s <n>".cyan());
    println!("  {:<20} Create a new chat", "/new, /n".cyan());
    println!("  {:<20} Refresh chat list", "/refresh, /r".cyan());
    println!("  {:<20} List all agents", "/agents, /a".cyan());
    println!("  {:<20} List all owners", "/owners, /o".cyan());
    println!("  {:<20} Show this help", "/help, /h".cyan());
    println!("  {:<20} Exit interactive mode", "/quit, /q, /exit".cyan());
    println!();
    println!("{}", "Quick Start:".bold());
    println!("  1. /list  — see available chats");
    println!("  2. /select 1  — pick a chat");
    println!("  3. Type anything to send a message");
    println!("  4. Use /select <n> to switch chats");
    println!();
}

// ─── Unit Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─── parse_command tests ─────────────────────────────────────────────────

    #[test]
    fn test_parse_empty_input() {
        assert_eq!(parse_command(""), None);
        assert_eq!(parse_command("   "), None);
    }

    #[test]
    fn test_parse_help() {
        assert_eq!(parse_command("/help"), Some(ParsedCommand::Help));
        assert_eq!(parse_command("/h"), Some(ParsedCommand::Help));
        assert_eq!(parse_command("/?"), Some(ParsedCommand::Help));
    }

    #[test]
    fn test_parse_list() {
        assert_eq!(parse_command("/list"), Some(ParsedCommand::List));
        assert_eq!(parse_command("/l"), Some(ParsedCommand::List));
    }

    #[test]
    fn test_parse_select() {
        assert_eq!(parse_command("/select 5"), Some(ParsedCommand::Select(Some(5))));
        assert_eq!(parse_command("/s 1"), Some(ParsedCommand::Select(Some(1))));
    }

    #[test]
    fn test_parse_select_no_arg() {
        assert_eq!(parse_command("/select"), Some(ParsedCommand::Select(None)));
        assert_eq!(parse_command("/s"), Some(ParsedCommand::Select(None)));
    }

    #[test]
    fn test_parse_select_invalid_arg() {
        assert_eq!(parse_command("/select abc"), Some(ParsedCommand::Select(None)));
        assert_eq!(parse_command("/s -1"), Some(ParsedCommand::Select(None)));
    }

    #[test]
    fn test_parse_new() {
        assert_eq!(parse_command("/new"), Some(ParsedCommand::New));
        assert_eq!(parse_command("/n"), Some(ParsedCommand::New));
    }

    #[test]
    fn test_parse_refresh() {
        assert_eq!(parse_command("/refresh"), Some(ParsedCommand::Refresh));
        assert_eq!(parse_command("/r"), Some(ParsedCommand::Refresh));
    }

    #[test]
    fn test_parse_agents() {
        assert_eq!(parse_command("/agents"), Some(ParsedCommand::Agents));
        assert_eq!(parse_command("/a"), Some(ParsedCommand::Agents));
    }

    #[test]
    fn test_parse_owners() {
        assert_eq!(parse_command("/owners"), Some(ParsedCommand::Owners));
        assert_eq!(parse_command("/o"), Some(ParsedCommand::Owners));
    }

    #[test]
    fn test_parse_quit() {
        assert_eq!(parse_command("/quit"), Some(ParsedCommand::Quit));
        assert_eq!(parse_command("/q"), Some(ParsedCommand::Quit));
        assert_eq!(parse_command("/exit"), Some(ParsedCommand::Quit));
    }

    #[test]
    fn test_parse_unknown_command() {
        assert_eq!(
            parse_command("/foobar"),
            Some(ParsedCommand::Unknown("/foobar".to_string()))
        );
    }

    #[test]
    fn test_parse_unknown_command_with_args() {
        assert_eq!(
            parse_command("/blah arg1 arg2"),
            Some(ParsedCommand::Unknown("/blah".to_string()))
        );
    }

    #[test]
    fn test_parse_non_command_is_none() {
        // Plain text (messages) should return None, not a command
        assert_eq!(parse_command("hello"), None);
        assert_eq!(parse_command("how are you?"), None);
        assert_eq!(parse_command("12345"), None);
    }

    #[test]
    fn test_parse_select_with_extra_spaces() {
        assert_eq!(parse_command("/select   42"), Some(ParsedCommand::Select(Some(42))));
    }

    #[test]
    fn test_parse_case_insensitivity() {
        assert_eq!(parse_command("/Help"), Some(ParsedCommand::Help));
        assert_eq!(parse_command("/LIST"), Some(ParsedCommand::List));
        assert_eq!(parse_command("/QuIt"), Some(ParsedCommand::Quit));
    }

    #[test]
    fn test_parse_select_trailing_spaces() {
        assert_eq!(
            parse_command("/select 3  "),
            Some(ParsedCommand::Select(Some(3)))
        );
    }

    // ─── print_help smoke tests ───────────────────────────────────────────────

    #[test]
    fn test_print_help_does_not_panic() {
        // print_help() outputs to stdout via println!. Verify it doesn't panic.
        print_help();
    }

    #[test]
    fn test_print_help_runs_twice() {
        // Ensure no side effects on repeated calls
        print_help();
        print_help();
    }
}

/// Refresh the cached chat list from the backend.
async fn refresh_chats(client: &Client, state: &mut ReplState) {
    match client.list_chats().await {
        Ok(resp) => {
            state.chats = resp.data;
            if state.chats.is_empty() {
                display::print_info("No chats yet. Use /new to create one.");
            } else {
                display::print_success(format!("Found {} chat(s)", state.chats.len()));
                display::print_chats(&state.chats);
            }
        }
        Err(e) => {
            display::print_api_error(&e, None);
        }
    }
}

/// Select a chat by index and fetch its messages.
async fn select_chat(client: &Client, state: &mut ReplState, index: usize) {
    if index >= state.chats.len() {
        display::print_error("Chat index out of range");
        return;
    }

    let chat_id = &state.chats[index].id;
    match client.get_chat(chat_id).await {
        Ok(resp) => {
            if let Some(chat) = resp.data {
                display::print_success(format!("Selected: {}", chat.title));
                state.current_chat = Some(chat.clone());
                display::print_chat_with_messages(&chat);
            }
        }
        Err(e) => {
            display::print_api_error(&e, None);
        }
    }
}

/// Interactive chat creation wizard.
async fn create_new_chat(client: &Client, state: &mut ReplState) {
    display::print_header("Create New Chat");

    // List available agents for selection
    let agents = match client.list_agents().await {
        Ok(resp) => resp.data,
        Err(e) => {
            display::print_api_error(&e, None);
            return;
        }
    };

    if agents.is_empty() {
        display::print_error("No agents found. Create one first with: chat-cli create agent --name <name>");
        display::print_info("Or in another terminal, run: cargo run -- create agent --name \"MyBot\"");
        return;
    }

    display::print_success(format!("Available agents: {}", agents.len()));
    display::print_agents(&agents);
    print!("{}", "  Enter agent number: ".cyan());
    io::stdout().flush().ok();
    let mut agent_num = String::new();
    io::stdin().read_line(&mut agent_num).ok();
    let agent_idx = agent_num.trim().parse::<usize>().unwrap_or(1).saturating_sub(1);
    let agent_idx = agent_idx.min(agents.len().saturating_sub(1));
    let agent_id = &agents[agent_idx].id;
    display::print_success(format!("Selected agent: {}", agents[agent_idx].name));

    // List available owners for selection
    let owners = match client.list_owners().await {
        Ok(resp) => resp.data,
        Err(e) => {
            display::print_api_error(&e, None);
            return;
        }
    };

    if owners.is_empty() {
        display::print_error("No owners found. Create one first with: chat-cli create owner --name <name> --email <email>");
        return;
    }

    display::print_success(format!("Available owners: {}", owners.len()));
    display::print_owners(&owners);
    print!("{}", "  Enter owner number: ".cyan());
    io::stdout().flush().ok();
    let mut owner_num = String::new();
    io::stdin().read_line(&mut owner_num).ok();
    let owner_idx = owner_num.trim().parse::<usize>().unwrap_or(1).saturating_sub(1);
    let owner_idx = owner_idx.min(owners.len().saturating_sub(1));
    let owner_id = &owners[owner_idx].id;
    display::print_success(format!("Selected owner: {}", owners[owner_idx].name));

    // Title
    print!("{}", "  Chat title (optional, press Enter for default): ".cyan());
    io::stdout().flush().ok();
    let mut title = String::new();
    io::stdin().read_line(&mut title).ok();
    let title = title.trim();
    let title_opt = if title.is_empty() { None } else { Some(title) };

    // Create the chat
    match client.create_chat(agent_id, owner_id, title_opt).await {
        Ok(resp) => {
            if let Some(chat) = resp.data {
                display::print_success(format!("Chat created: {} ({})", chat.title, chat.id));
                // Select it automatically
                match client.get_chat(&chat.id).await {
                    Ok(resp) => {
                        if let Some(chat) = resp.data {
                            state.current_chat = Some(chat.clone());
                            state.chats.push(Chat {
                                id: chat.id.clone(),
                                title: chat.title.clone(),
                                agent_id: chat.agent_id.clone(),
                                owner_id: chat.owner_id.clone(),
                                created_at: chat.created_at.clone(),
                                updated_at: chat.updated_at.clone(),
                            });
                            display::print_chat_with_messages(&chat);
                        }
                    }
                    Err(e) => display::print_api_error(&e, None),
                }
            }
        }
        Err(e) => display::print_api_error(&e, None),
    }
}

/// Send the user's text as a message in the current chat.
async fn send_as_message(client: &Client, state: &ReplState, text: &str) {
    let chat = match &state.current_chat {
        Some(c) => c,
        None => return,
    };

    // Ask who is sending the message
    display::print_info("Who is sending this message?");
    println!("  1. {} (Owner)", "Me".cyan());
    println!("  2. {} (Agent)", "Agent".yellow());

    print!("  Choice [1]: ");
    io::stdout().flush().ok();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim();

    let (sender_type, sender_id) = match choice {
        "2" | "agent" => ("agent", &chat.agent_id),
        _ => ("owner", &chat.owner_id),
    };

    client
        .send_and_print(&chat.id, sender_type, sender_id, text)
        .await;
}
