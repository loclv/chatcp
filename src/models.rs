use serde::{Deserialize, Serialize};
use worker::*;

// ─── SenderType Enum ─────────────────────────────────────────────────────────

/// Represents the type of message sender.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SenderType {
    Agent,
    Owner,
}

impl SenderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SenderType::Agent => "agent",
            SenderType::Owner => "owner",
        }
    }

    /// Parse a sender type from a string.
    #[allow(unused)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "agent" => Some(SenderType::Agent),
            "owner" => Some(SenderType::Owner),
            _ => None,
        }
    }
}

impl std::fmt::Display for SenderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ─── AppError Enum ───────────────────────────────────────────────────────────

/// Structured error type for the application.
/// Automatically converts into an HTTP Response with appropriate status codes
/// and structured JSON error payloads.
#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Validation(String),
    Database(String),
    Internal(String),
}

impl AppError {
    /// Returns the HTTP status code for this error.
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::NotFound(_) => 404,
            AppError::BadRequest(_) | AppError::Validation(_) => 400,
            AppError::Database(_) | AppError::Internal(_) => 500,
        }
    }

    /// Returns a machine-readable error code string.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::NotFound(_) => "ERR_NOT_FOUND",
            AppError::BadRequest(_) => "ERR_BAD_REQUEST",
            AppError::Validation(_) => "ERR_VALIDATION",
            AppError::Database(_) => "ERR_DATABASE",
            AppError::Internal(_) => "ERR_INTERNAL",
        }
    }

    /// Returns the human-readable error message.
    pub fn message(&self) -> &str {
        match self {
            AppError::NotFound(msg)
            | AppError::BadRequest(msg)
            | AppError::Validation(msg)
            | AppError::Database(msg)
            | AppError::Internal(msg) => msg,
        }
    }

    /// Convert this error into an HTTP Response with a structured JSON body.
    pub fn into_response(self) -> Result<Response> {
        // Log internal and database errors for debugging
        if matches!(self, AppError::Internal(_) | AppError::Database(_)) {
            console_error!("[{}] {}", self.code(), self.message());
        }

        let body = ErrorResponse {
            success: false,
            data: None,
            error: self.message().to_string(),
            code: self.code().to_string(),
        };

        Response::from_json(&body).map(|r| r.with_status(self.status_code()))
    }
}

/// Convert `worker::Error` into `AppError::Internal` for use with the `?` operator.
impl From<worker::Error> for AppError {
    fn from(e: worker::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

/// Convert a string error into `AppError::Internal`.
impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::Internal(msg)
    }
}

/// Convert an &str into `AppError::Internal`.
impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::Internal(msg.to_string())
    }
}

// ─── Agent ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub owner_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub owner_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAgentRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub owner_id: Option<String>,
}

// ─── Owner ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    pub id: String,
    pub name: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateOwnerRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOwnerRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
}

// ─── Chat ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: String,
    pub title: String,
    pub agent_id: String,
    pub owner_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateChatRequest {
    #[serde(default)]
    pub title: Option<String>,
    pub agent_id: String,
    pub owner_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChatRequest {
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatWithMessages {
    #[serde(flatten)]
    pub chat: Chat,
    pub messages: Vec<Message>,
}

// ─── Message ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub chat_id: String,
    pub sender_type: String,
    pub sender_id: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub sender_type: String,
    pub sender_id: String,
    pub content: String,
}

// ─── API Response Types ──────────────────────────────────────────────────────

/// Standard success response wrapper.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
}

/// Paginated list response.
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub success: bool,
    pub data: Vec<T>,
    pub total: usize,
}

/// Structured error response with a machine-readable error code.
/// Maintains the same structure as `ApiResponse` for consistency,
/// with the addition of a `code` field for machine-readable error types.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: String,
    pub code: String,
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─── SenderType ─────────────────────────────────────────────────────────

    #[test]
    fn test_sender_type_as_str() {
        assert_eq!(SenderType::Agent.as_str(), "agent");
        assert_eq!(SenderType::Owner.as_str(), "owner");
    }

    #[test]
    fn test_sender_type_from_str() {
        assert_eq!(SenderType::from_str("agent"), Some(SenderType::Agent));
        assert_eq!(SenderType::from_str("owner"), Some(SenderType::Owner));
        assert_eq!(SenderType::from_str("unknown"), None);
        assert_eq!(SenderType::from_str(""), None);
    }

    #[test]
    fn test_sender_type_display() {
        assert_eq!(format!("{}", SenderType::Agent), "agent");
        assert_eq!(format!("{}", SenderType::Owner), "owner");
    }

    #[test]
    fn test_sender_type_equality() {
        assert_eq!(SenderType::Agent, SenderType::Agent);
        assert_ne!(SenderType::Agent, SenderType::Owner);
    }

    // ─── AppError ───────────────────────────────────────────────────────────

    #[test]
    fn test_app_error_not_found() {
        let err = AppError::NotFound("Agent 'x' not found".to_string());
        assert_eq!(err.status_code(), 404);
        assert_eq!(err.code(), "ERR_NOT_FOUND");
        assert_eq!(err.message(), "Agent 'x' not found");
    }

    #[test]
    fn test_app_error_bad_request() {
        let err = AppError::BadRequest("Invalid JSON".to_string());
        assert_eq!(err.status_code(), 400);
        assert_eq!(err.code(), "ERR_BAD_REQUEST");
        assert_eq!(err.message(), "Invalid JSON");
    }

    #[test]
    fn test_app_error_validation() {
        let err = AppError::Validation("Name is required".to_string());
        assert_eq!(err.status_code(), 400);
        assert_eq!(err.code(), "ERR_VALIDATION");
        assert_eq!(err.message(), "Name is required");
    }

    #[test]
    fn test_app_error_database() {
        let err = AppError::Database("Connection failed".to_string());
        assert_eq!(err.status_code(), 500);
        assert_eq!(err.code(), "ERR_DATABASE");
        assert_eq!(err.message(), "Connection failed");
    }

    #[test]
    fn test_app_error_internal() {
        let err = AppError::Internal("Unexpected error".to_string());
        assert_eq!(err.status_code(), 500);
        assert_eq!(err.code(), "ERR_INTERNAL");
        assert_eq!(err.message(), "Unexpected error");
    }

    #[test]
    fn test_app_error_from_string() {
        let err: AppError = "something went wrong".into();
        assert_eq!(err.status_code(), 500);
        assert_eq!(err.code(), "ERR_INTERNAL");
        assert_eq!(err.message(), "something went wrong");
    }

    // ─── API Response types ─────────────────────────────────────────────────

    #[test]
    fn test_api_response_success() {
        let resp = ApiResponse::success(42);
        assert!(resp.success);
        assert_eq!(resp.data, Some(42));
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_error_response_serialization() {
        let err = ErrorResponse {
            success: false,
            data: None,
            error: "Not found".to_string(),
            code: "ERR_NOT_FOUND".to_string(),
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains(r#""success":false"#));
        assert!(json.contains(r#""error":"Not found""#));
        assert!(json.contains(r#""code":"ERR_NOT_FOUND""#));
    }

    #[test]
    fn test_agent_serialization() {
        let agent = Agent {
            id: "abc-123".to_string(),
            name: "TestBot".to_string(),
            description: "A test agent".to_string(),
            owner_id: Some("owner-456".to_string()),
            created_at: "2025-01-01 00:00:00".to_string(),
            updated_at: "2025-01-01 00:00:00".to_string(),
        };
        let json = serde_json::to_string(&agent).unwrap();
        assert!(json.contains(r#""name":"TestBot""#));
        assert!(json.contains(r#""owner_id":"owner-456""#));
    }

    #[test]
    fn test_create_agent_request_deserialize() {
        let json = r#"{"name":"TestBot","description":"A test agent","owner_id":null}"#;
        let req: CreateAgentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "TestBot");
        assert!(req.owner_id.is_none());
    }

    #[test]
    fn test_send_message_request_deserialize() {
        let json = r#"{"sender_type":"agent","sender_id":"abc-123","content":"Hello!"}"#;
        let req: SendMessageRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.sender_type, "agent");
        assert_eq!(req.content, "Hello!");
    }

    #[test]
    fn test_paginated_response_serialization() {
        let items = vec![1, 2, 3];
        let resp = PaginatedResponse {
            success: true,
            total: 3,
            data: items,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains(r#""success":true"#));
        assert!(json.contains(r#""total":3"#));
    }
}
