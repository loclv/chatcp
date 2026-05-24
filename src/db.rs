use uuid::Uuid;
use wasm_bindgen::JsValue;
use worker::*;

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
            assert!(
                b.is_ascii_hexdigit(),
                "Non-hex char '{}' at pos {}",
                b as char,
                i
            );
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
            },
            None => {
                AppError::Internal("Failed to retrieve created agent".to_string()).into_response()
            },
        },
        Err(e) => AppError::Database(format!("Failed to create agent: {}", e)).into_response(),
    }
}

pub async fn list_agents(d1: &D1Database, params: &QueryParams) -> Result<Response> {
    let mut query = "SELECT * FROM agents".to_string();
    let mut count_query = "SELECT COUNT(*) as total FROM agents".to_string();
    let mut args: Vec<JsValue> = Vec::new();

    if let Some(owner_id) = &params.owner_id {
        query.push_str(" WHERE owner_id = ?");
        count_query.push_str(" WHERE owner_id = ?");
        args.push(owner_id.as_str().into());
    }

    if let Some(after) = &params.created_after {
        let prefix = if query.contains("WHERE") {
            " AND "
        } else {
            " WHERE "
        };
        query.push_str(prefix);
        query.push_str("created_at >= ?");
        count_query.push_str(prefix);
        count_query.push_str("created_at >= ?");
        args.push(after.as_str().into());
    }

    if let Some(before) = &params.created_before {
        let prefix = if query.contains("WHERE") {
            " AND "
        } else {
            " WHERE "
        };
        query.push_str(prefix);
        query.push_str("created_at <= ?");
        count_query.push_str(prefix);
        count_query.push_str("created_at <= ?");
        args.push(before.as_str().into());
    }

    let sort_by = params.sort_by.as_deref().unwrap_or("created_at");
    let sort_order = params.sort_order.as_deref().unwrap_or("DESC");
    // Whitelist sort fields
    let safe_sort_by = match sort_by {
        "name" => "name",
        "updated_at" => "updated_at",
        _ => "created_at",
    };
    let safe_sort_order = if sort_order.to_uppercase() == "ASC" {
        "ASC"
    } else {
        "DESC"
    };

    query.push_str(&format!(" ORDER BY {} {}", safe_sort_by, safe_sort_order));
    query.push_str(" LIMIT ? OFFSET ?");

    let limit = params.limit();
    let offset = params.offset();

    let mut bind_args = args.clone();
    bind_args.push(limit.into());
    bind_args.push(offset.into());

    let results = d1.prepare(&query).bind(&bind_args)?.all().await;
    let total_count = get_total_count(d1, &count_query, &args).await?;

    match results {
        Ok(agents) => {
            let data: Vec<Agent> = agents.results()?;
            let has_more = offset as usize + data.len() < total_count;
            let resp = PaginatedResponse {
                success: true,
                data,
                pagination: PaginationMetadata {
                    limit,
                    offset,
                    total: total_count,
                    has_more,
                },
            };
            Response::from_json(&resp)
        },
        Err(e) => AppError::Database(format!("Failed to list agents: {}", e)).into_response(),
    }
}

async fn get_total_count(d1: &D1Database, query: &str, args: &[JsValue]) -> Result<usize> {
    let result = d1
        .prepare(query)
        .bind(args)?
        .first::<serde_json::Value>(None)
        .await?;
    match result {
        Some(row) => {
            let count = row.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
            Ok(count as usize)
        },
        None => Ok(0),
    }
}

pub async fn get_agent(d1: &D1Database, id: &str) -> Result<Response> {
    match get_agent_by_id(d1, id).await {
        Ok(Some(agent)) => {
            let resp = ApiResponse::success(agent);
            Response::from_json(&resp)
        },
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

/// Raw version of get_agent_by_id exposed to other modules.
pub async fn get_agent_raw(d1: &D1Database, id: &str) -> Result<Option<Agent>> {
    get_agent_by_id(d1, id).await
}

pub async fn get_agent_messages(
    d1: &D1Database,
    id: &str,
    params: &QueryParams,
) -> Result<Response> {
    let query = "SELECT * FROM messages WHERE sender_id = ? AND sender_type = 'agent' ORDER BY created_at DESC LIMIT ? OFFSET ?";
    let limit = params.limit();
    let offset = params.offset();

    let results = d1
        .prepare(query)
        .bind(&[id.into(), limit.into(), offset.into()])?
        .all()
        .await;

    let total_count = get_total_count(
        d1,
        "SELECT COUNT(*) as total FROM messages WHERE sender_id = ? AND sender_type = 'agent'",
        &[id.into()],
    )
    .await?;

    match results {
        Ok(messages) => {
            let data: Vec<Message> = messages.results()?;
            let has_more = offset as usize + data.len() < total_count;
            let resp = PaginatedResponse {
                success: true,
                data,
                pagination: PaginationMetadata {
                    limit,
                    offset,
                    total: total_count,
                    has_more,
                },
            };
            Response::from_json(&resp)
        },
        Err(e) => {
            AppError::Database(format!("Failed to get agent messages: {}", e)).into_response()
        },
    }
}

pub async fn update_agent(d1: &D1Database, id: &str, req: &UpdateAgentRequest) -> Result<Response> {
    let existing = match get_agent_by_id(d1, id).await? {
        Some(a) => a,
        None => return AppError::NotFound(format!("Agent '{}' not found", id)).into_response(),
    };

    let name = req.name.as_deref().unwrap_or(&existing.name);
    let description = req.description.as_deref().unwrap_or(&existing.description);
    let owner_id = req.owner_id.clone().or(existing.owner_id);

    let result = d1
        .prepare(
            "UPDATE agents SET name = ?, description = ?, owner_id = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(&[
            name.into(),
            description.into(),
            optional_js_value(&owner_id),
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
        },
        Err(e) => AppError::Database(format!("Failed to delete agent: {}", e)).into_response(),
    }
}

// ─── Owners ──────────────────────────────────────────────────────────────────

pub async fn create_owner(d1: &D1Database, req: &CreateOwnerRequest) -> Result<Response> {
    let id = generate_id();

    let (hash, salt, api_key) = if let Some(password) = &req.password {
        let salt_str = generate_id().replace("-", "");
        let hash_str = crate::auth::hash_password(password, &salt_str);
        let api_key_str = format!("o_{}", generate_id().replace("-", ""));
        (
            JsValue::from_str(&hash_str),
            JsValue::from_str(&salt_str),
            JsValue::from_str(&api_key_str),
        )
    } else {
        (JsValue::null(), JsValue::null(), JsValue::null())
    };

    let result = d1
        .prepare(
            "INSERT INTO owners (id, name, email, password_hash, salt, api_key, created_at) VALUES (?, ?, ?, ?, ?, ?, datetime('now'))",
        )
        .bind(&[
            id.as_str().into(),
            req.name.as_str().into(),
            req.email.as_str().into(),
            hash,
            salt,
            api_key,
        ])?
        .run()
        .await;

    match result {
        Ok(_) => match get_owner_by_id(d1, &id).await? {
            Some(owner) => {
                let resp = ApiResponse::success(owner);
                Response::from_json(&resp).map(|r| r.with_status(201))
            },
            None => {
                AppError::Internal("Failed to retrieve created owner".to_string()).into_response()
            },
        },
        Err(e) => {
            let msg = format!("{}", e);
            if msg.contains("UNIQUE") {
                return AppError::BadRequest("An owner with this email already exists".to_string())
                    .into_response();
            }
            AppError::Database(format!("Failed to create owner: {}", e)).into_response()
        },
    }
}

pub async fn list_owners(d1: &D1Database, params: &QueryParams) -> Result<Response> {
    let query = "SELECT * FROM owners ORDER BY created_at DESC LIMIT ? OFFSET ?";
    let limit = params.limit();
    let offset = params.offset();

    let results = d1
        .prepare(query)
        .bind(&[limit.into(), offset.into()])?
        .all()
        .await;
    let total_count = get_total_count(d1, "SELECT COUNT(*) as total FROM owners", &[]).await?;

    match results {
        Ok(owners) => {
            let data: Vec<Owner> = owners.results()?;
            let has_more = offset as usize + data.len() < total_count;
            let resp = PaginatedResponse {
                success: true,
                data,
                pagination: PaginationMetadata {
                    limit,
                    offset,
                    total: total_count,
                    has_more,
                },
            };
            Response::from_json(&resp)
        },
        Err(e) => AppError::Database(format!("Failed to list owners: {}", e)).into_response(),
    }
}

pub async fn get_owner(d1: &D1Database, id: &str) -> Result<Response> {
    match get_owner_by_id(d1, id).await {
        Ok(Some(owner)) => {
            let resp = ApiResponse::success(owner);
            Response::from_json(&resp)
        },
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
        },
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
            },
            None => {
                AppError::Internal("Failed to retrieve created chat".to_string()).into_response()
            },
        },
        Err(e) => AppError::Database(format!("Failed to create chat: {}", e)).into_response(),
    }
}

pub async fn list_chats(d1: &D1Database, params: &QueryParams) -> Result<Response> {
    let mut query = "SELECT * FROM chats".to_string();
    let mut count_query = "SELECT COUNT(*) as total FROM chats".to_string();
    let mut args: Vec<JsValue> = Vec::new();

    let mut where_clauses = Vec::new();
    if let Some(agent_id) = &params.agent_id {
        where_clauses.push("agent_id = ?");
        args.push(agent_id.as_str().into());
    }
    if let Some(owner_id) = &params.owner_id {
        where_clauses.push("owner_id = ?");
        args.push(owner_id.as_str().into());
    }

    if !where_clauses.is_empty() {
        let clause = format!(" WHERE {}", where_clauses.join(" AND "));
        query.push_str(&clause);
        count_query.push_str(&clause);
    }

    query.push_str(" ORDER BY updated_at DESC LIMIT ? OFFSET ?");

    let limit = params.limit();
    let offset = params.offset();

    let mut bind_args = args.clone();
    bind_args.push(limit.into());
    bind_args.push(offset.into());

    let results = d1.prepare(&query).bind(&bind_args)?.all().await;
    let total_count = get_total_count(d1, &count_query, &args).await?;

    match results {
        Ok(chats) => {
            let data: Vec<Chat> = chats.results()?;
            let has_more = offset as usize + data.len() < total_count;
            let resp = PaginatedResponse {
                success: true,
                data,
                pagination: PaginationMetadata {
                    limit,
                    offset,
                    total: total_count,
                    has_more,
                },
            };
            Response::from_json(&resp)
        },
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
        },
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
        },
        Err(e) => AppError::Database(format!("Failed to delete chat: {}", e)).into_response(),
    }
}

// ─── Messages ────────────────────────────────────────────────────────────────

pub async fn send_message(
    d1: &D1Database,
    chat_id: &str,
    req: &SendMessageRequest,
) -> Result<Response> {
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
                },
                None => AppError::Internal("Failed to retrieve created message".to_string())
                    .into_response(),
            }
        },
        Err(e) => AppError::Database(format!("Failed to send message: {}", e)).into_response(),
    }
}

pub async fn get_messages(
    d1: &D1Database,
    chat_id: &str,
    params: &QueryParams,
) -> Result<Response> {
    // Verify chat exists
    if get_chat_by_id(d1, chat_id).await?.is_none() {
        return AppError::NotFound(format!("Chat '{}' not found", chat_id)).into_response();
    }

    let mut query = "SELECT * FROM messages WHERE chat_id = ?".to_string();
    let mut count_query = "SELECT COUNT(*) as total FROM messages WHERE chat_id = ?".to_string();
    let mut args: Vec<JsValue> = vec![chat_id.into()];

    if let Some(q) = &params.q {
        let filter = format!("%{}%", q);
        query.push_str(" AND content LIKE ?");
        count_query.push_str(" AND content LIKE ?");
        args.push(filter.into());
    }

    query.push_str(" ORDER BY created_at ASC LIMIT ? OFFSET ?");

    let limit = params.limit();
    let offset = params.offset();

    let mut bind_args = args.clone();
    bind_args.push(limit.into());
    bind_args.push(offset.into());

    let results = d1.prepare(&query).bind(&bind_args)?.all().await;
    let total_count = get_total_count(d1, &count_query, &args).await?;

    match results {
        Ok(messages) => {
            let data: Vec<Message> = messages.results()?;
            let has_more = offset as usize + data.len() < total_count;
            let resp = PaginatedResponse {
                success: true,
                data,
                pagination: PaginationMetadata {
                    limit,
                    offset,
                    total: total_count,
                    has_more,
                },
            };
            Response::from_json(&resp)
        },
        Err(e) => AppError::Database(format!("Failed to get messages: {}", e)).into_response(),
    }
}

// ─── Authentication Operations ──────────────────────────────────────────────

pub async fn get_owner_raw(d1: &D1Database, id: &str) -> Result<Option<Owner>> {
    get_owner_by_id(d1, id).await
}

pub async fn get_owner_by_email(d1: &D1Database, email: &str) -> Result<Option<Owner>> {
    let result = d1
        .prepare("SELECT * FROM owners WHERE email = ?")
        .bind(&[email.into()])?
        .first::<Owner>(None)
        .await?;
    Ok(result)
}

pub async fn get_owner_by_api_key(d1: &D1Database, api_key: &str) -> Result<Option<Owner>> {
    let result = d1
        .prepare("SELECT * FROM owners WHERE api_key = ?")
        .bind(&[api_key.into()])?
        .first::<Owner>(None)
        .await?;
    Ok(result)
}

pub async fn get_agent_by_api_key(d1: &D1Database, api_key: &str) -> Result<Option<Agent>> {
    let result = d1
        .prepare("SELECT * FROM agents WHERE api_key = ?")
        .bind(&[api_key.into()])?
        .first::<Agent>(None)
        .await?;
    Ok(result)
}

pub async fn rotate_owner_api_key(d1: &D1Database, id: &str) -> Result<String> {
    let new_key = format!("o_{}", generate_id().replace("-", ""));
    d1.prepare("UPDATE owners SET api_key = ? WHERE id = ?")
        .bind(&[new_key.clone().into(), id.into()])?
        .run()
        .await?;
    Ok(new_key)
}

pub async fn rotate_agent_api_key(d1: &D1Database, id: &str) -> Result<String> {
    let new_key = format!("a_{}", generate_id().replace("-", ""));
    d1.prepare("UPDATE agents SET api_key = ? WHERE id = ?")
        .bind(&[new_key.clone().into(), id.into()])?
        .run()
        .await?;
    Ok(new_key)
}

pub async fn get_chat_raw(d1: &D1Database, id: &str) -> Result<Option<Chat>> {
    get_chat_by_id(d1, id).await
}
