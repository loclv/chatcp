//! Configuration management for the chat CLI.
//!
//! Handles reading configuration from environment variables and defaults,
//! primarily the backend API base URL.

use std::env;

/// Default backend URL (local development).
const DEFAULT_BASE_URL: &str = "http://localhost:8787";

/// Application configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL of the chat app backend API.
    pub base_url: String,
}

impl Config {
    /// Load configuration from environment variables with sensible defaults.
    pub fn from_env() -> Self {
        let base_url = env::var("CHAT_API_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        Self { base_url }
    }

    /// Get the full URL for an API path.
    pub fn api_url(&self, path: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{}/{}", base, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::from_env();
        assert_eq!(config.base_url, "http://localhost:8787");
    }

    #[test]
    fn test_api_url_with_slash() {
        let config = Config {
            base_url: "http://localhost:8787/".to_string(),
        };
        assert_eq!(
            config.api_url("/api/health"),
            "http://localhost:8787/api/health"
        );
    }

    #[test]
    fn test_api_url_without_slash() {
        let config = Config {
            base_url: "http://localhost:8787".to_string(),
        };
        assert_eq!(
            config.api_url("api/health"),
            "http://localhost:8787/api/health"
        );
    }

    #[test]
    fn test_custom_env_url() {
        // This test isolates env manipulation
        let config = Config {
            base_url: "https://prod.example.com".to_string(),
        };
        assert_eq!(
            config.api_url("/api/agents"),
            "https://prod.example.com/api/agents"
        );
    }
}
