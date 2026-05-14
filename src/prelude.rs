//! Common imports and constants for the chat app backend.
//!
//! This module re-exports commonly-used types and defines shared constants.
//! Import with: `use crate::prelude::*;`

pub use crate::models::{
    Agent, ApiResponse, AppError, Chat, ChatWithMessages, CreateAgentRequest, CreateChatRequest,
    CreateOwnerRequest, ErrorResponse, Message, Owner, PaginatedResponse, SendMessageRequest,
    SenderType, UpdateAgentRequest, UpdateChatRequest, UpdateOwnerRequest,
};
pub use serde_json;
pub use worker::{console_error, D1Database, Request, Response, RouteContext, Result};

// ─── Constants ───────────────────────────────────────────────────────────────

/// Default title for new chats when none is provided.
pub const DEFAULT_CHAT_TITLE: &str = "New Chat";

/// Maximum length for agent and owner names.
pub const MAX_NAME_LENGTH: usize = 200;

/// Minimum length for agent and owner names.
pub const MIN_NAME_LENGTH: usize = 1;

/// Maximum length for chat titles.
pub const MAX_TITLE_LENGTH: usize = 500;

/// Maximum length for message content.
pub const MAX_CONTENT_LENGTH: usize = 10_000;

/// Maximum length for agent descriptions.
pub const MAX_DESCRIPTION_LENGTH: usize = 2000;

/// Maximum length for owner emails.
pub const MAX_EMAIL_LENGTH: usize = 320;

/// Default pagination limit.
pub const DEFAULT_PAGE_LIMIT: u32 = 50;

/// Maximum pagination limit.
pub const MAX_PAGE_LIMIT: u32 = 1000;


