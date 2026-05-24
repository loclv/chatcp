//! Chat App Backend — Cloudflare Worker entry point.
//!
//! This module initializes the Worker runtime, builds the HTTP router,
//! and dispatches incoming requests to the appropriate handlers.

use worker::*;

mod db;
mod handlers;
mod models;
mod prelude;
mod router;
mod validation;
// #[cfg(test)]
// mod integration_tests;

/// The main entry point for the Cloudflare Worker.
/// Creates the router and dispatches the incoming request.
#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let start_time = Date::now().as_millis();

    // Extract the CF-Ray ID before moving the request into the router
    let request_id = req
        .headers()
        .get("cf-ray")
        .unwrap_or_else(|_| Some(uuid::Uuid::new_v4().to_string()))
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let router = router::build_router();
    let mut resp = router.run(req, env).await?;

    let end_time = Date::now().as_millis();
    let duration = end_time - start_time;

    let headers = resp.headers_mut();
    headers.set("X-Request-Id", &request_id).ok();
    headers
        .set("X-Response-Time", &format!("{}ms", duration))
        .ok();

    Ok(resp)
}
