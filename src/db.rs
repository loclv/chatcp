use worker::*;
use uuid::Uuid;

use crate::prelude::*;

// ─── Helper ──────────────────────────────────────────────────────────────────

/// Generate a UUID v4 string for use as a primary key.
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// Convert an `Option<String>` to a `D1Value` (NULL if None).
fn optional_d1_value(opt: &Option<String>) -> D1Value {
    match opt {
        Some(val) => val.as_str().into(),
        None => D1Value::Null,
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
            optional_d1_value(&req.owner_id),
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
        .all::<Agent>()
        .await;

    match result {
        Ok(agents) => {
            let results = agents.results();
            let resp = PaginatedResponse {
                success: true,
                total: results.len(),
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
        .first::<Agent>()
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
            optional_d1_value(&req.owner_id),
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
            if meta.changes() > 0 {
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
        .all::<Owner>()
        .await;

    match result {
        Ok(owners) => {
            let results = owners.results();
            let resp = PaginatedResponse {
                success: true,
                total: results.len(),
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
        .first::<Owner>()
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
            if meta.changes() > 0 {
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
        .all::<Chat>()
        .await;

    match result {
        Ok(chats) => {
            let results = chats.results();
            let resp = PaginatedResponse {
                success: true,
                total: results.len(),
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
        .first::<Chat>()
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
        .all::<Message>()
        .await;

    match messages_result {
        Ok(messages) => {
            let chat_with_msgs = ChatWithMessages {
                chat,
                messages: messages.results(),
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
            if meta.changes() > 0 {
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
                .first::<Message>()
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
        .all::<Message>()
        .await;

    match result {
        Ok(messages) => {
            let results = messages.results();
            let resp = PaginatedResponse {
                success: true,
                total: results.len(),
                data: results,
            };
            Response::from_json(&resp)
        }
        Err(e) => AppError::Database(format!("Failed to get messages: {}", e)).into_response(),
    }
}
