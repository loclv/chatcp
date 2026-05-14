use wasm_bindgen::JsValue;
use worker::*;
use uuid::Uuid;

use crate::prelude::*;

// ─── Helper ──────────────────────────────────────────────────────────────────

/// Generate a UUID v4 string for use as a primary key.
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// Convert an `Option<String>` to a `JsValue` (null if None).
fn optional_js_value(opt: &Option<String>) -> JsValue {
    match opt {
        Some(val) => JsValue::from_str(val.as_str()),
        None => JsValue::null(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id_is_uuid_format() {
        let id = generate_id();
        // UUID v4 format: 8-4-4-4-12 hex chars (36 total)
        assert_eq!(id.len(), 36);
        let bytes = id.as_bytes();
        assert_eq!(bytes[8], b'-');
        assert_eq!(bytes[13], b'-');
        assert_eq!(bytes[18], b'-');
        assert_eq!(bytes[23], b'-');
        // Version nibble should be '4'
        assert_eq!(bytes[14], b'4');
    }

    #[test]
    fn test_generate_id_is_unique() {
        let id1 = generate_id();
        let id2 = generate_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_id_hex_characters() {
        let id = generate_id();
        let bytes = id.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            if i == 8 || i == 13 || i == 18 || i == 23 {
                continue; // dashes
            }
            assert!(b.is_ascii_hexdigit(), "Non-hex char '{}' at pos {}", b as char, i);
        }
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_optional_js_value_some() {
        let result = optional_js_value(&Some("hello".to_string()));
        assert_eq!(result.as_string(), Some("hello".to_string()));
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_optional_js_value_none() {
        let result = optional_js_value(&None);
        assert!(result.is_null());
    }
}

// ─── Agents ──────────────────────────────────────────────────────────────────

pub async fn create_agent(d1: &D1Database, req: &CreateAgentRequest) -> Result<Response> {
    let id = generate_id();
    let description = req.description.as_deref().unwrap_or("");

    let result = d1
        .prepare(
            "INSERT INTO agents (id, name, description, owner_id, created_at, updated_at) \
             VALUES (?, ?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(&[
            id.as_str().into(),
            req.name.as_str().into(),
            description.into(),
            optional_js_value(&req.owner_id),
        ])?
        .run()
        .await;

    match result {
        Ok(_) => match get_agent_by_id(d1, &id).await? {
            Some(agent) => {
                let resp = ApiResponse::success(agent);
                Response::from_json(&resp).map(|r| r.with_status(201))
            }
            None => AppError::Internal("Failed to retrieve created agent".to_string()).into_response(),
        },
        Err(e) => {
            AppError::Database(format!("Failed to create agent: {}", e)).into_response()
        }
    }
}

pub async fn list_agents(d1: &D1Database) -> Result<Response> {
    let result = d1
        .prepare("SELECT * FROM agents ORDER BY created_at DESC")
        .all()
        .await;

    match result {
        Ok(agents) => {
            let results: Vec<Agent> = agents.results()?;
            let total = results.len();
            let resp = PaginatedResponse {
                success: true,
                total,
                data: results,
            };
            Response::from_json(&resp)
        }
        Err(e) => AppError::Database(format!("Failed to list agents: {}", e)).into_response(),
    }
}

pub async fn get_agent(d1: &D1Database, id: &str) -> Result<Response> {
    match get_agent_by_id(d1, id).await {
        Ok(Some(agent)) => {
            let resp = ApiResponse::success(agent);
            Response::from_json(&resp)
        }
        Ok(None) => AppError::NotFound(format!("Agent '{}' not found", id)).into_response(),
        Err(e) => AppError::Database(e.to_string()).into_response(),
    }
}

async fn get_agent_by_id(d1: &D1Database, id: &str) -> Result<Option<Agent>> {
    let result = d1
        .prepare("SELECT * FROM agents WHERE id = ?")
        .bind(&[id.into()])?
        .first::<Agent>(None)
        .await?;
    Ok(result)
}

pub async fn update_agent(d1: &D1Database, id: &str, req: &UpdateAgentRequest) -> Result<Response> {
    let existing = match get_agent_by_id(d1, id).await? {
        Some(a) => a,
        None => return AppError::NotFound(format!("Agent '{}' not found", id)).into_response(),
    };

    let name = req.name.as_deref().unwrap_or(&existing.name);
    let description = req.description.as_deref().unwrap_or(&existing.description);

    let result = d1
        .prepare(
            "UPDATE agents SET name = ?, description = ?, owner_id = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(&[
            name.into(),
            description.into(),
            optional_js_value(&req.owner_id),
            id.into(),
        ])?
        .run()
        .await;

    match result {
        Ok(_) => get_agent(d1, id).await,
        Err(e) => AppError::Database(format!("Failed to update agent: {}", e)).into_response(),
    }
}

pub async fn delete_agent(d1: &D1Database, id: &str) -> Result<Response> {
    let result = d1
        .prepare("DELETE FROM agents WHERE id = ?")
        .bind(&[id.into()])?
        .run()
        .await;

    match result {
        Ok(meta) => {
            let changes = meta.meta()?.and_then(|m| m.changes).unwrap_or(0);
            if changes > 0 {
                let resp = ApiResponse::success(serde_json::json!({"deleted": true}));
                Response::from_json(&resp)
            } else {
                AppError::NotFound(format!("Agent '{}' not found", id)).into_response()
            }
        }
        Err(e) => AppError::Database(format!("Failed to delete agent: {}", e)).into_response(),
    }
}

// ─── Owners ──────────────────────────────────────────────────────────────────

pub async fn create_owner(d1: &D1Database, req: &CreateOwnerRequest) -> Result<Response> {
    let id = generate_id();

    let result = d1
        .prepare(
            "INSERT INTO owners (id, name, email, created_at) VALUES (?, ?, ?, datetime('now'))",
        )
        .bind(&[id.as_str().into(), req.name.as_str().into(), req.email.as_str().into()])?
        .run()
        .await;

    match result {
        Ok(_) => match get_owner_by_id(d1, &id).await? {
            Some(owner) => {
                let resp = ApiResponse::success(owner);
                Response::from_json(&resp).map(|r| r.with_status(201))
            }
            None => AppError::Internal("Failed to retrieve created owner".to_string()).into_response(),
        },
        Err(e) => {
            let msg = format!("{}", e);
            if msg.contains("UNIQUE") {
                return AppError::BadRequest("An owner with this email already exists".to_string())
                    .into_response();
            }
            AppError::Database(format!("Failed to create owner: {}", e)).into_response()
        }
    }
}

pub async fn list_owners(d1: &D1Database) -> Result<Response> {
    let result = d1
        .prepare("SELECT * FROM owners ORDER BY created_at DESC")
        .all()
        .await;

    match result {
        Ok(owners) => {
            let results: Vec<Owner> = owners.results()?;
            let total = results.len();
            let resp = PaginatedResponse {
                success: true,
                total,
                data: results,
            };
            Response::from_json(&resp)
        }
        Err(e) => AppError::Database(format!("Failed to list owners: {}", e)).into_response(),
    }
}

pub async fn get_owner(d1: &D1Database, id: &str) -> Result<Response> {
    match get_owner_by_id(d1, id).await {
        Ok(Some(owner)) => {
            let resp = ApiResponse::success(owner);
            Response::from_json(&resp)
        }
        Ok(None) => AppError::NotFound(format!("Owner '{}' not found", id)).into_response(),
        Err(e) => AppError::Database(e.to_string()).into_response(),
    }
}

async fn get_owner_by_id(d1: &D1Database, id: &str) -> Result<Option<Owner>> {
    let result = d1
        .prepare("SELECT * FROM owners WHERE id = ?")
        .bind(&[id.into()])?
        .first::<Owner>(None)
        .await?;
    Ok(result)
}

pub async fn update_owner(d1: &D1Database, id: &str, req: &UpdateOwnerRequest) -> Result<Response> {
    let existing = match get_owner_by_id(d1, id).await? {
        Some(o) => o,
        None => return AppError::NotFound(format!("Owner '{}' not found", id)).into_response(),
    };

    let name = req.name.as_deref().unwrap_or(&existing.name);
    let email = req.email.as_deref().unwrap_or(&existing.email);

    let result = d1
        .prepare("UPDATE owners SET name = ?, email = ? WHERE id = ?")
        .bind(&[name.into(), email.into(), id.into()])?
        .run()
        .await;

    match result {
        Ok(_) => get_owner(d1, id).await,
        Err(e) => AppError::Database(format!("Failed to update owner: {}", e)).into_response(),
    }
}

pub async fn delete_owner(d1: &D1Database, id: &str) -> Result<Response> {
    let result = d1
        .prepare("DELETE FROM owners WHERE id = ?")
        .bind(&[id.into()])?
        .run()
        .await;

    match result {
        Ok(meta) => {
            let changes = meta.meta()?.and_then(|m| m.changes).unwrap_or(0);
            if changes > 0 {
                let resp = ApiResponse::success(serde_json::json!({"deleted": true}));
                Response::from_json(&resp)
            } else {
                AppError::NotFound(format!("Owner '{}' not found", id)).into_response()
            }
        }
        Err(e) => AppError::Database(format!("Failed to delete owner: {}", e)).into_response(),
    }
}

// ─── Chats ───────────────────────────────────────────────────────────────────

pub async fn create_chat(d1: &D1Database, req: &CreateChatRequest) -> Result<Response> {
    let id = generate_id();
    let title = req.title.as_deref().unwrap_or(DEFAULT_CHAT_TITLE);

    let result = d1
        .prepare(
            "INSERT INTO chats (id, title, agent_id, owner_id, created_at, updated_at) \
             VALUES (?, ?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(&[
            id.as_str().into(),
            title.into(),
            req.agent_id.as_str().into(),
            req.owner_id.as_str().into(),
        ])?
        .run()
        .await;

    match result {
        Ok(_) => match get_chat_by_id(d1, &id).await? {
            Some(chat) => {
                let resp = ApiResponse::success(chat);
                Response::from_json(&resp).map(|r| r.with_status(201))
            }
            None => AppError::Internal("Failed to retrieve created chat".to_string()).into_response(),
        },
        Err(e) => AppError::Database(format!("Failed to create chat: {}", e)).into_response(),
    }
}

pub async fn list_chats(d1: &D1Database) -> Result<Response> {
    let result = d1
        .prepare("SELECT * FROM chats ORDER BY updated_at DESC")
        .all()
        .await;

    match result {
        Ok(chats) => {
            let results: Vec<Chat> = chats.results()?;
            let total = results.len();
            let resp = PaginatedResponse {
                success: true,
                total,
                data: results,
            };
            Response::from_json(&resp)
        }
        Err(e) => AppError::Database(format!("Failed to list chats: {}", e)).into_response(),
    }
}

async fn get_chat_by_id(d1: &D1Database, id: &str) -> Result<Option<Chat>> {
    let result = d1
        .prepare("SELECT * FROM chats WHERE id = ?")
        .bind(&[id.into()])?
        .first::<Chat>(None)
        .await?;
    Ok(result)
}

pub async fn get_chat_with_messages(d1: &D1Database, id: &str) -> Result<Response> {
    let chat = match get_chat_by_id(d1, id).await? {
        Some(c) => c,
        None => return AppError::NotFound(format!("Chat '{}' not found", id)).into_response(),
    };

    let messages_result = d1
        .prepare("SELECT * FROM messages WHERE chat_id = ? ORDER BY created_at ASC")
        .bind(&[id.into()])?
        .all()
        .await;

    match messages_result {
        Ok(messages) => {
            let results: Vec<Message> = messages.results()?;
            let chat_with_msgs = ChatWithMessages {
                chat,
                messages: results,
            };
            let resp = ApiResponse::success(chat_with_msgs);
            Response::from_json(&resp)
        }
        Err(e) => AppError::Database(format!("Failed to get messages: {}", e)).into_response(),
    }
}

pub async fn update_chat(d1: &D1Database, id: &str, req: &UpdateChatRequest) -> Result<Response> {
    let existing = match get_chat_by_id(d1, id).await? {
        Some(c) => c,
        None => return AppError::NotFound(format!("Chat '{}' not found", id)).into_response(),
    };

    let title = req.title.as_deref().unwrap_or(&existing.title);

    let result = d1
        .prepare("UPDATE chats SET title = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(&[title.into(), id.into()])?
        .run()
        .await;

    match result {
        Ok(_) => get_chat_with_messages(d1, id).await,
        Err(e) => AppError::Database(format!("Failed to update chat: {}", e)).into_response(),
    }
}

pub async fn delete_chat(d1: &D1Database, id: &str) -> Result<Response> {
    let result = d1
        .prepare("DELETE FROM chats WHERE id = ?")
        .bind(&[id.into()])?
        .run()
        .await;

    match result {
        Ok(meta) => {
            let changes = meta.meta()?.and_then(|m| m.changes).unwrap_or(0);
            if changes > 0 {
                let resp = ApiResponse::success(serde_json::json!({"deleted": true}));
                Response::from_json(&resp)
            } else {
                AppError::NotFound(format!("Chat '{}' not found", id)).into_response()
            }
        }
        Err(e) => AppError::Database(format!("Failed to delete chat: {}", e)).into_response(),
    }
}

// ─── Messages ────────────────────────────────────────────────────────────────

pub async fn send_message(d1: &D1Database, chat_id: &str, req: &SendMessageRequest) -> Result<Response> {
    // Verify chat exists
    if get_chat_by_id(d1, chat_id).await?.is_none() {
        return AppError::NotFound(format!("Chat '{}' not found", chat_id)).into_response();
    }

    let id = generate_id();

    let result = d1
        .prepare(
            "INSERT INTO messages (id, chat_id, sender_type, sender_id, content, created_at) \
             VALUES (?, ?, ?, ?, ?, datetime('now'))",
        )
        .bind(&[
            id.as_str().into(),
            chat_id.into(),
            req.sender_type.as_str().into(),
            req.sender_id.as_str().into(),
            req.content.as_str().into(),
        ])?
        .run()
        .await;

    // Touch the chat's updated_at timestamp
    let _ = d1
        .prepare("UPDATE chats SET updated_at = datetime('now') WHERE id = ?")
        .bind(&[chat_id.into()])?
        .run()
        .await;

    match result {
        Ok(_) => {
            let msg = d1
                .prepare("SELECT * FROM messages WHERE id = ?")
                .bind(&[id.as_str().into()])?
                .first::<Message>(None)
                .await?;

            match msg {
                Some(m) => {
                    let resp = ApiResponse::success(m);
                    Response::from_json(&resp).map(|r| r.with_status(201))
                }
                None => AppError::Internal("Failed to retrieve created message".to_string()).into_response(),
            }
        }
        Err(e) => AppError::Database(format!("Failed to send message: {}", e)).into_response(),
    }
}

pub async fn get_messages(d1: &D1Database, chat_id: &str) -> Result<Response> {
    // Verify chat exists
    if get_chat_by_id(d1, chat_id).await?.is_none() {
        return AppError::NotFound(format!("Chat '{}' not found", chat_id)).into_response();
    }

    let result = d1
        .prepare("SELECT * FROM messages WHERE chat_id = ? ORDER BY created_at ASC")
        .bind(&[chat_id.into()])?
        .all()
        .await;

    match result {
        Ok(messages) => {
            let results: Vec<Message> = messages.results()?;
            let total = results.len();
            let resp = PaginatedResponse {
                success: true,
                total,
                data: results,
            };
            Response::from_json(&resp)
        }
        Err(e) => AppError::Database(format!("Failed to get messages: {}", e)).into_response(),
    }
}
