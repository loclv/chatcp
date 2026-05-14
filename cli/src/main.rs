//! Chat CLI — terminal client for the Chat App Backend.
//!
//! Usage:
//!   chat-cli [OPTIONS] <COMMAND>
//!
//! Commands:
//!   repl           Interactive chat REPL
//!   list           List resources (agents, owners, chats)
//!   create         Create resources (agent, owner, chat)
//!   get            Get a resource by ID
//!   update         Update a resource by ID
//!   delete         Delete a resource by ID
//!   send           Send a message in a chat
//!   messages       View messages in a chat
//!   health         Check backend health

use clap::{Parser, Subcommand};

mod client;
mod config;
mod display;
mod models;
mod repl;

/// CLI client for the Chat App Backend — chat with agents and humans from the terminal.
#[derive(Parser)]
#[command(name = "chat-cli")]
#[command(version, about, long_about = None)]
#[command(styles = clap_cargo_style())]
struct Cli {
    /// Backend API base URL (overrides CHAT_API_URL env var).
    #[arg(long, global = true, env = "CHAT_API_URL")]
    api_url: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

/// Get clap styles for a nice terminal look.
fn clap_cargo_style() -> clap::builder::Styles {
    use clap::builder::styling;
    styling::Styles::styled()
        .header(styling::AnsiColor::Yellow.on_default().bold())
        .usage(styling::AnsiColor::Green.on_default().bold())
        .literal(styling::AnsiColor::Cyan.on_default().bold())
        .placeholder(styling::AnsiColor::White.on_default().dimmed())
}

#[derive(Subcommand)]
enum Commands {
    /// Start interactive chat REPL.
    Repl {
        /// Automatically select this chat by ID on startup.
        #[arg(short, long)]
        chat_id: Option<String>,
    },

    /// List resources.
    #[command(subcommand)]
    List(ListCommands),

    /// Create a resource.
    #[command(subcommand)]
    Create(CreateCommands),

    /// Get a resource by ID.
    #[command(subcommand)]
    Get(GetCommands),

    /// Update a resource by ID.
    #[command(subcommand)]
    Update(UpdateCommands),

    /// Delete a resource by ID.
    #[command(subcommand)]
    Delete(DeleteCommands),

    /// Send a message in a chat.
    Send {
        /// Chat ID to send the message to.
        #[arg(short, long)]
        chat_id: String,

        /// Sender type: "agent" or "owner".
        #[arg(short, long)]
        as_type: String,

        /// Sender ID (UUID of the agent or owner).
        #[arg(short = 'i', long)]
        sender_id: String,

        /// Message content.
        #[arg(short, long)]
        text: String,
    },

    /// View messages in a chat.
    Messages {
        /// Chat ID.
        #[arg(short, long)]
        chat_id: String,
    },

    /// Check backend health.
    Health,
}

#[derive(Subcommand)]
enum ListCommands {
    /// List all agents.
    Agents,
    /// List all owners.
    Owners,
    /// List all chats.
    Chats,
}

#[derive(Subcommand)]
enum CreateCommands {
    /// Create a new agent.
    Agent {
        /// Agent name.
        #[arg(short, long)]
        name: String,

        /// Agent description.
        #[arg(short, long)]
        description: Option<String>,

        /// Owner ID to assign.
        #[arg(short = 'o', long)]
        owner_id: Option<String>,
    },
    /// Create a new owner.
    Owner {
        /// Owner name.
        #[arg(short, long)]
        name: String,

        /// Owner email.
        #[arg(short, long)]
        email: String,
    },
    /// Create a new chat between an agent and an owner.
    Chat {
        /// Agent ID.
        #[arg(short = 'a', long)]
        agent_id: String,

        /// Owner ID.
        #[arg(short = 'o', long)]
        owner_id: String,

        /// Chat title (optional).
        #[arg(short, long)]
        title: Option<String>,
    },
}

#[derive(Subcommand)]
enum GetCommands {
    /// Get an agent by ID.
    Agent {
        /// Agent ID.
        id: String,
    },
    /// Get an owner by ID.
    Owner {
        /// Owner ID.
        id: String,
    },
    /// Get a chat with messages by ID.
    Chat {
        /// Chat ID.
        id: String,
    },
}

#[derive(Subcommand)]
enum UpdateCommands {
    /// Update an agent.
    Agent {
        /// Agent ID.
        #[arg(short, long)]
        id: String,

        /// New name.
        #[arg(short, long)]
        name: Option<String>,

        /// New description.
        #[arg(short, long)]
        description: Option<String>,                    /// New owner ID.
                    #[arg(short = 'o', long)]
                    owner_id: Option<String>,
    },
    /// Update an owner.
    Owner {
        /// Owner ID.
        #[arg(short, long)]
        id: String,

        /// New name.
        #[arg(short, long)]
        name: Option<String>,

        /// New email.
        #[arg(short = 'e', long)]
        email: Option<String>,
    },
    /// Update a chat title.
    Chat {
        /// Chat ID.
        #[arg(short, long)]
        id: String,

        /// New title.
        #[arg(short, long)]
        title: String,
    },
}

#[derive(Subcommand)]
enum DeleteCommands {
    /// Delete an agent.
    Agent {
        /// Agent ID.
        id: String,
    },
    /// Delete an owner.
    Owner {
        /// Owner ID.
        id: String,
    },
    /// Delete a chat.
    Chat {
        /// Chat ID.
        id: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Override API URL if provided via --api-url flag
    let mut cfg = config::Config::from_env();
    if let Some(url) = &cli.api_url {
        cfg.base_url = url.clone();
    }

    let client = client::Client::new(cfg);

    if let Err(e) = dispatch(cli.command, &client).await {
        display::print_error(&e);
        std::process::exit(1);
    }
}

/// Dispatch the parsed command to the appropriate handler.
async fn dispatch(command: Commands, client: &client::Client) -> Result<(), String> {
    match command {
        Commands::Health => {
            client.check_health().await;
            Ok(())
        }

        Commands::Repl { chat_id: _ } => {
            repl::run(client).await;
            Ok(())
        }

        Commands::List(list_cmd) => match list_cmd {
            ListCommands::Agents => {
                client.print_agents().await;
                Ok(())
            }
            ListCommands::Owners => {
                client.print_owners().await;
                Ok(())
            }
            ListCommands::Chats => {
                client.print_chats().await;
                Ok(())
            }
        },

        Commands::Create(create_cmd) => match create_cmd {
            CreateCommands::Agent {
                name,
                description,
                owner_id,
            } => {
                match client
                    .create_agent(&name, description.as_deref(), owner_id.as_deref())
                    .await
                {
                    Ok(resp) => {
                        if let Some(agent) = resp.data {
                            display::print_success("Agent created!");
                            display::print_agent(&agent);
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            CreateCommands::Owner { name, email } => {
                match client.create_owner(&name, &email).await {
                    Ok(resp) => {
                        if let Some(owner) = resp.data {
                            display::print_success("Owner created!");
                            display::print_owner(&owner);
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            CreateCommands::Chat {
                agent_id,
                owner_id,
                title,
            } => {
                match client.create_chat(&agent_id, &owner_id, title.as_deref()).await {
                    Ok(resp) => {
                        if let Some(chat) = resp.data {
                            display::print_success("Chat created!");
                            display::print_chat(&chat);
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
        },

        Commands::Get(get_cmd) => match get_cmd {
            GetCommands::Agent { id } => {
                match client.get_agent(&id).await {
                    Ok(resp) => {
                        if let Some(agent) = resp.data {
                            display::print_agent(&agent);
                        } else {
                            display::print_error("Agent not found");
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            GetCommands::Owner { id } => {
                match client.get_owner(&id).await {
                    Ok(resp) => {
                        if let Some(owner) = resp.data {
                            display::print_owner(&owner);
                        } else {
                            display::print_error("Owner not found");
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            GetCommands::Chat { id } => {
                client.print_chat(&id).await;
                Ok(())
            }
        },

        Commands::Update(update_cmd) => match update_cmd {
            UpdateCommands::Agent {
                id,
                name,
                description,
                owner_id,
            } => {
                match client
                    .update_agent(&id, name.as_deref(), description.as_deref(), owner_id.as_deref())
                    .await
                {
                    Ok(resp) => {
                        if let Some(agent) = resp.data {
                            display::print_success("Agent updated!");
                            display::print_agent(&agent);
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            UpdateCommands::Owner { id, name, email } => {
                match client.update_owner(&id, name.as_deref(), email.as_deref()).await {
                    Ok(resp) => {
                        if let Some(owner) = resp.data {
                            display::print_success("Owner updated!");
                            display::print_owner(&owner);
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            UpdateCommands::Chat { id, title } => {
                match client.update_chat(&id, &title).await {
                    Ok(resp) => {
                        if let Some(chat) = resp.data {
                            display::print_success("Chat updated!");
                            display::print_chat_with_messages(&chat);
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
        },

        Commands::Delete(delete_cmd) => match delete_cmd {
            DeleteCommands::Agent { id } => {
                match client.delete_agent(&id).await {
                    Ok(_) => {
                        display::print_success("Agent deleted!");
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            DeleteCommands::Owner { id } => {
                match client.delete_owner(&id).await {
                    Ok(_) => {
                        display::print_success("Owner deleted!");
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            DeleteCommands::Chat { id } => {
                match client.delete_chat(&id).await {
                    Ok(_) => {
                        display::print_success("Chat deleted!");
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
        },

        Commands::Send {
            chat_id,
            as_type,
            sender_id,
            text,
        } => {
            client
                .send_and_print(&chat_id, &as_type, &sender_id, &text)
                .await;
            Ok(())
        }

        Commands::Messages { chat_id } => {
            match client.get_messages(&chat_id).await {
                Ok(resp) => {
                    display::print_success(format!("Found {} message(s)", resp.total));
                    display::print_messages(&resp.data);
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Top-level command parsing ────────────────────────────────────────────────

    #[test]
    fn test_parse_health() {
        let cli = Cli::try_parse_from(&["chat-cli", "health"]).unwrap();
        assert!(matches!(cli.command, Commands::Health));
    }

    #[test]
    fn test_parse_health_with_url() {
        let cli =
            Cli::try_parse_from(&["chat-cli", "--api-url", "http://test:8787", "health"]).unwrap();
        assert_eq!(cli.api_url, Some("http://test:8787".to_string()));
        assert!(matches!(cli.command, Commands::Health));
    }

    #[test]
    fn test_parse_repl() {
        let cli = Cli::try_parse_from(&["chat-cli", "repl"]).unwrap();
        assert!(matches!(cli.command, Commands::Repl { .. }));
    }

    #[test]
    fn test_parse_repl_with_chat_id() {
        let cli = Cli::try_parse_from(&["chat-cli", "repl", "--chat-id", "abc-123"]).unwrap();
        if let Commands::Repl { chat_id } = cli.command {
            assert_eq!(chat_id, Some("abc-123".to_string()));
        } else {
            panic!("Expected Repl command");
        }
    }

    // ─── List subcommands ────────────────────────────────────────────────────

    #[test]
    fn test_parse_list_agents() {
        let cli = Cli::try_parse_from(&["chat-cli", "list", "agents"]).unwrap();
        assert!(matches!(cli.command, Commands::List(ListCommands::Agents)));
    }

    #[test]
    fn test_parse_list_owners() {
        let cli = Cli::try_parse_from(&["chat-cli", "list", "owners"]).unwrap();
        assert!(matches!(cli.command, Commands::List(ListCommands::Owners)));
    }

    #[test]
    fn test_parse_list_chats() {
        let cli = Cli::try_parse_from(&["chat-cli", "list", "chats"]).unwrap();
        assert!(matches!(cli.command, Commands::List(ListCommands::Chats)));
    }

    // ─── Create subcommands ──────────────────────────────────────────────────

    #[test]
    fn test_parse_create_agent() {
        let cli = Cli::try_parse_from(&["chat-cli", "create", "agent", "--name", "TestBot"]).unwrap();
        if let Commands::Create(CreateCommands::Agent { name, description, owner_id }) = cli.command {
            assert_eq!(name, "TestBot");
            assert!(description.is_none());
            assert!(owner_id.is_none());
        } else {
            panic!("Expected Create Agent command");
        }
    }

    #[test]
    fn test_parse_create_agent_full() {
        let cli = Cli::try_parse_from(&[
            "chat-cli", "create", "agent",
            "--name", "TestBot",
            "--description", "A test bot",
            "--owner-id", "abc-123",
        ]).unwrap();
        if let Commands::Create(CreateCommands::Agent { name, description, owner_id }) = cli.command {
            assert_eq!(name, "TestBot");
            assert_eq!(description, Some("A test bot".to_string()));
            assert_eq!(owner_id, Some("abc-123".to_string()));
        } else {
            panic!("Expected Create Agent command");
        }
    }

    #[test]
    fn test_parse_create_owner() {
        let cli = Cli::try_parse_from(&[
            "chat-cli", "create", "owner",
            "--name", "Alice",
            "--email", "alice@example.com",
        ]).unwrap();
        if let Commands::Create(CreateCommands::Owner { name, email }) = cli.command {
            assert_eq!(name, "Alice");
            assert_eq!(email, "alice@example.com");
        } else {
            panic!("Expected Create Owner command");
        }
    }

    #[test]
    fn test_parse_create_chat() {
        let cli = Cli::try_parse_from(&[
            "chat-cli", "create", "chat",
            "--agent-id", "agent-1",
            "--owner-id", "owner-1",
        ]).unwrap();
        if let Commands::Create(CreateCommands::Chat { agent_id, owner_id, title }) = cli.command {
            assert_eq!(agent_id, "agent-1");
            assert_eq!(owner_id, "owner-1");
            assert!(title.is_none());
        } else {
            panic!("Expected Create Chat command");
        }
    }

    #[test]
    fn test_parse_create_chat_with_title() {
        let cli = Cli::try_parse_from(&[
            "chat-cli", "create", "chat",
            "--agent-id", "agent-1",
            "--owner-id", "owner-1",
            "--title", "Help Chat",
        ]).unwrap();
        if let Commands::Create(CreateCommands::Chat { title, .. }) = cli.command {
            assert_eq!(title, Some("Help Chat".to_string()));
        } else {
            panic!("Expected Create Chat command");
        }
    }

    // ─── Get subcommands ─────────────────────────────────────────────────────

    #[test]
    fn test_parse_get_agent() {
        let cli = Cli::try_parse_from(&["chat-cli", "get", "agent", "abc-123"]).unwrap();
        if let Commands::Get(GetCommands::Agent { id }) = cli.command {
            assert_eq!(id, "abc-123");
        } else {
            panic!("Expected Get Agent command");
        }
    }

    #[test]
    fn test_parse_get_owner() {
        let cli = Cli::try_parse_from(&["chat-cli", "get", "owner", "abc-123"]).unwrap();
        assert!(matches!(cli.command, Commands::Get(GetCommands::Owner { .. })));
    }

    #[test]
    fn test_parse_get_chat() {
        let cli = Cli::try_parse_from(&["chat-cli", "get", "chat", "abc-123"]).unwrap();
        assert!(matches!(cli.command, Commands::Get(GetCommands::Chat { .. })));
    }

    // ─── Update subcommands ──────────────────────────────────────────────────

    #[test]
    fn test_parse_update_agent_name() {
        let cli = Cli::try_parse_from(&[
            "chat-cli", "update", "agent",
            "--id", "abc-123",
            "--name", "NewName",
        ]).unwrap();
        if let Commands::Update(UpdateCommands::Agent { id, name, .. }) = cli.command {
            assert_eq!(id, "abc-123");
            assert_eq!(name, Some("NewName".to_string()));
        } else {
            panic!("Expected Update Agent command");
        }
    }

    #[test]
    fn test_parse_update_chat() {
        let cli = Cli::try_parse_from(&[
            "chat-cli", "update", "chat",
            "--id", "abc-123",
            "--title", "Updated Title",
        ]).unwrap();
        if let Commands::Update(UpdateCommands::Chat { id, title }) = cli.command {
            assert_eq!(id, "abc-123");
            assert_eq!(title, "Updated Title");
        } else {
            panic!("Expected Update Chat command");
        }
    }

    // ─── Delete subcommands ──────────────────────────────────────────────────

    #[test]
    fn test_parse_delete_agent() {
        let cli = Cli::try_parse_from(&["chat-cli", "delete", "agent", "abc-123"]).unwrap();
        assert!(matches!(cli.command, Commands::Delete(DeleteCommands::Agent { .. })));
    }

    #[test]
    fn test_parse_delete_owner() {
        let cli = Cli::try_parse_from(&["chat-cli", "delete", "owner", "abc-123"]).unwrap();
        assert!(matches!(cli.command, Commands::Delete(DeleteCommands::Owner { .. })));
    }

    #[test]
    fn test_parse_delete_chat() {
        let cli = Cli::try_parse_from(&["chat-cli", "delete", "chat", "abc-123"]).unwrap();
        assert!(matches!(cli.command, Commands::Delete(DeleteCommands::Chat { .. })));
    }

    // ─── Send & Messages ────────────────────────────────────────────────────

    #[test]
    fn test_parse_send_message() {
        let cli = Cli::try_parse_from(&[
            "chat-cli", "send",
            "--chat-id", "chat-1",
            "--as-type", "agent",
            "--sender-id", "agent-1",
            "--text", "Hello!",
        ]).unwrap();
        if let Commands::Send { chat_id, as_type, sender_id, text } = cli.command {
            assert_eq!(chat_id, "chat-1");
            assert_eq!(as_type, "agent");
            assert_eq!(sender_id, "agent-1");
            assert_eq!(text, "Hello!");
        } else {
            panic!("Expected Send command");
        }
    }

    #[test]
    fn test_parse_messages() {
        let cli = Cli::try_parse_from(&["chat-cli", "messages", "--chat-id", "chat-1"]).unwrap();
        if let Commands::Messages { chat_id } = cli.command {
            assert_eq!(chat_id, "chat-1");
        } else {
            panic!("Expected Messages command");
        }
    }

    // ─── Error cases ────────────────────────────────────────────────────────

    #[test]
    fn test_parse_invalid_command_fails() {
        let result = Cli::try_parse_from(&["chat-cli", "invalid-command"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_list_without_subcommand_fails() {
        let result = Cli::try_parse_from(&["chat-cli", "list"]);
        assert!(result.is_err());
    }

    // ─── Helper functions ─────────────────────────────────────────────────────

    #[test]
    fn test_clap_cargo_style_returns_styles() {
        let _styles = clap_cargo_style();
    }

    // ─── More parsing edge cases ───────────────────────────────────────────────

    #[test]
    fn test_parse_health_with_extra_arg_fails() {
        let result = Cli::try_parse_from(&["chat-cli", "health", "extra_arg"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_send_missing_required() {
        let result = Cli::try_parse_from(&["chat-cli", "send", "--chat-id", "chat-1"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_send_empty_text() {
        let cli = Cli::try_parse_from(&[
            "chat-cli", "send",
            "--chat-id", "chat-1",
            "--as-type", "agent",
            "--sender-id", "agent-1",
            "--text", "",
        ]).unwrap();
        if let Commands::Send { text, .. } = cli.command {
            assert_eq!(text, "");
        }
    }

    #[test]
    fn test_parse_messages_missing_chat_id_fails() {
        let result = Cli::try_parse_from(&["chat-cli", "messages"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_create_agent_missing_name_fails() {
        let result = Cli::try_parse_from(&["chat-cli", "create", "agent"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_create_owner_missing_email_fails() {
        let result = Cli::try_parse_from(&["chat-cli", "create", "owner", "--name", "Alice"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_create_chat_missing_agent_fails() {
        let result = Cli::try_parse_from(&["chat-cli", "create", "chat", "--owner-id", "owner-1"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_delete_missing_id_fails() {
        let result = Cli::try_parse_from(&["chat-cli", "delete", "agent"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_list_subcommand_typo_fails() {
        let result = Cli::try_parse_from(&["chat-cli", "list", "agent"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_get_with_extra_args_fails() {
        let result = Cli::try_parse_from(&["chat-cli", "get", "agent", "id-1", "extra"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_update_owner_email() {
        let cli = Cli::try_parse_from(&[
            "chat-cli", "update", "owner",
            "--id", "owner-1",
            "--email", "new@example.com",
        ]).unwrap();
        if let Commands::Update(UpdateCommands::Owner { id, name, email }) = cli.command {
            assert_eq!(id, "owner-1");
            assert!(name.is_none());
            assert_eq!(email, Some("new@example.com".to_string()));
        }
    }

    #[test]
    fn test_parse_update_agent_with_owner() {
        let cli = Cli::try_parse_from(&[
            "chat-cli", "update", "agent",
            "--id", "agent-1",
            "--owner-id", "owner-1",
        ]).unwrap();
        if let Commands::Update(UpdateCommands::Agent { id, owner_id, .. }) = cli.command {
            assert_eq!(id, "agent-1");
            assert_eq!(owner_id, Some("owner-1".to_string()));
        }
    }
}

