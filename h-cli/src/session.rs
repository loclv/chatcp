use serde::{Deserialize, Serialize};

/// Session configuration for the human-chat CLI.
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    /// The ID of the owner currently logged in.
    pub owner_id: Option<String>,
    /// The backend API URL.
    pub api_url: String,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            owner_id: None,
            api_url: "http://localhost:8787".to_string(),
        }
    }
}

impl Session {
    /// Load the session from the default config location.
    pub fn load() -> Self {
        confy::load("chat-app", "human-session").unwrap_or_default()
    }

    /// Save the session to the default config location.
    pub fn save(&self) -> Result<(), anyhow::Error> {
        confy::store("chat-app", "human-session", self).map_err(anyhow::Error::from)
    }
}
