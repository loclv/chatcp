//! Data models matching the chat app backend API responses.

use serde::Deserialize;

// ─── Generic API Response Wrappers ───────────────────────────────────────────

/// Standard API response for single resources.
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    #[allow(dead_code)]
    pub success: bool,
    pub data: Option<T>,
    #[allow(dead_code)]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationMetadata {
    pub limit: u32,
    pub offset: u32,
    pub total: usize,
    pub has_more: bool,
}

/// Paginated API response for list endpoints.
#[derive(Debug, Deserialize)]
pub struct PaginatedResponse<T> {
    #[allow(dead_code)]
    pub success: bool,
    pub data: Vec<T>,
    pub pagination: PaginationMetadata,
}

/// Error response from the API (v0.2.0+ format with error code).
#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    #[allow(dead_code)]
    pub success: bool,
    pub error: String,
    pub code: Option<String>,
}

// ─── Entity Types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub owner_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Owner {
    pub id: String,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginResponseData {
    pub token: String,
    pub owner: Owner,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Chat {
    pub id: String,
    pub title: String,
    pub agent_id: String,
    pub owner_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    #[allow(dead_code)]
    pub id: String,
    #[allow(dead_code)]
    pub chat_id: String,
    pub sender_type: String,
    pub sender_id: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatWithMessages {
    pub id: String,
    pub title: String,
    pub agent_id: String,
    pub owner_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub messages: Vec<Message>,
}

// ─── Health Check ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: Option<String>,
    pub version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Agent deserialization ──────────────────────────────────────────────

    #[test]
    fn test_agent_deserialize_full() {
        let json = r#"{
            "id": "abc-123",
            "name": "TestBot",
            "description": "A test agent",
            "owner_id": "owner-456",
            "created_at": "2025-01-01 00:00:00",
            "updated_at": "2025-01-01 00:00:00"
        }"#;
        let agent: Agent = serde_json::from_str(json).unwrap();
        assert_eq!(agent.id, "abc-123");
        assert_eq!(agent.name, "TestBot");
        assert_eq!(agent.owner_id, Some("owner-456".to_string()));
    }

    #[test]
    fn test_agent_deserialize_without_owner() {
        let json = r#"{
            "id": "abc-123",
            "name": "SoloBot",
            "description": "",
            "owner_id": null,
            "created_at": "2025-01-01 00:00:00",
            "updated_at": "2025-01-01 00:00:00"
        }"#;
        let agent: Agent = serde_json::from_str(json).unwrap();
        assert_eq!(agent.name, "SoloBot");
        assert!(agent.owner_id.is_none());
    }

    #[test]
    fn test_agent_deserialize_missing_description() {
        // description has #[serde(default)] so missing field is empty string
        let json = r#"{
            "id": "abc-123",
            "name": "MinimalBot",
            "owner_id": null,
            "created_at": "2025-01-01 00:00:00",
            "updated_at": "2025-01-01 00:00:00"
        }"#;
        let agent: Agent = serde_json::from_str(json).unwrap();
        assert_eq!(agent.description, "");
    }

    // ─── Owner deserialization ──────────────────────────────────────────────

    #[test]
    fn test_owner_deserialize() {
        let json = r#"{
            "id": "owner-1",
            "name": "Alice",
            "email": "alice@example.com",
            "created_at": "2025-01-01 00:00:00"
        }"#;
        let owner: Owner = serde_json::from_str(json).unwrap();
        assert_eq!(owner.name, "Alice");
        assert_eq!(owner.email, "alice@example.com");
    }

    // ─── Chat deserialization ───────────────────────────────────────────────

    #[test]
    fn test_chat_deserialize() {
        let json = r#"{
            "id": "chat-1",
            "title": "Help Chat",
            "agent_id": "agent-1",
            "owner_id": "owner-1",
            "created_at": "2025-01-01 00:00:00",
            "updated_at": "2025-01-01 00:00:00"
        }"#;
        let chat: Chat = serde_json::from_str(json).unwrap();
        assert_eq!(chat.title, "Help Chat");
        assert_eq!(chat.agent_id, "agent-1");
    }

    // ─── Message deserialization ────────────────────────────────────────────

    #[test]
    fn test_message_deserialize() {
        let json = r#"{
            "id": "msg-1",
            "chat_id": "chat-1",
            "sender_type": "agent",
            "sender_id": "agent-1",
            "content": "Hello!",
            "created_at": "2025-01-01 00:00:00"
        }"#;
        let msg: Message = serde_json::from_str(json).unwrap();
        assert_eq!(msg.sender_type, "agent");
        assert_eq!(msg.content, "Hello!");
        assert_eq!(msg.chat_id, "chat-1");
    }

    // ─── ChatWithMessages deserialization ───────────────────────────────────

    #[test]
    fn test_chat_with_messages_deserialize() {
        let json = r#"{
            "id": "chat-1",
            "title": "Help Chat",
            "agent_id": "agent-1",
            "owner_id": "owner-1",
            "created_at": "2025-01-01 00:00:00",
            "updated_at": "2025-01-01 00:00:00",
            "messages": [
                {
                    "id": "msg-1",
                    "chat_id": "chat-1",
                    "sender_type": "agent",
                    "sender_id": "agent-1",
                    "content": "Hello!",
                    "created_at": "2025-01-01 00:00:00"
                },
                {
                    "id": "msg-2",
                    "chat_id": "chat-1",
                    "sender_type": "owner",
                    "sender_id": "owner-1",
                    "content": "Hi there!",
                    "created_at": "2025-01-01 00:01:00"
                }
            ]
        }"#;
        let chat: ChatWithMessages = serde_json::from_str(json).unwrap();
        assert_eq!(chat.title, "Help Chat");
        assert_eq!(chat.messages.len(), 2);
        assert_eq!(chat.messages[0].sender_type, "agent");
        assert_eq!(chat.messages[1].content, "Hi there!");
    }

    #[test]
    fn test_chat_with_messages_empty() {
        let json = r#"{
            "id": "chat-1",
            "title": "Empty Chat",
            "agent_id": "agent-1",
            "owner_id": "owner-1",
            "created_at": "2025-01-01 00:00:00",
            "updated_at": "2025-01-01 00:00:00",
            "messages": []
        }"#;
        let chat: ChatWithMessages = serde_json::from_str(json).unwrap();
        assert!(chat.messages.is_empty());
    }

    // ─── API Response wrappers ──────────────────────────────────────────────

    #[test]
    fn test_api_response_deserialize_success() {
        let json = r#"{"success": true, "data": {"id": "abc", "name": "Test", "owner_id": null}, "error": null}"#;
        let resp: ApiResponse<serde_json::Value> = serde_json::from_str(json).unwrap();
        assert!(resp.success);
        assert!(resp.data.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_api_response_deserialize_error() {
        let json = r#"{"success": false, "data": null, "error": "Something went wrong"}"#;
        let resp: ApiResponse<serde_json::Value> = serde_json::from_str(json).unwrap();
        assert!(!resp.success);
        assert!(resp.data.is_none());
        assert_eq!(resp.error, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_paginated_response_deserialize() {
        let json = r#"{"success": true, "data": [1, 2, 3], "pagination": {"limit": 50, "offset": 0, "total": 3, "has_more": false}}"#;
        let resp: PaginatedResponse<i32> = serde_json::from_str(json).unwrap();
        assert!(resp.success);
        assert_eq!(resp.pagination.total, 3);
        assert_eq!(resp.data.len(), 3);
    }

    #[test]
    fn test_paginated_response_empty() {
        let json = r#"{"success": true, "data": [], "pagination": {"limit": 50, "offset": 0, "total": 0, "has_more": false}}"#;
        let resp: PaginatedResponse<i32> = serde_json::from_str(json).unwrap();
        assert!(resp.success);
        assert_eq!(resp.pagination.total, 0);
        assert!(resp.data.is_empty());
    }

    // ─── Error response ─────────────────────────────────────────────────────

    #[test]
    fn test_error_response_with_code() {
        let json = r#"{"success": false, "error": "Not found", "code": "ERR_NOT_FOUND"}"#;
        let err: ErrorResponse = serde_json::from_str(json).unwrap();
        assert!(!err.success);
        assert_eq!(err.error, "Not found");
        assert_eq!(err.code, Some("ERR_NOT_FOUND".to_string()));
    }

    #[test]
    fn test_error_response_without_code() {
        // Backward compatibility: old format without code field
        let json = r#"{"success": false, "error": "Server error"}"#;
        let err: ErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(err.error, "Server error");
        assert!(err.code.is_none());
    }

    // ─── Health response ────────────────────────────────────────────────────

    #[test]
    fn test_health_response_deserialize() {
        let json = r#"{"status": "ok", "service": "chat-app-backend", "version": "0.2.0"}"#;
        let health: HealthResponse = serde_json::from_str(json).unwrap();
        assert_eq!(health.status, "ok");
        assert_eq!(health.service, Some("chat-app-backend".to_string()));
        assert_eq!(health.version, Some("0.2.0".to_string()));
    }

    #[test]
    fn test_health_response_minimal() {
        let json = r#"{"status": "ok"}"#;
        let health: HealthResponse = serde_json::from_str(json).unwrap();
        assert_eq!(health.status, "ok");
        assert!(health.service.is_none());
        assert!(health.version.is_none());
    }
}
