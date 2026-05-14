//! HTTP request handlers.
//!
//! Bridges the HTTP layer and the database layer. Each handler:
//! 1. Extracts path parameters from the route context
//! 2. Parses and validates JSON request bodies
//! 3. Calls the appropriate database function
//! 4. Applies CORS headers to the response

use worker::*;
use serde::de::DeserializeOwned;

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
        }
    };

    match body.validate() {
        Ok(()) => Ok(body),
        Err(e) => Err(e.into_response().unwrap()),
    }
}

/// Add CORS headers to a response for development use.
fn with_cors(mut resp: Response) -> Response {
    let headers = resp.headers_mut();
    headers.set("Access-Control-Allow-Origin", "*").ok();
    headers
        .set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        .ok();
    headers
        .set("Access-Control-Allow-Headers", "Content-Type, Authorization")
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
            headers.get("Access-Control-Allow-Origin").unwrap().as_deref(),
            Some("*")
        );
        assert_eq!(
            headers.get("Access-Control-Allow-Methods").unwrap().as_deref(),
            Some("GET, POST, PUT, DELETE, OPTIONS")
        );
        assert_eq!(
            headers.get("Access-Control-Allow-Headers").unwrap().as_deref(),
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
            headers.get("Access-Control-Allow-Origin").unwrap().as_deref(),
            Some("*")
        );
    }
}


// ─── Agents ──────────────────────────────────────────────────────────────────

pub async fn create_agent(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let body = match parse_and_validate::<CreateAgentRequest>(&mut req).await {
        Ok(b) => b,
        Err(resp) => return Ok(with_cors(resp)),
    };
    let resp = db::create_agent(&d1, &body).await?;
    Ok(with_cors(resp))
}

pub async fn list_agents(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let resp = db::list_agents(&d1).await?;
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
    let resp = db::delete_agent(&d1, id).await?;
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

pub async fn list_owners(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let resp = db::list_owners(&d1).await?;
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

// ─── Chats ───────────────────────────────────────────────────────────────────

pub async fn create_chat(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let body = match parse_and_validate::<CreateChatRequest>(&mut req).await {
        Ok(b) => b,
        Err(resp) => return Ok(with_cors(resp)),
    };
    let resp = db::create_chat(&d1, &body).await?;
    Ok(with_cors(resp))
}

pub async fn list_chats(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let resp = db::list_chats(&d1).await?;
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
    let resp = db::delete_chat(&d1, id).await?;
    Ok(with_cors(resp))
}

// ─── Messages ────────────────────────────────────────────────────────────────

pub async fn send_message(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let chat_id = ctx.param("id").map_or("", |v| v.as_str());
    let body = match parse_and_validate::<SendMessageRequest>(&mut req).await {
        Ok(b) => b,
        Err(resp) => return Ok(with_cors(resp)),
    };
    let resp = db::send_message(&d1, chat_id, &body).await?;
    Ok(with_cors(resp))
}

pub async fn get_messages(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("DB")?;
    let chat_id = ctx.param("id").map_or("", |v| v.as_str());
    let resp = db::get_messages(&d1, chat_id).await?;
    Ok(with_cors(resp))
}
