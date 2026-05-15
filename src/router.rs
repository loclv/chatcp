//! HTTP router configuration.
//!
//! Defines all API routes and maps them to the appropriate handlers.
//! This module is called from `lib.rs` to create the fully-configured router.

use worker::*;

use crate::handlers;

/// Build and return the configured HTTP router with all routes registered.
pub fn build_router() -> Router<'static, ()> {
    let router = Router::new();

    router
        // ─── CORS preflight ─────────────────────────────────────────────────
        .options_async("/*", |_req, _ctx| async move {
            handlers::handle_options()
        })

        // ─── Health ─────────────────────────────────────────────────────────
        .get_async("/api/health", |_req, _ctx| async move {
            Response::from_json(&serde_json::json!({
                "status": "ok",
                "service": "chat-app-backend",
                "version": env!("CARGO_PKG_VERSION")
            }))
        })

        // ─── Agents CRUD ────────────────────────────────────────────────────
        .post_async("/api/agents", handlers::create_agent)
        .get_async("/api/agents", handlers::list_agents)
        .get_async("/api/agents/:id", handlers::get_agent)
        .put_async("/api/agents/:id", handlers::update_agent)
        .patch_async("/api/agents/:id", handlers::update_agent)
        .delete_async("/api/agents/:id", handlers::delete_agent)
        .get_async("/api/agents/:id/chats", handlers::get_agent_chats)
        .get_async("/api/agents/:id/owner", handlers::get_agent_owner)
        .get_async("/api/agents/:id/messages", handlers::get_agent_messages)

        // ─── Owners CRUD ────────────────────────────────────────────────────
        .post_async("/api/owners", handlers::create_owner)
        .get_async("/api/owners", handlers::list_owners)
        .get_async("/api/owners/:id", handlers::get_owner)
        .put_async("/api/owners/:id", handlers::update_owner)
        .delete_async("/api/owners/:id", handlers::delete_owner)
        .get_async("/api/owners/:id/agents", handlers::get_owner_agents)
        .get_async("/api/owners/:id/chats", handlers::get_owner_chats)

        // ─── Chats CRUD ─────────────────────────────────────────────────────
        .post_async("/api/chats", handlers::create_chat)
        .get_async("/api/chats", handlers::list_chats)
        .get_async("/api/chats/:id", handlers::get_chat_with_messages)
        .put_async("/api/chats/:id", handlers::update_chat)
        .delete_async("/api/chats/:id", handlers::delete_chat)

        // ─── Messages ────────────────────────────────────────────────────────
        .post_async("/api/chats/:id/messages", handlers::send_message)
        .get_async("/api/chats/:id/messages", handlers::get_messages)

        // ─── 404 fallback ────────────────────────────────────────────────────
        .or_else_any_method_async("/*", |_req, _ctx| async move {
            Response::error("Not Found", 404)
        })
}
