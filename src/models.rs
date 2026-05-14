use serde::{Deserialize, Serialize};

// ─── Agent ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
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
    pub name: Option<String>,
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
    pub title: Option<String>,
    pub agent_id: String,
    pub owner_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChatRequest {
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

// ─── Generic API response wrapper ────────────────────────────────────────────

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

    pub fn error(msg: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub success: bool,
    pub data: Vec<T>,
    pub total: usize,
}
