//! Terminal display utilities for the chat CLI.
//!
//! Provides pretty-printing for agents, owners, chats, messages, and errors
//! using colored output and table formatting.

use colored::*;
use prettytable::{format, row, Table};

use crate::models::*;

// ─── Terminal Helpers ────────────────────────────────────────────────────────

/// Print a success message in green.
pub fn print_success(msg: impl AsRef<str>) {
    println!("{} {}", "✓".green().bold(), msg.as_ref().green());
}

/// Print an info message in blue.
pub fn print_info(msg: impl AsRef<str>) {
    println!("{} {}", "ℹ".blue().bold(), msg.as_ref().blue());
}

/// Print an error message in red.
pub fn print_error(msg: impl AsRef<str>) {
    eprintln!("{} {}", "✗".red().bold(), msg.as_ref().red());
}

/// Print a warning message in yellow.
pub fn print_warning(msg: impl AsRef<str>) {
    println!("{} {}", "⚠".yellow().bold(), msg.as_ref().yellow());
}

/// Print a section header.
pub fn print_header(msg: impl AsRef<str>) {
    println!("\n{}", msg.as_ref().bold().underline());
}

/// Print a labeled value.
pub fn print_field(label: &str, value: impl AsRef<str>) {
    println!("  {}: {}", label.cyan().bold(), value.as_ref().white());
}

/// Print an empty line.
pub fn print_empty() {
    println!();
}

// ─── Agent Display ───────────────────────────────────────────────────────────

/// Display a single agent in detail.
pub fn print_agent(agent: &Agent) {
    print_header("Agent");
    print_field("ID", &agent.id);
    print_field("Name", &agent.name);
    print_field("Description", &agent.description);
    print_field(
        "Owner",
        agent
            .owner_id
            .as_deref()
            .unwrap_or("(unassigned)"),
    );
    print_field("Created", &agent.created_at);
    print_field("Updated", &agent.updated_at);
    print_empty();
}

/// Display a list of agents in a table.
pub fn print_agents(agents: &[Agent]) {
    if agents.is_empty() {
        print_info("No agents found.");
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BOX_CHARS);
    table.set_titles(row![
        "ID",
        "Name",
        "Description",
        "Owner ID",
        "Created"
    ]);

    for agent in agents {
        let short_id = &agent.id[..8];
        let desc = if agent.description.len() > 30 {
            format!("{}...", &agent.description[..27])
        } else {
            agent.description.clone()
        };
        let owner = agent.owner_id.as_deref().unwrap_or("-");
        table.add_row(row![
            short_id,
            agent.name.yellow().bold(),
            desc,
            owner,
            &agent.created_at[..10],
        ]);
    }

    table.printstd();
}

// ─── Owner Display ───────────────────────────────────────────────────────────

/// Display a single owner in detail.
pub fn print_owner(owner: &Owner) {
    print_header("Owner");
    print_field("ID", &owner.id);
    print_field("Name", &owner.name);
    print_field("Email", &owner.email);
    print_field("Created", &owner.created_at);
    print_empty();
}

/// Display a list of owners in a table.
pub fn print_owners(owners: &[Owner]) {
    if owners.is_empty() {
        print_info("No owners found.");
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BOX_CHARS);
    table.set_titles(row!["ID", "Name", "Email", "Created"]);

    for owner in owners {
        let short_id = &owner.id[..8];
        table.add_row(row![
            short_id,
            owner.name.yellow().bold(),
            owner.email.cyan(),
            &owner.created_at[..10],
        ]);
    }

    table.printstd();
}

// ─── Chat Display ────────────────────────────────────────────────────────────

/// Display a single chat in detail.
pub fn print_chat(chat: &Chat) {
    print_header("Chat");
    print_field("ID", &chat.id);
    print_field("Title", &chat.title);
    print_field("Agent ID", &chat.agent_id);
    print_field("Owner ID", &chat.owner_id);
    print_field("Created", &chat.created_at);
    print_field("Updated", &chat.updated_at);
    print_empty();
}

/// Display a list of chats in a table.
pub fn print_chats(chats: &[Chat]) {
    if chats.is_empty() {
        print_info("No chats found.");
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BOX_CHARS);
    table.set_titles(row!["#", "ID", "Title", "Agent", "Owner", "Updated"]);

    for (i, chat) in chats.iter().enumerate() {
        let short_id = &chat.id[..8];
        let title = if chat.title.len() > 25 {
            format!("{}...", &chat.title[..22])
        } else {
            chat.title.clone()
        };
        let agent_short = &chat.agent_id[..8];
        let owner_short = &chat.owner_id[..8];
        table.add_row(row![
            (i + 1).to_string(),
            short_id,
            title.yellow().bold(),
            agent_short,
            owner_short,
            &chat.updated_at[..10],
        ]);
    }

    table.printstd();
}

// ─── Message Display ─────────────────────────────────────────────────────────

/// Color to use for different sender types.
fn sender_color(sender_type: &str) -> colored::Color {
    match sender_type {
        "agent" => colored::Color::BrightMagenta,
        "owner" => colored::Color::BrightCyan,
        _ => colored::Color::White,
    }
}

/// Sender label for display.
fn sender_label(sender_type: &str) -> &str {
    match sender_type {
        "agent" => "Agent",
        "owner" => "Owner",
        _ => "Unknown",
    }
}

/// Display a single message.
pub fn print_message(msg: &Message) {
    let color = sender_color(&msg.sender_type);
    let label = sender_label(&msg.sender_type);
    let sender = format!("[{}] {} {}", label, msg.sender_id, msg.created_at);
    println!("  {}", sender.color(color).bold());
    println!("  {}", msg.content.white());
    println!();
}

/// Display a list of messages (conversation view).
pub fn print_messages(messages: &[Message]) {
    if messages.is_empty() {
        print_info("No messages in this chat.");
        return;
    }

    for msg in messages {
        print_message(msg);
    }
}

/// Display a chat with its full message history.
pub fn print_chat_with_messages(chat: &ChatWithMessages) {
    print_header(format!("Chat: {}", chat.title.bold()));
    print_field("ID", &chat.id);
    print_field("Agent", &chat.agent_id);
    print_field("Owner", &chat.owner_id);
    print_field("Messages", chat.messages.len().to_string());
    print_empty();

    if chat.messages.is_empty() {
        print_info("No messages yet. Start the conversation!");
    } else {
        print_messages(&chat.messages);
    }
}

// ─── Error Display ───────────────────────────────────────────────────────────

/// Display an API error response.
pub fn print_api_error(error: &str, code: Option<&str>) {
    if let Some(c) = code {
        print_error(format!("[{}] {}", c, error));
    } else {
        print_error(error);
    }
}

/// Display an HTTP-level error (connection refused, timeout, etc.).
pub fn print_http_error(error: &str) {
    print_error(format!("Connection error: {}", error));
    print_info("Make sure the backend is running. Use --api-url or set CHAT_API_URL to change the URL.");
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Color/Label helpers ──────────────────────────────────────────────────

    #[test]
    fn test_sender_color_agent() {
        assert_eq!(sender_color("agent"), colored::Color::BrightMagenta);
    }

    #[test]
    fn test_sender_color_owner() {
        assert_eq!(sender_color("owner"), colored::Color::BrightCyan);
    }

    #[test]
    fn test_sender_color_unknown() {
        assert_eq!(sender_color("system"), colored::Color::White);
        assert_eq!(sender_color(""), colored::Color::White);
    }

    #[test]
    fn test_sender_label_agent() {
        assert_eq!(sender_label("agent"), "Agent");
    }

    #[test]
    fn test_sender_label_owner() {
        assert_eq!(sender_label("owner"), "Owner");
    }

    #[test]
    fn test_sender_label_unknown() {
        assert_eq!(sender_label("system"), "Unknown");
        assert_eq!(sender_label(""), "Unknown");
    }

    // ─── Terminal helper smoke tests ───────────────────────────────────────────

    /// Helper: run a function that prints to stdout/sdterr, just verify no panic.
    fn check_no_panic(f: impl FnOnce()) {
        f();
    }

    #[test]
    fn test_print_success() { check_no_panic(|| print_success("done")); }

    #[test]
    fn test_print_info() { check_no_panic(|| print_info("info")); }

    #[test]
    fn test_print_error() { check_no_panic(|| print_error("err")); }

    #[test]
    fn test_print_warning() { check_no_panic(|| print_warning("warn")); }

    #[test]
    fn test_print_header() { check_no_panic(|| print_header("Title")); }

    #[test]
    fn test_print_field() { check_no_panic(|| print_field("key", "value")); }

    #[test]
    fn test_print_empty() { check_no_panic(print_empty); }

    // ─── Agent display smoke tests ─────────────────────────────────────────────

    fn sample_agent(owner: Option<&str>) -> Agent {
        Agent {
            id: "550e8400-e29b-41d4-a716-446655440000".into(),
            name: "TestBot".into(),
            description: "A test agent".into(),
            owner_id: owner.map(|s| s.into()),
            created_at: "2025-01-01 00:00:00".into(),
            updated_at: "2025-01-01 00:00:00".into(),
        }
    }

    #[test]
    fn test_print_agent() {
        check_no_panic(|| print_agent(&sample_agent(Some("own-1"))));
    }

    #[test]
    fn test_print_agent_unassigned() {
        check_no_panic(|| print_agent(&sample_agent(None)));
    }

    #[test]
    fn test_print_agents_empty() {
        check_no_panic(|| print_agents(&[]));
    }

    #[test]
    fn test_print_agents_non_empty() {
        check_no_panic(|| print_agents(&[sample_agent(Some("o1")), sample_agent(None)]));
    }

    // ─── Owner display smoke tests ─────────────────────────────────────────────

    fn sample_owner() -> Owner {
        Owner {
            id: "f47ac10b-58cc-4372-a567-0e02b2c3d479".into(),
            name: "Alice".into(),
            email: "alice@test.com".into(),
            created_at: "2025-01-01".into(),
        }
    }

    #[test]
    fn test_print_owner() {
        check_no_panic(|| print_owner(&sample_owner()));
    }

    #[test]
    fn test_print_owners_empty() {
        check_no_panic(|| print_owners(&[]));
    }

    #[test]
    fn test_print_owners_non_empty() {
        check_no_panic(|| print_owners(&[sample_owner()]));
    }

    // ─── Chat display smoke tests ──────────────────────────────────────────────

    fn sample_chat() -> Chat {
        Chat {
            id: "550e8400-e29b-41d4-a716-446655440000".into(),
            title: "Help Chat".into(),
            agent_id: "f47ac10b-58cc-4372-a567-0e02b2c3d479".into(),
            owner_id: "a1b2c3d4-e5f6-4789-abcd-ef0123456789".into(),
            created_at: "2025-01-01".into(),
            updated_at: "2025-01-01".into(),
        }
    }

    #[test]
    fn test_print_chat() {
        check_no_panic(|| print_chat(&sample_chat()));
    }

    #[test]
    fn test_print_chats_empty() {
        check_no_panic(|| print_chats(&[]));
    }

    #[test]
    fn test_print_chats_non_empty() {
        check_no_panic(|| print_chats(&[sample_chat()]));
    }

    // ─── Message display smoke tests ───────────────────────────────────────────

    fn sample_msg(sender_type: &str) -> Message {
        Message {
            id: "msg-1".into(),
            chat_id: "chat-1".into(),
            sender_type: sender_type.into(),
            sender_id: "sender-1".into(),
            content: "Hello world!".into(),
            created_at: "2025-01-01 12:00:00".into(),
        }
    }

    #[test]
    fn test_print_message_agent() {
        check_no_panic(|| print_message(&sample_msg("agent")));
    }

    #[test]
    fn test_print_message_owner() {
        check_no_panic(|| print_message(&sample_msg("owner")));
    }

    #[test]
    fn test_print_messages_empty() {
        check_no_panic(|| print_messages(&[]));
    }

    #[test]
    fn test_print_messages_non_empty() {
        check_no_panic(|| print_messages(&[sample_msg("agent"), sample_msg("owner")]));
    }

    // ─── ChatWithMessages display smoke tests ──────────────────────────────────

    fn sample_chat_with_msgs(msg_count: usize) -> ChatWithMessages {
        ChatWithMessages {
            id: "chat-1".into(),
            title: "Help Chat".into(),
            agent_id: "agent-1".into(),
            owner_id: "owner-1".into(),
            created_at: "2025-01-01".into(),
            updated_at: "2025-01-01".into(),
            messages: (0..msg_count).map(|i| Message {
                id: format!("msg-{}", i),
                chat_id: "chat-1".into(),
                sender_type: if i % 2 == 0 { "agent" } else { "owner" }.into(),
                sender_id: "sender-1".into(),
                content: format!("Message {}", i),
                created_at: "2025-01-01".into(),
            }).collect(),
        }
    }

    #[test]
    fn test_print_chat_with_messages_empty() {
        check_no_panic(|| print_chat_with_messages(&sample_chat_with_msgs(0)));
    }

    #[test]
    fn test_print_chat_with_messages_with_msgs() {
        check_no_panic(|| print_chat_with_messages(&sample_chat_with_msgs(5)));
    }

    #[test]
    fn test_print_chat_with_messages_single_msg() {
        check_no_panic(|| print_chat_with_messages(&sample_chat_with_msgs(1)));
    }

    // ─── Error display smoke tests ─────────────────────────────────────────────

    #[test]
    fn test_print_api_error_with_code() {
        check_no_panic(|| print_api_error("not found", Some("ERR_NOT_FOUND")));
    }

    #[test]
    fn test_print_api_error_without_code() {
        check_no_panic(|| print_api_error("something broke", None));
    }

    #[test]
    fn test_print_http_error() {
        check_no_panic(|| print_http_error("connection refused"));
    }

    // ─── Long content edge cases ───────────────────────────────────────────────

    fn sample_long_desc_agent() -> Agent {
        Agent {
            id: "550e8400-e29b-41d4-a716-446655440000".into(),
            name: "LongDescBot".into(),
            description: "A very long description that exceeds thirty characters for truncation testing purposes".into(),
            owner_id: Some("f47ac10b-58cc-4372-a567-0e02b2c3d479".into()),
            created_at: "2025-01-01 00:00:00".into(),
            updated_at: "2025-01-01 00:00:00".into(),
        }
    }

    fn sample_long_title_chat() -> Chat {
        Chat {
            id: "550e8400-e29b-41d4-a716-446655440000".into(),
            title: "A very long chat title that definitely exceeds twenty five characters for testing".into(),
            agent_id: "f47ac10b-58cc-4372-a567-0e02b2c3d479".into(),
            owner_id: "a1b2c3d4-e5f6-4789-abcd-ef0123456789".into(),
            created_at: "2025-01-01".into(),
            updated_at: "2025-01-01".into(),
        }
    }

    #[test]
    fn test_print_agent_long_description() {
        check_no_panic(|| print_agent(&sample_long_desc_agent()));
    }

    #[test]
    fn test_print_agents_with_long_descriptions() {
        check_no_panic(|| print_agents(&[sample_long_desc_agent()]));
    }

    #[test]
    fn test_print_chat_long_title() {
        check_no_panic(|| print_chat(&sample_long_title_chat()));
    }

    #[test]
    fn test_print_chats_with_long_titles() {
        check_no_panic(|| print_chats(&[sample_long_title_chat()]));
    }

    #[test]
    fn test_print_messages_large_batch() {
        let msgs: Vec<Message> = (0..20).map(|i| Message {
            id: format!("msg-{}", i),
            chat_id: "chat-1".into(),
            sender_type: if i % 2 == 0 { "agent" } else { "owner" }.into(),
            sender_id: "sender-1".into(),
            content: format!("Long message content number {} that goes on for a while to test rendering", i),
            created_at: "2025-01-01".into(),
        }).collect();
        check_no_panic(|| print_messages(&msgs));
    }
}

