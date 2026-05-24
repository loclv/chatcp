//! Common imports and constants for the chat app backend.
//!
//! This module re-exports commonly-used types and defines shared constants.
//! Import with: `use crate::prelude::*;`

pub use crate::models::{
    Agent, ApiResponse, AppError, Chat, ChatWithMessages, CreateAgentRequest, CreateChatRequest,
    CreateOwnerRequest, LoginRequest, Message, Owner, PaginatedResponse, PaginationMetadata,
    QueryParams, SendMessageRequest, SenderType, UpdateAgentRequest, UpdateChatRequest,
    UpdateOwnerRequest,
};
pub use serde_json;

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

// Default pagination limit (unused currently but kept for future use).
#[allow(unused)]
pub(crate) const DEFAULT_PAGE_LIMIT: u32 = 50;

// Maximum pagination limit (unused currently but kept for future use).
#[allow(unused)]
pub(crate) const MAX_PAGE_LIMIT: u32 = 1000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_values() {
        assert_eq!(DEFAULT_CHAT_TITLE, "New Chat");
        assert_eq!(MAX_NAME_LENGTH, 200);
        assert_eq!(MIN_NAME_LENGTH, 1);
        assert_eq!(MAX_TITLE_LENGTH, 500);
        assert_eq!(MAX_CONTENT_LENGTH, 10_000);
        assert_eq!(MAX_DESCRIPTION_LENGTH, 2000);
        assert_eq!(MAX_EMAIL_LENGTH, 320);
        assert_eq!(DEFAULT_PAGE_LIMIT, 50);
        assert_eq!(MAX_PAGE_LIMIT, 1000);
    }

    #[test]
    fn test_min_less_than_max() {
        assert!(MIN_NAME_LENGTH < MAX_NAME_LENGTH);
        assert!(DEFAULT_PAGE_LIMIT < MAX_PAGE_LIMIT);
    }

    #[test]
    fn test_reasonable_name_limit() {
        // MAX_NAME_LENGTH should be reasonable for a name field
        assert!(MAX_NAME_LENGTH >= 100, "name max should be at least 100");
        assert!(MAX_NAME_LENGTH <= 500, "name max should be at most 500");
    }

    #[test]
    fn test_reasonable_content_limit() {
        // MAX_CONTENT_LENGTH should accommodate long messages
        assert!(
            MAX_CONTENT_LENGTH >= 1000,
            "content max should be at least 1000"
        );
        assert!(
            MAX_CONTENT_LENGTH <= 100_000,
            "content max should be at most 100k"
        );
    }
}
