//! HTTP request handlers.
//!
//! Bridges the HTTP layer and the database layer. Each handler:
//! 1. Extracts path parameters from the route context
//! 2. Parses and validates JSON request bodies
//! 3. Calls the appropriate database function
//! 4. Applies CORS headers to the response

use serde::de::DeserializeOwned;
use worker::*;

use crate::db;
use crate::prelude::*;
use crate::validation::Validator;

// ─── Helper ──────────────────────────────────────────────────────────────────

/// Parse a JSON request body into the given type, validate it, and return
/// the validated struct or a 400 error response.
async fn parse_and_validate<T>(req: &mut Request) -> std::result::Result<T, Response>
where
    T: DeserializeOwned + Validator,
{
    let body: T = match req.json::<T>().await {
        Ok(body) => body,
        Err(e) => {
            let err = AppError::BadRequest(format!("Invalid request body: {}", e));
            return Err(err.into_response().unwrap());
        },
    };

    match body.validate() {
        Ok(()) => Ok(body),
        Err(e) => Err(e.into_response().unwrap()),
    }
}

/// Parse a query string into the given QueryParams struct, validate it, and
/// return the validated struct or an error response.
fn parse_query(req: &Request) -> std::result::Result<QueryParams, Response> {
    let params = match req.query::<QueryParams>() {
        Ok(p) => p,
        Err(e) => {
            let err = AppError::BadRequest(format!("Invalid query parameters: {}", e));
            return Err(with_cors(err.into_response().unwrap()));
        },
    };
    match params.validate() {
        Ok(()) => Ok(params),
        Err(e) => Err(with_cors(e.into_response().unwrap())),
    }
}

/// Add CORS headers to a response for development use.
fn with_cors(mut resp: Response) -> Response {
    let headers = resp.headers_mut();
    headers.set("Access-Control-Allow-Origin", "*").ok();
    headers
        .set(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE, OPTIONS",
        )
        .ok();
    headers
        .set(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        )
        .ok();
    headers.set("Access-Control-Max-Age", "86400").ok();
    resp
}

/// Handle OPTIONS preflight requests.
pub fn handle_options() -> Result<Response> {
    let resp = Response::empty()?.with_status(204);
    Ok(with_cors(resp))
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;

    #[test]
    fn test_handle_options_status() {
        let resp = handle_options().unwrap();
        assert_eq!(resp.status_code(), 204);
    }

    #[test]
    fn test_handle_options_has_cors() {
        let resp = handle_options().unwrap();
        let headers = resp.headers();
        assert_eq!(
            headers
                .get("Access-Control-Allow-Origin")
                .unwrap()
                .as_deref(),
            Some("*")
        );
        assert_eq!(
            headers
                .get("Access-Control-Allow-Methods")
                .unwrap()
                .as_deref(),
            Some("GET, POST, PUT, DELETE, OPTIONS")
        );
        assert_eq!(
            headers
                .get("Access-Control-Allow-Headers")
                .unwrap()
                .as_deref(),
            Some("Content-Type, Authorization")
        );
        assert_eq!(
            headers.get("Access-Control-Max-Age").unwrap().as_deref(),
            Some("86400")
        );
    }

    #[test]
    fn test_with_cors_adds_headers() {
        let resp = Response::empty().unwrap();
        let resp = with_cors(resp);
        let headers = resp.headers();
        assert_eq!(
            headers
                .get("Access-Control-Allow-Origin")
                .unwrap()
                .as_deref(),
            Some("*")
        );
    }
}

// ─── Agents ──────────────────────────────────────────────────────────────────

pub async fn create_agent(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let secret = match ctx.var("JWT_SECRET") {
        Ok(s) => s.to_string(),
        Err(_) => "fallback_secret_key_1234567890".to_string(),
    };

    let auth = match crate::auth::authenticate_request(&req, &d1, &secret).await {
        Ok(a) => a,
        Err(e) => return Ok(with_cors(e.into_response()?)),
    };

    let owner = match auth.get_owner() {
        Ok(o) => o,
        Err(e) => return Ok(with_cors(e.into_response()?)),
    };

    let mut body = match parse_and_validate::<CreateAgentRequest>(&mut req).await {
        Ok(b) => b,
        Err(resp) => return Ok(with_cors(resp)),
    };

    body.owner_id = Some(owner.id);

    let resp = db::create_agent(&d1, &body).await?;
    Ok(with_cors(resp))
}

pub async fn list_agents(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let params = match parse_query(&req) {
        Ok(p) => p,
        Err(resp) => return Ok(resp),
    };
    let resp = db::list_agents(&d1, &params).await?;
    Ok(with_cors(resp))
}

pub async fn get_agent(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let id = ctx.param("id").map_or("", |v| v.as_str());
    let resp = db::get_agent(&d1, id).await?;
    Ok(with_cors(resp))
}

pub async fn update_agent(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let id = ctx.param("id").map_or("", |v| v.as_str());

    let secret = match ctx.var("JWT_SECRET") {
        Ok(s) => s.to_string(),
        Err(_) => "fallback_secret_key_1234567890".to_string(),
    };

    let auth = match crate::auth::authenticate_request(&req, &d1, &secret).await {
        Ok(a) => a,
        Err(e) => return Ok(with_cors(e.into_response()?)),
    };

    let owner = match auth.get_owner() {
        Ok(o) => o,
        Err(e) => return Ok(with_cors(e.into_response()?)),
    };

    let agent = match db::get_agent_raw(&d1, id).await? {
        Some(a) => a,
        None => {
            return Ok(with_cors(
                AppError::NotFound(format!("Agent '{}' not found", id)).into_response()?,
            ))
        },
    };

    if agent.owner_id != Some(owner.id) {
        return Ok(with_cors(
            AppError::BadRequest("You can only modify agents you own".to_string())
                .into_response()?,
        ));
    }

    let body = match parse_and_validate::<UpdateAgentRequest>(&mut req).await {
        Ok(b) => b,
        Err(resp) => return Ok(with_cors(resp)),
    };
    let resp = db::update_agent(&d1, id, &body).await?;
    Ok(with_cors(resp))
}

pub async fn delete_agent(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let id = ctx.param("id").map_or("", |v| v.as_str());

    let secret = match ctx.var("JWT_SECRET") {
        Ok(s) => s.to_string(),
        Err(_) => "fallback_secret_key_1234567890".to_string(),
    };

    let auth = match crate::auth::authenticate_request(&_req, &d1, &secret).await {
        Ok(a) => a,
        Err(e) => return Ok(with_cors(e.into_response()?)),
    };

    let owner = match auth.get_owner() {
        Ok(o) => o,
        Err(e) => return Ok(with_cors(e.into_response()?)),
    };

    let agent = match db::get_agent_raw(&d1, id).await? {
        Some(a) => a,
        None => {
            return Ok(with_cors(
                AppError::NotFound(format!("Agent '{}' not found", id)).into_response()?,
            ))
        },
    };

    if agent.owner_id != Some(owner.id) {
        return Ok(with_cors(
            AppError::BadRequest("You can only delete agents you own".to_string())
                .into_response()?,
        ));
    }

    let resp = db::delete_agent(&d1, id).await?;
    Ok(with_cors(resp))
}

pub async fn get_agent_chats(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let id = ctx.param("id").map_or("", |v| v.as_str());
    let mut params = match parse_query(&req) {
        Ok(p) => p,
        Err(resp) => return Ok(resp),
    };
    params.agent_id = Some(id.to_string());
    let resp = db::list_chats(&d1, &params).await?;
    Ok(with_cors(resp))
}

pub async fn get_agent_owner(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let id = ctx.param("id").map_or("", |v| v.as_str());

    let agent = match db::get_agent_raw(&d1, id).await? {
        Some(a) => a,
        None => {
            return Ok(with_cors(
                AppError::NotFound(format!("Agent '{}' not found", id)).into_response()?,
            ))
        },
    };

    let owner_id = match agent.owner_id {
        Some(oid) => oid,
        None => {
            return Ok(with_cors(
                AppError::NotFound(format!("Agent '{}' has no owner", id)).into_response()?,
            ))
        },
    };

    let resp = db::get_owner(&d1, &owner_id).await?;
    Ok(with_cors(resp))
}

pub async fn get_agent_messages(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let id = ctx.param("id").map_or("", |v| v.as_str());
    let params = match parse_query(&req) {
        Ok(p) => p,
        Err(resp) => return Ok(resp),
    };
    let resp = db::get_agent_messages(&d1, id, &params).await?;
    Ok(with_cors(resp))
}

// ─── Owners ──────────────────────────────────────────────────────────────────

pub async fn create_owner(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let body = match parse_and_validate::<CreateOwnerRequest>(&mut req).await {
        Ok(b) => b,
        Err(resp) => return Ok(with_cors(resp)),
    };
    let resp = db::create_owner(&d1, &body).await?;
    Ok(with_cors(resp))
}

pub async fn list_owners(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let params = match parse_query(&req) {
        Ok(p) => p,
        Err(resp) => return Ok(resp),
    };
    let resp = db::list_owners(&d1, &params).await?;
    Ok(with_cors(resp))
}

pub async fn get_owner(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let id = ctx.param("id").map_or("", |v| v.as_str());
    let resp = db::get_owner(&d1, id).await?;
    Ok(with_cors(resp))
}

pub async fn update_owner(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let id = ctx.param("id").map_or("", |v| v.as_str());
    let body = match parse_and_validate::<UpdateOwnerRequest>(&mut req).await {
        Ok(b) => b,
        Err(resp) => return Ok(with_cors(resp)),
    };
    let resp = db::update_owner(&d1, id, &body).await?;
    Ok(with_cors(resp))
}

pub async fn delete_owner(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let id = ctx.param("id").map_or("", |v| v.as_str());
    let resp = db::delete_owner(&d1, id).await?;
    Ok(with_cors(resp))
}

pub async fn get_owner_agents(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let id = ctx.param("id").map_or("", |v| v.as_str());
    let mut params = match parse_query(&req) {
        Ok(p) => p,
        Err(resp) => return Ok(resp),
    };
    params.owner_id = Some(id.to_string());
    let resp = db::list_agents(&d1, &params).await?;
    Ok(with_cors(resp))
}

pub async fn get_owner_chats(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let id = ctx.param("id").map_or("", |v| v.as_str());
    let mut params = match parse_query(&req) {
        Ok(p) => p,
        Err(resp) => return Ok(resp),
    };
    params.owner_id = Some(id.to_string());
    let resp = db::list_chats(&d1, &params).await?;
    Ok(with_cors(resp))
}

// ─── Chats ───────────────────────────────────────────────────────────────────

pub async fn create_chat(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;

    let secret = match ctx.var("JWT_SECRET") {
        Ok(s) => s.to_string(),
        Err(_) => "fallback_secret_key_1234567890".to_string(),
    };

    let auth = match crate::auth::authenticate_request(&req, &d1, &secret).await {
        Ok(a) => a,
        Err(e) => return Ok(with_cors(e.into_response()?)),
    };

    let owner = match auth.get_owner() {
        Ok(o) => o,
        Err(e) => return Ok(with_cors(e.into_response()?)),
    };

    let mut body = match parse_and_validate::<CreateChatRequest>(&mut req).await {
        Ok(b) => b,
        Err(resp) => return Ok(with_cors(resp)),
    };

    let agent = match db::get_agent_raw(&d1, &body.agent_id).await? {
        Some(a) => a,
        None => {
            return Ok(with_cors(
                AppError::NotFound(format!("Agent '{}' not found", body.agent_id))
                    .into_response()?,
            ))
        },
    };

    if agent.owner_id != Some(owner.id.clone()) {
        return Ok(with_cors(
            AppError::BadRequest("You can only create chats with agents you own".to_string())
                .into_response()?,
        ));
    }

    body.owner_id = owner.id;

    let resp = db::create_chat(&d1, &body).await?;
    Ok(with_cors(resp))
}

pub async fn list_chats(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let params = match parse_query(&req) {
        Ok(p) => p,
        Err(resp) => return Ok(resp),
    };
    let resp = db::list_chats(&d1, &params).await?;
    Ok(with_cors(resp))
}

pub async fn get_chat_with_messages(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let id = ctx.param("id").map_or("", |v| v.as_str());
    let resp = db::get_chat_with_messages(&d1, id).await?;
    Ok(with_cors(resp))
}

pub async fn update_chat(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let id = ctx.param("id").map_or("", |v| v.as_str());

    let secret = match ctx.var("JWT_SECRET") {
        Ok(s) => s.to_string(),
        Err(_) => "fallback_secret_key_1234567890".to_string(),
    };

    let auth = match crate::auth::authenticate_request(&req, &d1, &secret).await {
        Ok(a) => a,
        Err(e) => return Ok(with_cors(e.into_response()?)),
    };

    let owner = match auth.get_owner() {
        Ok(o) => o,
        Err(e) => return Ok(with_cors(e.into_response()?)),
    };

    let chat = match db::get_chat_raw(&d1, id).await? {
        Some(c) => c,
        None => {
            return Ok(with_cors(
                AppError::NotFound(format!("Chat '{}' not found", id)).into_response()?,
            ))
        },
    };

    if chat.owner_id != owner.id {
        return Ok(with_cors(
            AppError::BadRequest("You can only modify chats you own".to_string())
                .into_response()?,
        ));
    }

    let body = match parse_and_validate::<UpdateChatRequest>(&mut req).await {
        Ok(b) => b,
        Err(resp) => return Ok(with_cors(resp)),
    };
    let resp = db::update_chat(&d1, id, &body).await?;
    Ok(with_cors(resp))
}

pub async fn delete_chat(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let id = ctx.param("id").map_or("", |v| v.as_str());

    let secret = match ctx.var("JWT_SECRET") {
        Ok(s) => s.to_string(),
        Err(_) => "fallback_secret_key_1234567890".to_string(),
    };

    let auth = match crate::auth::authenticate_request(&_req, &d1, &secret).await {
        Ok(a) => a,
        Err(e) => return Ok(with_cors(e.into_response()?)),
    };

    let owner = match auth.get_owner() {
        Ok(o) => o,
        Err(e) => return Ok(with_cors(e.into_response()?)),
    };

    let chat = match db::get_chat_raw(&d1, id).await? {
        Some(c) => c,
        None => {
            return Ok(with_cors(
                AppError::NotFound(format!("Chat '{}' not found", id)).into_response()?,
            ))
        },
    };

    if chat.owner_id != owner.id {
        return Ok(with_cors(
            AppError::BadRequest("You can only delete chats you own".to_string())
                .into_response()?,
        ));
    }

    let resp = db::delete_chat(&d1, id).await?;
    Ok(with_cors(resp))
}

// ─── Messages ────────────────────────────────────────────────────────────────

pub async fn send_message(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let chat_id = ctx.param("id").map_or("", |v| v.as_str());

    let secret = match ctx.var("JWT_SECRET") {
        Ok(s) => s.to_string(),
        Err(_) => "fallback_secret_key_1234567890".to_string(),
    };

    let auth = match crate::auth::authenticate_request(&req, &d1, &secret).await {
        Ok(a) => a,
        Err(e) => return Ok(with_cors(e.into_response()?)),
    };

    let chat = match db::get_chat_raw(&d1, chat_id).await? {
        Some(c) => c,
        None => {
            return Ok(with_cors(
                AppError::NotFound(format!("Chat '{}' not found", chat_id)).into_response()?,
            ))
        },
    };

    let mut body = match parse_and_validate::<SendMessageRequest>(&mut req).await {
        Ok(b) => b,
        Err(resp) => return Ok(with_cors(resp)),
    };

    match auth {
        crate::auth::AuthContext::Owner(owner) => {
            if chat.owner_id != owner.id {
                return Ok(with_cors(
                    AppError::BadRequest("You do not have access to this chat".to_string())
                        .into_response()?,
                ));
            }
            body.sender_type = "owner".to_string();
            body.sender_id = owner.id;
        },
        crate::auth::AuthContext::Agent(agent) => {
            if chat.agent_id != agent.id {
                return Ok(with_cors(
                    AppError::BadRequest(
                        "You are not the assigned agent for this chat".to_string(),
                    )
                    .into_response()?,
                ));
            }
            body.sender_type = "agent".to_string();
            body.sender_id = agent.id;
        },
    }

    let resp = db::send_message(&d1, chat_id, &body).await?;
    Ok(with_cors(resp))
}

pub async fn get_messages(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let chat_id = ctx.param("id").map_or("", |v| v.as_str());
    let params = match parse_query(&req) {
        Ok(p) => p,
        Err(resp) => return Ok(resp),
    };
    let resp = db::get_messages(&d1, chat_id, &params).await?;
    Ok(with_cors(resp))
}

// ─── Authentication Handlers ─────────────────────────────────────────────────

pub async fn login(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let body = match parse_and_validate::<LoginRequest>(&mut req).await {
        Ok(b) => b,
        Err(resp) => return Ok(with_cors(resp)),
    };

    let owner = match db::get_owner_by_email(&d1, &body.email).await? {
        Some(o) => o,
        None => {
            return Ok(with_cors(
                AppError::Validation("Invalid email or password".to_string()).into_response()?,
            ))
        },
    };

    let stored_hash = match &owner.password_hash {
        Some(h) => h,
        None => {
            return Ok(with_cors(
                AppError::Validation("Account has no password configured".to_string())
                    .into_response()?,
            ))
        },
    };

    let stored_salt = match &owner.salt {
        Some(s) => s,
        None => {
            return Ok(with_cors(
                AppError::Validation("Account has no salt configured".to_string())
                    .into_response()?,
            ))
        },
    };

    let input_hash = crate::auth::hash_password(&body.password, stored_salt);
    if input_hash != *stored_hash {
        return Ok(with_cors(
            AppError::Validation("Invalid email or password".to_string()).into_response()?,
        ));
    }

    let secret = match ctx.var("JWT_SECRET") {
        Ok(s) => s.to_string(),
        Err(_) => "fallback_secret_key_1234567890".to_string(),
    };

    let token = crate::auth::generate_jwt(&owner.id, &secret, 86400)?; // 24 hours

    let resp = ApiResponse::success(serde_json::json!({
        "token": token,
        "owner": owner
    }));
    Ok(with_cors(Response::from_json(&resp)?))
}

pub async fn rotate_owner_key(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let id = ctx.param("id").map_or("", |v| v.as_str());

    let secret = match ctx.var("JWT_SECRET") {
        Ok(s) => s.to_string(),
        Err(_) => "fallback_secret_key_1234567890".to_string(),
    };

    let auth = match crate::auth::authenticate_request(&req, &d1, &secret).await {
        Ok(a) => a,
        Err(e) => return Ok(with_cors(e.into_response()?)),
    };

    let caller = match auth.get_owner() {
        Ok(o) => o,
        Err(e) => return Ok(with_cors(e.into_response()?)),
    };

    if caller.id != id {
        return Ok(with_cors(
            AppError::BadRequest("You can only rotate your own API key".to_string())
                .into_response()?,
        ));
    }

    let new_key = db::rotate_owner_api_key(&d1, id).await?;
    let resp = ApiResponse::success(serde_json::json!({
        "api_key": new_key
    }));
    Ok(with_cors(Response::from_json(&resp)?))
}

pub async fn rotate_agent_key(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let id = ctx.param("id").map_or("", |v| v.as_str());

    let secret = match ctx.var("JWT_SECRET") {
        Ok(s) => s.to_string(),
        Err(_) => "fallback_secret_key_1234567890".to_string(),
    };

    let auth = match crate::auth::authenticate_request(&req, &d1, &secret).await {
        Ok(a) => a,
        Err(e) => return Ok(with_cors(e.into_response()?)),
    };

    let caller = match auth.get_owner() {
        Ok(o) => o,
        Err(e) => return Ok(with_cors(e.into_response()?)),
    };

    let agent = match db::get_agent_raw(&d1, id).await? {
        Some(a) => a,
        None => {
            return Ok(with_cors(
                AppError::NotFound(format!("Agent '{}' not found", id)).into_response()?,
            ))
        },
    };

    if agent.owner_id != Some(caller.id) {
        return Ok(with_cors(
            AppError::BadRequest("You can only rotate keys for agents you own".to_string())
                .into_response()?,
        ));
    }

    let new_key = db::rotate_agent_api_key(&d1, id).await?;
    let resp = ApiResponse::success(serde_json::json!({
        "api_key": new_key
    }));
    Ok(with_cors(Response::from_json(&resp)?))
}

pub async fn get_me(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let secret = match ctx.var("JWT_SECRET") {
        Ok(s) => s.to_string(),
        Err(_) => "fallback_secret_key_1234567890".to_string(),
    };

    let auth = match crate::auth::authenticate_request(&req, &d1, &secret).await {
        Ok(a) => a,
        Err(e) => return Ok(with_cors(e.into_response()?)),
    };

    let resp = match auth {
        crate::auth::AuthContext::Owner(owner) => ApiResponse::success(serde_json::json!({
            "type": "owner",
            "profile": owner
        })),
        crate::auth::AuthContext::Agent(agent) => ApiResponse::success(serde_json::json!({
            "type": "agent",
            "profile": agent
        })),
    };

    Ok(with_cors(Response::from_json(&resp)?))
}
