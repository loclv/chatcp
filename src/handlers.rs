use worker::*;
use serde::de::DeserializeOwned;

use crate::db;
use crate::models::*;

// ─── Helper ──────────────────────────────────────────────────────────────────

/// Parse a JSON request body into the given type, returning a 400 error on failure.
fn parse_body<T: DeserializeOwned>(req: &Request) -> std::result::Result<T, Response> {
    match req.json::<T>() {
        Ok(body) => Ok(body),
        Err(e) => {
            let resp = ApiResponse::<()>::error(&format!("Invalid request body: {}", e));
            Err(Response::from_json(&resp)
                .unwrap()
                .with_status(400))
        }
    }
}

/// Add CORS headers to a response for development use.
fn with_cors(mut resp: Response) -> Response {
    let headers = resp.headers_mut();
    headers.set("Access-Control-Allow-Origin", "*").ok();
    headers.set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS").ok();
    headers.set("Access-Control-Allow-Headers", "Content-Type, Authorization").ok();
    headers.set("Access-Control-Max-Age", "86400").ok();
    resp
}

/// Handle OPTIONS preflight requests.
pub fn handle_options() -> Result<Response> {
    let resp = Response::empty()?.with_status(204);
    Ok(with_cors(resp))
}

// ─── Agents ──────────────────────────────────────────────────────────────────

pub async fn create_agent(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let body = match parse_body::<CreateAgentRequest>(&req) {
        Ok(b) => b,
        Err(resp) => return Ok(with_cors(resp)),
    };
    let resp = db::create_agent(&d1, &body).await?;
    Ok(with_cors(resp))
}

pub async fn list_agents(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let resp = db::list_agents(&d1).await?;
    Ok(with_cors(resp))
}

pub async fn get_agent(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let id = ctx.param("id").unwrap_or("");
    let resp = db::get_agent(&d1, id).await?;
    Ok(with_cors(resp))
}

pub async fn update_agent(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let id = ctx.param("id").unwrap_or("");
    let body = match parse_body::<UpdateAgentRequest>(&req) {
        Ok(b) => b,
        Err(resp) => return Ok(with_cors(resp)),
    };
    let resp = db::update_agent(&d1, id, &body).await?;
    Ok(with_cors(resp))
}

pub async fn delete_agent(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let id = ctx.param("id").unwrap_or("");
    let resp = db::delete_agent(&d1, id).await?;
    Ok(with_cors(resp))
}

// ─── Owners ──────────────────────────────────────────────────────────────────

pub async fn create_owner(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let body = match parse_body::<CreateOwnerRequest>(&req) {
        Ok(b) => b,
        Err(resp) => return Ok(with_cors(resp)),
    };
    let resp = db::create_owner(&d1, &body).await?;
    Ok(with_cors(resp))
}

pub async fn list_owners(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let resp = db::list_owners(&d1).await?;
    Ok(with_cors(resp))
}

pub async fn get_owner(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let id = ctx.param("id").unwrap_or("");
    let resp = db::get_owner(&d1, id).await?;
    Ok(with_cors(resp))
}

pub async fn update_owner(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let id = ctx.param("id").unwrap_or("");
    let body = match parse_body::<UpdateOwnerRequest>(&req) {
        Ok(b) => b,
        Err(resp) => return Ok(with_cors(resp)),
    };
    let resp = db::update_owner(&d1, id, &body).await?;
    Ok(with_cors(resp))
}

pub async fn delete_owner(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let id = ctx.param("id").unwrap_or("");
    let resp = db::delete_owner(&d1, id).await?;
    Ok(with_cors(resp))
}

// ─── Chats ───────────────────────────────────────────────────────────────────

pub async fn create_chat(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let body = match parse_body::<CreateChatRequest>(&req) {
        Ok(b) => b,
        Err(resp) => return Ok(with_cors(resp)),
    };
    let resp = db::create_chat(&d1, &body).await?;
    Ok(with_cors(resp))
}

pub async fn list_chats(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let resp = db::list_chats(&d1).await?;
    Ok(with_cors(resp))
}

pub async fn get_chat_with_messages(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let id = ctx.param("id").unwrap_or("");
    let resp = db::get_chat_with_messages(&d1, id).await?;
    Ok(with_cors(resp))
}

pub async fn update_chat(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let id = ctx.param("id").unwrap_or("");
    let body = match parse_body::<UpdateChatRequest>(&req) {
        Ok(b) => b,
        Err(resp) => return Ok(with_cors(resp)),
    };
    let resp = db::update_chat(&d1, id, &body).await?;
    Ok(with_cors(resp))
}

pub async fn delete_chat(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let id = ctx.param("id").unwrap_or("");
    let resp = db::delete_chat(&d1, id).await?;
    Ok(with_cors(resp))
}

// ─── Messages ────────────────────────────────────────────────────────────────

pub async fn send_message(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let chat_id = ctx.param("id").unwrap_or("");
    let body = match parse_body::<SendMessageRequest>(&req) {
        Ok(b) => b,
        Err(resp) => return Ok(with_cors(resp)),
    };
    let resp = db::send_message(&d1, chat_id, &body).await?;
    Ok(with_cors(resp))
}

pub async fn get_messages(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("DB")?;
    let chat_id = ctx.param("id").unwrap_or("");
    let resp = db::get_messages(&d1, chat_id).await?;
    Ok(with_cors(resp))
}
