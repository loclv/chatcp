//! API client for the chat app backend.
//!
//! Provides methods for all 17 backend endpoints with typed responses.
//! Uses reqwest for HTTP calls and serde for deserialization.

use reqwest::Client as HttpClient;
use serde::Serialize;

use crate::config::Config;
use crate::display;
use crate::models::*;

/// Shared API client wrapping reqwest.
pub struct Client {
    http: HttpClient,
    config: Config,
}

/// Result type for CLI operations.
pub type CliResult<T> = Result<T, String>;

impl Client {
    /// Create a new client from the given configuration.
    pub fn new(config: Config) -> Self {
        Self {
            http: HttpClient::new(),
            config,
        }
    }

    /// Get the config.
    pub fn config(&self) -> &Config {
        &self.config
    }

    // ─── Generic Request Helpers ────────────────────────────────────────────

    /// Check HTTP response status and convert errors into CliResult.
    async fn check_response(&self, resp: reqwest::Response) -> CliResult<reqwest::Response> {
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&body) {
                return Err(format!(
                    "[{}] {} (HTTP {})",
                    err.code.unwrap_or_default(),
                    err.error,
                    status
                ));
            }
            return Err(format!("HTTP {}: {}", status, body));
        }
        Ok(resp)
    }

    /// Apply authorization headers if environment variables are present.
    fn apply_auth(&self, mut builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Ok(key) = std::env::var("CHAT_API_KEY") {
            builder = builder.header("Authorization", format!("ApiKey {}", key));
        } else if let Ok(token) = std::env::var("CHAT_JWT_TOKEN") {
            builder = builder.header("Authorization", format!("Bearer {}", token));
        }
        builder
    }

    /// Make a GET request and deserialize the response.
    async fn get_json<T: serde::de::DeserializeOwned>(&self, path: &str) -> CliResult<T> {
        let url = self.config.api_url(path);
        let builder = self.http.get(&url);
        let builder = self.apply_auth(builder);
        let resp = builder
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        let resp = self.check_response(resp).await?;
        resp.json::<T>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Make a POST request with a JSON body.
    async fn post_json<B: Serialize, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> CliResult<T> {
        let url = self.config.api_url(path);
        let builder = self.http.post(&url).json(body);
        let builder = self.apply_auth(builder);
        let resp = builder
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        let resp = self.check_response(resp).await?;
        resp.json::<T>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Make a PUT request with a JSON body.
    async fn put_json<B: Serialize, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> CliResult<T> {
        let url = self.config.api_url(path);
        let builder = self.http.put(&url).json(body);
        let builder = self.apply_auth(builder);
        let resp = builder
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        let resp = self.check_response(resp).await?;
        resp.json::<T>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Make a DELETE request.
    async fn delete_json<T: serde::de::DeserializeOwned>(&self, path: &str) -> CliResult<T> {
        let url = self.config.api_url(path);
        let builder = self.http.delete(&url);
        let builder = self.apply_auth(builder);
        let resp = builder
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        let resp = self.check_response(resp).await?;
        resp.json::<T>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    // ─── Health ─────────────────────────────────────────────────────────────

    /// Check backend health.
    pub async fn health(&self) -> CliResult<HealthResponse> {
        self.get_json("/api/health").await
    }

    // ─── Agents ─────────────────────────────────────────────────────────────

    /// List all agents.
    pub async fn list_agents(&self) -> CliResult<PaginatedResponse<Agent>> {
        self.get_json("/api/agents").await
    }

    /// Get an agent by ID.
    pub async fn get_agent(&self, id: &str) -> CliResult<ApiResponse<Agent>> {
        self.get_json(&format!("/api/agents/{}", id)).await
    }

    /// Create a new agent.
    pub async fn create_agent(
        &self,
        name: &str,
        description: Option<&str>,
        owner_id: Option<&str>,
    ) -> CliResult<ApiResponse<Agent>> {
        #[derive(Serialize)]
        struct CreateAgentBody<'a> {
            name: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            owner_id: Option<&'a str>,
        }

        let body = CreateAgentBody {
            name,
            description,
            owner_id,
        };
        self.post_json("/api/agents", &body).await
    }

    /// Update an agent.
    pub async fn update_agent(
        &self,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
        owner_id: Option<&str>,
    ) -> CliResult<ApiResponse<Agent>> {
        #[derive(Serialize)]
        struct UpdateAgentBody<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            owner_id: Option<&'a str>,
        }

        let body = UpdateAgentBody {
            name,
            description,
            owner_id,
        };
        self.put_json(&format!("/api/agents/{}", id), &body).await
    }

    /// Delete an agent.
    pub async fn delete_agent(&self, id: &str) -> CliResult<ApiResponse<serde_json::Value>> {
        self.delete_json(&format!("/api/agents/{}", id)).await
    }

    // ─── Owners ─────────────────────────────────────────────────────────────

    /// List all owners.
    pub async fn list_owners(&self) -> CliResult<PaginatedResponse<Owner>> {
        self.get_json("/api/owners").await
    }

    /// Get an owner by ID.
    pub async fn get_owner(&self, id: &str) -> CliResult<ApiResponse<Owner>> {
        self.get_json(&format!("/api/owners/{}", id)).await
    }

    /// Create a new owner.
    pub async fn create_owner(
        &self,
        name: &str,
        email: &str,
        password: Option<&str>,
    ) -> CliResult<ApiResponse<Owner>> {
        #[derive(Serialize)]
        struct CreateOwnerBody<'a> {
            name: &'a str,
            email: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            password: Option<&'a str>,
        }

        let body = CreateOwnerBody {
            name,
            email,
            password,
        };
        self.post_json("/api/owners", &body).await
    }

    /// Update an owner.
    pub async fn update_owner(
        &self,
        id: &str,
        name: Option<&str>,
        email: Option<&str>,
    ) -> CliResult<ApiResponse<Owner>> {
        #[derive(Serialize)]
        struct UpdateOwnerBody<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            email: Option<&'a str>,
        }

        let body = UpdateOwnerBody { name, email };
        self.put_json(&format!("/api/owners/{}", id), &body).await
    }

    /// Delete an owner.
    pub async fn delete_owner(&self, id: &str) -> CliResult<ApiResponse<serde_json::Value>> {
        self.delete_json(&format!("/api/owners/{}", id)).await
    }

    // ─── Chats ──────────────────────────────────────────────────────────────

    /// List all chats.
    pub async fn list_chats(&self) -> CliResult<PaginatedResponse<Chat>> {
        self.get_json("/api/chats").await
    }

    /// Get a chat with its messages.
    pub async fn get_chat(&self, id: &str) -> CliResult<ApiResponse<ChatWithMessages>> {
        self.get_json(&format!("/api/chats/{}", id)).await
    }

    /// Create a new chat.
    pub async fn create_chat(
        &self,
        agent_id: &str,
        owner_id: &str,
        title: Option<&str>,
    ) -> CliResult<ApiResponse<Chat>> {
        #[derive(Serialize)]
        struct CreateChatBody<'a> {
            agent_id: &'a str,
            owner_id: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            title: Option<&'a str>,
        }

        let body = CreateChatBody {
            agent_id,
            owner_id,
            title,
        };
        self.post_json("/api/chats", &body).await
    }

    /// Update a chat title.
    pub async fn update_chat(
        &self,
        id: &str,
        title: &str,
    ) -> CliResult<ApiResponse<ChatWithMessages>> {
        #[derive(Serialize)]
        struct UpdateChatBody<'a> {
            title: &'a str,
        }

        let body = UpdateChatBody { title };
        self.put_json(&format!("/api/chats/{}", id), &body).await
    }

    /// Delete a chat.
    pub async fn delete_chat(&self, id: &str) -> CliResult<ApiResponse<serde_json::Value>> {
        self.delete_json(&format!("/api/chats/{}", id)).await
    }

    // ─── Messages ───────────────────────────────────────────────────────────

    /// Send a message in a chat.
    pub async fn send_message(
        &self,
        chat_id: &str,
        sender_type: &str,
        sender_id: &str,
        content: &str,
    ) -> CliResult<ApiResponse<Message>> {
        #[derive(Serialize)]
        struct SendMessageBody<'a> {
            sender_type: &'a str,
            sender_id: &'a str,
            content: &'a str,
        }

        let body = SendMessageBody {
            sender_type,
            sender_id,
            content,
        };
        self.post_json(&format!("/api/chats/{}/messages", chat_id), &body)
            .await
    }

    /// Get messages for a chat.
    pub async fn get_messages(&self, chat_id: &str) -> CliResult<PaginatedResponse<Message>> {
        self.get_json(&format!("/api/chats/{}/messages", chat_id))
            .await
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Construct a `reqwest::Response` for testing.
    fn mock_response(status: u16, body: &str) -> reqwest::Response {
        let http_resp = http::Response::builder()
            .status(status)
            .header("content-type", "application/json")
            .body(reqwest::Body::from(body.to_string()))
            .unwrap();
        reqwest::Response::from(http_resp)
    }

    fn make_config() -> Config {
        Config {
            base_url: "http://localhost:8787".to_string(),
        }
    }

    #[tokio::test]
    async fn test_check_response_ok() {
        let client = Client::new(make_config());
        let resp = mock_response(200, r#"{"success":true}"#);
        let result = client.check_response(resp).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_response_400_with_error_body() {
        let client = Client::new(make_config());
        let resp = mock_response(
            400,
            r#"{"success":false,"error":"Bad request","code":"ERR_BAD_REQUEST"}"#,
        );
        let result = client.check_response(resp).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("ERR_BAD_REQUEST"));
        assert!(err.contains("Bad request"));
        assert!(err.contains("400"));
    }

    #[tokio::test]
    async fn test_check_response_404_without_code() {
        let client = Client::new(make_config());
        let resp = mock_response(404, r#"{"success":false,"error":"Not found","code":""}"#);
        let result = client.check_response(resp).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Not found"));
        assert!(err.contains("404"));
    }

    #[tokio::test]
    async fn test_check_response_500_with_plain_text() {
        let client = Client::new(make_config());
        let resp = mock_response(500, "Internal Server Error");
        let result = client.check_response(resp).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("500"));
        assert!(err.contains("Internal Server Error"));
    }

    #[tokio::test]
    async fn test_check_response_invalid_json_body() {
        let client = Client::new(make_config());
        // Valid JSON but not the expected ErrorResponse shape
        let resp = mock_response(400, r#"{"unexpected": "shape"}"#);
        let result = client.check_response(resp).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_client_new() {
        let config = make_config();
        let client = Client::new(config);
        assert_eq!(client.config().base_url, "http://localhost:8787");
    }

    // ─── Body serialization struct tests ───────────────────────────────────

    #[test]
    fn test_create_agent_body_serialization() {
        #[derive(Serialize)]
        struct CreateAgentBody<'a> {
            name: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            owner_id: Option<&'a str>,
        }

        let body = CreateAgentBody {
            name: "TestBot",
            description: Some("A test agent"),
            owner_id: Some("owner-1"),
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains(r#""name":"TestBot""#));
        assert!(json.contains(r#""description":"A test agent""#));
        assert!(json.contains(r#""owner_id":"owner-1""#));
    }

    #[test]
    fn test_create_agent_body_minimal() {
        #[derive(Serialize)]
        struct CreateAgentBody<'a> {
            name: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            owner_id: Option<&'a str>,
        }

        let body = CreateAgentBody {
            name: "MinimalBot",
            description: None,
            owner_id: None,
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains(r#""name":"MinimalBot""#));
        assert!(!json.contains("description"));
        assert!(!json.contains("owner_id"));
    }

    #[test]
    fn test_update_agent_body_serialization() {
        #[derive(Serialize)]
        struct UpdateAgentBody<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            owner_id: Option<&'a str>,
        }

        let body = UpdateAgentBody {
            name: Some("NewName"),
            description: None,
            owner_id: None,
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains(r#""name":"NewName""#));
        assert!(!json.contains("description"));
    }

    #[test]
    fn test_send_message_body_serialization() {
        #[derive(Serialize)]
        struct SendMessageBody<'a> {
            sender_type: &'a str,
            sender_id: &'a str,
            content: &'a str,
        }

        let body = SendMessageBody {
            sender_type: "agent",
            sender_id: "agent-1",
            content: "Hello!",
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains(r#""sender_type":"agent""#));
        assert!(json.contains(r#""content":"Hello!""#));
    }
}

// ─── High-level CLI helpers ──────────────────────────────────────────────────

impl Client {
    /// Ping the backend health endpoint and print result.
    pub async fn check_health(&self) {
        match self.health().await {
            Ok(h) => {
                display::print_success(format!(
                    "Backend is {} ({} v{})",
                    h.status,
                    h.service.as_deref().unwrap_or("chat-app-backend"),
                    h.version.as_deref().unwrap_or("?"),
                ));
            },
            Err(e) => {
                display::print_http_error(&e);
            },
        }
    }

    /// List all agents and print them.
    pub async fn print_agents(&self) {
        match self.list_agents().await {
            Ok(resp) => {
                display::print_success(format!("Found {} agent(s)", resp.pagination.total));
                display::print_agents(&resp.data);
            },
            Err(e) => display::print_api_error(&e, None),
        }
    }

    /// List all owners and print them.
    pub async fn print_owners(&self) {
        match self.list_owners().await {
            Ok(resp) => {
                display::print_success(format!("Found {} owner(s)", resp.pagination.total));
                display::print_owners(&resp.data);
            },
            Err(e) => display::print_api_error(&e, None),
        }
    }

    /// List all chats and print them.
    pub async fn print_chats(&self) {
        match self.list_chats().await {
            Ok(resp) => {
                display::print_success(format!("Found {} chat(s)", resp.pagination.total));
                display::print_chats(&resp.data);
            },
            Err(e) => display::print_api_error(&e, None),
        }
    }

    /// Print a chat with its messages.
    pub async fn print_chat(&self, id: &str) {
        match self.get_chat(id).await {
            Ok(resp) => {
                if let Some(chat) = resp.data {
                    display::print_chat_with_messages(&chat);
                }
            },
            Err(e) => display::print_api_error(&e, None),
        }
    }

    /// Send a message and print the result.
    pub async fn send_and_print(
        &self,
        chat_id: &str,
        sender_type: &str,
        sender_id: &str,
        content: &str,
    ) {
        match self
            .send_message(chat_id, sender_type, sender_id, content)
            .await
        {
            Ok(resp) => {
                if let Some(msg) = resp.data {
                    display::print_success("Message sent!");
                    display::print_message(&msg);
                }
            },
            Err(e) => display::print_api_error(&e, None),
        }
    }

    // ─── Authentication & Key Rotation ──────────────────────────────────────

    /// Login with email and password to retrieve JWT.
    pub async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> CliResult<ApiResponse<LoginResponseData>> {
        #[derive(Serialize)]
        struct LoginBody<'a> {
            email: &'a str,
            password: &'a str,
        }
        let body = LoginBody { email, password };
        self.post_json("/api/auth/login", &body).await
    }

    /// Retrieve logged in user's profile.
    pub async fn get_me(&self) -> CliResult<ApiResponse<serde_json::Value>> {
        self.get_json("/api/auth/me").await
    }

    /// Rotate owner API key.
    pub async fn rotate_owner_key(
        &self,
        owner_id: &str,
    ) -> CliResult<ApiResponse<serde_json::Value>> {
        self.post_json(
            &format!("/api/owners/{}/key", owner_id),
            &serde_json::json!({}),
        )
        .await
    }

    /// Rotate agent API key.
    pub async fn rotate_agent_key(
        &self,
        agent_id: &str,
    ) -> CliResult<ApiResponse<serde_json::Value>> {
        self.post_json(
            &format!("/api/agents/{}/key", agent_id),
            &serde_json::json!({}),
        )
        .await
    }
}
