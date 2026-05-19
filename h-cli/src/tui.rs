use std::io;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use chat_cli::client::Client;
use chat_cli::models::{Agent, Chat, Message, Owner};
use crate::session::Session;

/// The different screens in the TUI.
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    /// Selecting which owner to act as.
    SelectOwner,
    /// Viewing the list of chats for the selected owner.
    ChatList,
    /// Inside a specific chat room.
    ChatRoom(String),
}

/// The state of the TUI application.
pub struct TuiApp<'a> {
    pub client: &'a Client,
    pub session: Session,
    pub screen: Screen,
    pub should_quit: bool,

    // Data
    pub owners: Vec<Owner>,
    pub owners_state: ListState,
    pub chats: Vec<Chat>,
    pub chats_state: ListState,
    pub messages: Vec<Message>,
    pub agents: Vec<Agent>,

    // Input
    pub input: String,
    pub cursor_position: usize,

    // Polling
    pub last_refresh: Instant,
}

impl<'a> TuiApp<'a> {
    pub fn new(client: &'a Client, session: Session) -> Self {
        Self {
            client,
            session,
            screen: Screen::SelectOwner,
            should_quit: false,
            owners: Vec::new(),
            owners_state: ListState::default(),
            chats: Vec::new(),
            chats_state: ListState::default(),
            messages: Vec::new(),
            agents: Vec::new(),
            input: String::new(),
            cursor_position: 0,
            last_refresh: Instant::now(),
        }
    }

    pub async fn init(&mut self) -> Result<()> {
        // Fetch owners to start with
        let resp = self.client.list_owners().await.map_err(|e| anyhow::anyhow!(e))?;
        self.owners = resp.data;
        if !self.owners.is_empty() {
            self.owners_state.select(Some(0));
        }

        // Fetch agents too (needed for names)
        let resp = self.client.list_agents().await.map_err(|e| anyhow::anyhow!(e))?;
        self.agents = resp.data;

        // If we have an owner_id in session, skip SelectOwner
        if let Some(owner_id) = &self.session.owner_id {
            if self.owners.iter().any(|o| o.id == *owner_id) {
                self.screen = Screen::ChatList;
                self.refresh_chats().await?;
            }
        }

        Ok(())
    }

    pub async fn refresh_chats(&mut self) -> Result<()> {
        let owner_id = match &self.session.owner_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let resp = self.client.list_chats().await.map_err(|e| anyhow::anyhow!(e))?;
        // Filter chats by owner_id (Backend should ideally do this, but we'll do it here for now)
        self.chats = resp.data.into_iter().filter(|c| c.owner_id == *owner_id).collect();
        
        if self.chats_state.selected().is_none() && !self.chats.is_empty() {
            self.chats_state.select(Some(0));
        }
        self.last_refresh = Instant::now();
        Ok(())
    }

    pub async fn refresh_messages(&mut self, chat_id: &str) -> Result<()> {
        let resp = self.client.get_chat(chat_id).await.map_err(|e| anyhow::anyhow!(e))?;
        if let Some(chat_with_msgs) = resp.data {
            self.messages = chat_with_msgs.messages;
        }
        self.last_refresh = Instant::now();
        Ok(())
    }

    pub async fn send_message(&mut self) -> Result<()> {
        if self.input.is_empty() {
            return Ok(());
        }

        if let Screen::ChatRoom(chat_id) = &self.screen {
            let owner_id = self.session.owner_id.as_ref().unwrap();
            let chat_id = chat_id.clone();
            let text = self.input.clone();
            
            self.client.send_message(&chat_id, "owner", owner_id, &text).await.map_err(|e| anyhow::anyhow!(e))?;
            
            self.input.clear();
            self.cursor_position = 0;
            self.refresh_messages(&chat_id).await?;
        }

        Ok(())
    }
}

pub fn run_tui(mut app: TuiApp) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let rt = tokio::runtime::Handle::current();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.screen {
                        Screen::SelectOwner => match key.code {
                            KeyCode::Char('q') => app.should_quit = true,
                            KeyCode::Up => {
                                let i = match app.owners_state.selected() {
                                    Some(i) => {
                                        if i == 0 {
                                            app.owners.len().saturating_sub(1)
                                        } else {
                                            i - 1
                                        }
                                    }
                                    None => 0,
                                };
                                app.owners_state.select(Some(i));
                            }
                            KeyCode::Down => {
                                let i = match app.owners_state.selected() {
                                    Some(i) => {
                                        if i >= app.owners.len().saturating_sub(1) {
                                            0
                                        } else {
                                            i + 1
                                        }
                                    }
                                    None => 0,
                                };
                                app.owners_state.select(Some(i));
                            }
                            KeyCode::Enter => {
                                if let Some(idx) = app.owners_state.selected() {
                                    let owner = &app.owners[idx];
                                    app.session.owner_id = Some(owner.id.clone());
                                    app.session.save().ok();
                                    app.screen = Screen::ChatList;
                                    rt.block_on(app.refresh_chats())?;
                                }
                            }
                            _ => {}
                        },
                        Screen::ChatList => match key.code {
                            KeyCode::Char('q') => app.should_quit = true,
                            KeyCode::Char('l') => {
                                app.screen = Screen::SelectOwner;
                            }
                            KeyCode::Up => {
                                let i = match app.chats_state.selected() {
                                    Some(i) => {
                                        if i == 0 {
                                            app.chats.len().saturating_sub(1)
                                        } else {
                                            i - 1
                                        }
                                    }
                                    None => 0,
                                };
                                app.chats_state.select(Some(i));
                            }
                            KeyCode::Down => {
                                let i = match app.chats_state.selected() {
                                    Some(i) => {
                                        if i >= app.chats.len().saturating_sub(1) {
                                            0
                                        } else {
                                            i + 1
                                        }
                                    }
                                    None => 0,
                                };
                                app.chats_state.select(Some(i));
                            }
                            KeyCode::Enter => {
                                if let Some(idx) = app.chats_state.selected() {
                                    let chat = &app.chats[idx];
                                    let chat_id = chat.id.clone();
                                    app.screen = Screen::ChatRoom(chat_id.clone());
                                    rt.block_on(app.refresh_messages(&chat_id))?;
                                }
                            }
                            _ => {}
                        },
                        Screen::ChatRoom(ref _chat_id) => match key.code {
                            KeyCode::Esc => {
                                app.screen = Screen::ChatList;
                            }
                            KeyCode::Enter => {
                                rt.block_on(app.send_message())?;
                            }
                            KeyCode::Char(c) => {
                                app.input.push(c);
                                app.cursor_position += 1;
                            }
                            KeyCode::Backspace if app.cursor_position > 0 => {
                                app.input.pop();
                                app.cursor_position -= 1;
                            }
                            _ => {}
                        },
                    }
                }
            }
        }

        // Auto-refresh in background every 2 seconds
        if app.last_refresh.elapsed() > Duration::from_secs(2) {
            match &app.screen {
                Screen::ChatList => {
                    rt.block_on(app.refresh_chats()).ok();
                }
                Screen::ChatRoom(chat_id) => {
                    let id = chat_id.clone();
                    rt.block_on(app.refresh_messages(&id)).ok();
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &mut TuiApp) {
    let size = f.size();

    match &app.screen {
        Screen::SelectOwner => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(size);

            let header = Paragraph::new("Select your profile (Owner)")
                .block(Block::default().borders(Borders::ALL).title("Human Chat"))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Cyan).bold());
            f.render_widget(header, chunks[0]);

            let items: Vec<ListItem> = app
                .owners
                .iter()
                .map(|o| {
                    ListItem::new(format!("{} ({})", o.name, o.email))
                        .style(Style::default().fg(Color::White))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Profiles"))
                .highlight_style(Style::default().bg(Color::Blue).fg(Color::White).bold())
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, chunks[1], &mut app.owners_state);
        }
        Screen::ChatList => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
                .split(size);

            let owner_name = app.owners.iter().find(|o| Some(o.id.clone()) == app.session.owner_id).map(|o| o.name.as_str()).unwrap_or("Unknown");

            let header = Paragraph::new(format!("Logged in as: {}", owner_name))
                .block(Block::default().borders(Borders::ALL).title("Chats"))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Green).bold());
            f.render_widget(header, chunks[0]);

            let items: Vec<ListItem> = app
                .chats
                .iter()
                .map(|c| {
                    let agent_name = app.agents.iter().find(|a| a.id == c.agent_id).map(|a| a.name.as_str()).unwrap_or("Agent");
                    ListItem::new(format!("{} with {}", c.title, agent_name))
                        .style(Style::default().fg(Color::White))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Select a conversation"))
                .highlight_style(Style::default().bg(Color::Green).fg(Color::White).bold())
                .highlight_symbol("💬 ");

            f.render_stateful_widget(list, chunks[1], &mut app.chats_state);

            let footer = Paragraph::new("Enter: Open Chat | l: Logout | q: Quit")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(footer, chunks[2]);
        }
        Screen::ChatRoom(chat_id) => {
            let chat = app.chats.iter().find(|c| c.id == *chat_id);
            let title = chat.map(|c| c.title.as_str()).unwrap_or("Chat");

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(size);

            let header = Paragraph::new(format!("Talking in: {}", title))
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow).bold());
            f.render_widget(header, chunks[0]);

            let messages_content: Vec<ListItem> = app
                .messages
                .iter()
                .map(|m| {
                    let sender = if m.sender_type == "owner" {
                        "Me".cyan().bold()
                    } else {
                        let name = app.agents.iter().find(|a| a.id == m.sender_id).map(|a| a.name.as_str()).unwrap_or("Agent");
                        name.yellow().bold()
                    };
                    ListItem::new(vec![
                        Line::from(vec![sender, ": ".into(), m.content.clone().into()]),
                        Line::from(format!("      {}", m.created_at).dark_gray()),
                    ])
                })
                .collect();

            let messages_list = List::new(messages_content)
                .block(Block::default().borders(Borders::ALL).title("Messages"));

            f.render_widget(messages_list, chunks[1]);

            let input = Paragraph::new(app.input.as_str())
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::ALL).title("Your message (Press Enter to send, Esc to go back)"));
            f.render_widget(input, chunks[2]);
        }
    }
}
