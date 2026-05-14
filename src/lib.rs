use worker::*;

mod db;
mod handlers;
mod models;

/// The main entry point for the Cloudflare Worker.
/// Routes HTTP requests to the appropriate handlers.
#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Create router
    let router = Router::new();

    // ─── CORS preflight ─────────────────────────────────────────────────
    router
        .options_async("/*", |_req, _ctx| async move {
            handlers::handle_options()
        })

        // ─── Health ───────────────────────────────────────────────────────
        .get_async("/api/health", |_req, _ctx| async move {
            Response::from_json(&serde_json::json!({
                "status": "ok",
                "service": "chat-app-backend",
                "version": env!("CARGO_PKG_VERSION")
            }))
        })

        // ─── Agents CRUD ────────────────────────────────────────────────
        .post_async("/api/agents", handlers::create_agent)
        .get_async("/api/agents", handlers::list_agents)
        .get_async("/api/agents/:id", handlers::get_agent)
        .put_async("/api/agents/:id", handlers::update_agent)
        .delete_async("/api/agents/:id", handlers::delete_agent)

        // ─── Owners CRUD ────────────────────────────────────────────────
        .post_async("/api/owners", handlers::create_owner)
        .get_async("/api/owners", handlers::list_owners)
        .get_async("/api/owners/:id", handlers::get_owner)
        .put_async("/api/owners/:id", handlers::update_owner)
        .delete_async("/api/owners/:id", handlers::delete_owner)

        // ─── Chats CRUD ─────────────────────────────────────────────────
        .post_async("/api/chats", handlers::create_chat)
        .get_async("/api/chats", handlers::list_chats)
        .get_async("/api/chats/:id", handlers::get_chat_with_messages)
        .put_async("/api/chats/:id", handlers::update_chat)
        .delete_async("/api/chats/:id", handlers::delete_chat)

        // ─── Messages ────────────────────────────────────────────────────
        .post_async("/api/chats/:id/messages", handlers::send_message)
        .get_async("/api/chats/:id/messages", handlers::get_messages)

        // ─── 404 fallback ────────────────────────────────────────────────
        .or_else(|_req, _ctx| async move {
            Response::error("Not Found", 404)
        })

    // Run the router
    router.run(req, env).await
}
