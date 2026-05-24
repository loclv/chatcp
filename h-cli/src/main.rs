use anyhow::Result;
use clap::Parser;

mod session;
mod tui;

use crate::session::Session;
use crate::tui::{run_tui, TuiApp};
use chat_cli::client::Client;
use chat_cli::config::Config;

/// Human Chat CLI — a beautiful TUI for chatting with your agents.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Backend API URL (defaults to http://localhost:8787)
    #[arg(short, long, env = "CHAT_API_URL")]
    api_url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Load or initialize session
    let mut session = Session::load();
    if let Some(url) = args.api_url {
        session.api_url = url;
    }

    // Initialize API client
    let config = Config {
        base_url: session.api_url.clone(),
    };
    let client = Client::new(config);

    // Initialize TUI App
    let mut app = TuiApp::new(&client, session);
    app.init().await?;

    // Run the TUI
    run_tui(app)?;

    Ok(())
}
