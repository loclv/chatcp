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

/// The main entry point for the Cloudflare Worker.
/// Creates the router and dispatches the incoming request.
#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = router::build_router();
    router.run(req, env).await
}
