//! Input validation module.
//!
//! Provides reusable validators for all API request types.
//! Each validator returns `Result<(), String>` — `Ok(())` if valid,
//! or `Err(error_message)` if invalid.

use crate::prelude::*;

/// Trait for types that can validate their own fields.
pub trait Validator {
    /// Validate the input, returning an `AppError::Validation` on failure.
    fn validate(&self) -> std::result::Result<(), AppError>;
}

// ─── Field-level validators ──────────────────────────────────────────────────

/// Validate that a string is non-empty and within length bounds.
pub fn validate_name(name: &str, field: &str) -> std::result::Result<(), AppError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(format!(
            "{} must not be empty",
            field
        )));
    }
    if trimmed.len() > MAX_NAME_LENGTH {
        return Err(AppError::Validation(format!(
            "{} must be at most {} characters (got {})",
            field,
            MAX_NAME_LENGTH,
            trimmed.len()
        )));
    }
    if trimmed.len() < MIN_NAME_LENGTH {
        return Err(AppError::Validation(format!(
            "{} must be at least {} character(s) (got {})",
            field,
            MIN_NAME_LENGTH,
            trimmed.len()
        )));
    }
    Ok(())
}

/// Validate an email address with a basic format check.
pub fn validate_email(email: &str, field: &str) -> std::result::Result<(), AppError> {
    let trimmed = email.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(format!(
            "{} must not be empty",
            field
        )));
    }
    if trimmed.len() > MAX_EMAIL_LENGTH {
        return Err(AppError::Validation(format!(
            "{} must be at most {} characters (got {})",
            field,
            MAX_EMAIL_LENGTH,
            trimmed.len()
        )));
    }
    // Basic email validation: must contain exactly one '@', have a domain part with a dot
    let parts: Vec<&str> = trimmed.splitn(2, '@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(AppError::Validation(format!(
            "{} must be a valid email address",
            field
        )));
    }
    let domain_parts: Vec<&str> = parts[1].splitn(2, '.').collect();
    if domain_parts.len() != 2 || domain_parts[0].is_empty() || domain_parts[1].is_empty() {
        return Err(AppError::Validation(format!(
            "{} must be a valid email address",
            field
        )));
    }
    Ok(())
}

/// Validate that a title/string is within max length bounds.
pub fn validate_content(content: &str, field: &str, max_len: usize) -> std::result::Result<(), AppError> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(format!(
            "{} must not be empty",
            field
        )));
    }
    if trimmed.len() > max_len {
        return Err(AppError::Validation(format!(
            "{} must be at most {} characters (got {})",
            field, max_len, trimmed.len()
        )));
    }
    Ok(())
}

/// Validate that a string looks like a UUID v4.
pub fn validate_uuid(id: &str, field: &str) -> std::result::Result<(), AppError> {
    if id.is_empty() {
        return Err(AppError::Validation(format!(
            "{} must not be empty",
            field
        )));
    }
    // Quick check: UUID v4 format (36 chars, specific pattern)
    let bytes = id.as_bytes();
    if bytes.len() != 36 {
        return Err(AppError::Validation(format!(
            "{} must be a valid UUID (36 characters, got {})",
            field,
            bytes.len()
        )));
    }
    // Check dashes at positions 8, 13, 18, 23
    if bytes[8] != b'-' || bytes[13] != b'-' || bytes[18] != b'-' || bytes[23] != b'-' {
        return Err(AppError::Validation(format!(
            "{} must be a valid UUID format",
            field
        )));
    }
    // Check version nibble (character after third dash should be '4')
    if bytes[14] != b'4' {
        return Err(AppError::Validation(format!(
            "{} must be a UUID v4",
            field
        )));
    }
    // Check all hex characters
    for (i, &b) in bytes.iter().enumerate() {
        if i == 8 || i == 13 || i == 18 || i == 23 {
            continue; // dashes
        }
        if !b.is_ascii_hexdigit() {
            return Err(AppError::Validation(format!(
                "{} must be a valid UUID (invalid character at position {})",
                field, i
            )));
        }
    }
    Ok(())
}

// ─── Implement Validator for request types ───────────────────────────────────

impl Validator for CreateAgentRequest {
    fn validate(&self) -> std::result::Result<(), AppError> {
        validate_name(&self.name, "name")?;
        if let Some(desc) = &self.description {
            validate_content(desc, "description", MAX_DESCRIPTION_LENGTH)?;
        }
        if let Some(owner_id) = &self.owner_id {
            if !owner_id.trim().is_empty() {
                validate_uuid(owner_id, "owner_id")?;
            }
        }
        Ok(())
    }
}

impl Validator for UpdateAgentRequest {
    fn validate(&self) -> std::result::Result<(), AppError> {
        if let Some(name) = &self.name {
            validate_name(name, "name")?;
        }
        if let Some(desc) = &self.description {
            validate_content(desc, "description", MAX_DESCRIPTION_LENGTH)?;
        }
        if let Some(owner_id) = &self.owner_id {
            if !owner_id.trim().is_empty() {
                validate_uuid(owner_id, "owner_id")?;
            }
        }
        Ok(())
    }
}

impl Validator for CreateOwnerRequest {
    fn validate(&self) -> std::result::Result<(), AppError> {
        validate_name(&self.name, "name")?;
        validate_email(&self.email, "email")?;
        Ok(())
    }
}

impl Validator for UpdateOwnerRequest {
    fn validate(&self) -> std::result::Result<(), AppError> {
        if let Some(name) = &self.name {
            validate_name(name, "name")?;
        }
        if let Some(email) = &self.email {
            validate_email(email, "email")?;
        }
        Ok(())
    }
}

impl Validator for CreateChatRequest {
    fn validate(&self) -> std::result::Result<(), AppError> {
        validate_uuid(&self.agent_id, "agent_id")?;
        validate_uuid(&self.owner_id, "owner_id")?;
        if let Some(title) = &self.title {
            validate_content(title, "title", MAX_TITLE_LENGTH)?;
        }
        Ok(())
    }
}

impl Validator for UpdateChatRequest {
    fn validate(&self) -> std::result::Result<(), AppError> {
        if let Some(title) = &self.title {
            validate_content(title, "title", MAX_TITLE_LENGTH)?;
        }
        Ok(())
    }
}

impl Validator for SendMessageRequest {
    fn validate(&self) -> std::result::Result<(), AppError> {
        // Validate sender_type
        if self.sender_type != SenderType::Agent.as_str() && self.sender_type != SenderType::Owner.as_str() {
            return Err(AppError::Validation(
                "sender_type must be 'agent' or 'owner'".to_string(),
            ));
        }
        validate_uuid(&self.sender_id, "sender_id")?;
        validate_content(&self.content, "content", MAX_CONTENT_LENGTH)?;
        Ok(())
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─── validate_name ────────────────────────────────────────────────────────

    #[test]
    fn test_validate_name_valid() {
        assert!(validate_name("Alice", "name").is_ok());
        assert!(validate_name("A", "name").is_ok());
        assert!(validate_name("Bob The Agent", "name").is_ok());
    }

    #[test]
    fn test_validate_name_empty() {
        let err = validate_name("", "name").unwrap_err();
        assert_eq!(err.code(), "ERR_VALIDATION");
        assert!(err.message().contains("must not be empty"));
    }

    #[test]
    fn test_validate_name_whitespace() {
        let err = validate_name("   ", "name").unwrap_err();
        assert_eq!(err.code(), "ERR_VALIDATION");
    }

    #[test]
    fn test_validate_name_too_long() {
        let long_name = "a".repeat(MAX_NAME_LENGTH + 1);
        let err = validate_name(&long_name, "name").unwrap_err();
        assert_eq!(err.code(), "ERR_VALIDATION");
        assert!(err.message().contains("must be at most"));
    }

    #[test]
    fn test_validate_name_trimmed() {
        assert!(validate_name("  Alice  ", "name").is_ok());
    }

    // ─── validate_email ──────────────────────────────────────────────────────

    #[test]
    fn test_validate_email_valid() {
        assert!(validate_email("alice@example.com", "email").is_ok());
        assert!(validate_email("a@b.co", "email").is_ok());
        assert!(validate_email("user+tag@domain.org", "email").is_ok());
    }

    #[test]
    fn test_validate_email_empty() {
        let err = validate_email("", "email").unwrap_err();
        assert_eq!(err.code(), "ERR_VALIDATION");
    }

    #[test]
    fn test_validate_email_no_at() {
        let err = validate_email("notanemail", "email").unwrap_err();
        assert!(err.message().contains("valid email"));
    }

    #[test]
    fn test_validate_email_no_domain() {
        let err = validate_email("user@", "email").unwrap_err();
        assert!(err.message().contains("valid email"));
    }

    #[test]
    fn test_validate_email_no_tld() {
        let err = validate_email("user@domain", "email").unwrap_err();
        assert!(err.message().contains("valid email"));
    }

    #[test]
    fn test_validate_email_multi_domain() {
        assert!(validate_email("user@sub.domain.com", "email").is_ok());
    }

    // ─── validate_content ────────────────────────────────────────────────────

    #[test]
    fn test_validate_content_valid() {
        assert!(validate_content("Hello!", "content", 100).is_ok());
        assert!(validate_content("A", "content", 100).is_ok());
    }

    #[test]
    fn test_validate_content_empty() {
        let err = validate_content("", "content", 100).unwrap_err();
        assert_eq!(err.code(), "ERR_VALIDATION");
        assert!(err.message().contains("must not be empty"));
    }

    #[test]
    fn test_validate_content_exceeds_max() {
        let long = "a".repeat(101);
        let err = validate_content(&long, "content", 100).unwrap_err();
        assert!(err.message().contains("must be at most"));
    }

    #[test]
    fn test_validate_content_at_max() {
        let exact = "a".repeat(100);
        assert!(validate_content(&exact, "content", 100).is_ok());
    }

    // ─── validate_uuid ───────────────────────────────────────────────────────

    #[test]
    fn test_validate_uuid_valid() {
        assert!(validate_uuid("550e8400-e29b-41d4-a716-446655440000", "id").is_ok());
        assert!(validate_uuid("f47ac10b-58cc-4372-a567-0e02b2c3d479", "id").is_ok());
    }

    #[test]
    fn test_validate_uuid_empty() {
        let err = validate_uuid("", "id").unwrap_err();
        assert_eq!(err.code(), "ERR_VALIDATION");
    }

    #[test]
    fn test_validate_uuid_wrong_length() {
        let err = validate_uuid("too-short", "id").unwrap_err();
        assert!(err.message().contains("36 characters"));
    }

    #[test]
    fn test_validate_uuid_wrong_version() {
        // Version 1 UUID (not v4)
        let err = validate_uuid("550e8400-e29b-11d4-a716-446655440000", "id").unwrap_err();
        assert!(err.message().contains("UUID v4"));
    }

    #[test]
    fn test_validate_uuid_invalid_chars() {
        let err = validate_uuid("zzzzzzzz-zzzz-4zzz-zzzz-zzzzzzzzzzzz", "id").unwrap_err();
        assert!(err.message().contains("invalid character"));
    }

    #[test]
    fn test_validate_uuid_missing_dashes() {
        let err = validate_uuid("550e8400e29b41d4a716446655440000", "id").unwrap_err();
        // 32 chars hits the length check first
        assert!(err.message().contains("36 characters") && err.message().contains("32"));
    }

    // ─── Validator trait implementations ──────────────────────────────────────

    #[test]
    fn test_create_agent_request_validation() {
        let req = CreateAgentRequest {
            name: "TestBot".to_string(),
            description: Some("A test agent".to_string()),
            owner_id: Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_create_agent_request_invalid_name() {
        let req = CreateAgentRequest {
            name: "".to_string(),
            description: None,
            owner_id: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_create_owner_request_validation() {
        let req = CreateOwnerRequest {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_create_owner_request_invalid_email() {
        let req = CreateOwnerRequest {
            name: "Alice".to_string(),
            email: "not-an-email".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_create_chat_request_validation() {
        let req = CreateChatRequest {
            title: Some("Help chat".to_string()),
            agent_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            owner_id: "f47ac10b-58cc-4372-a567-0e02b2c3d479".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_create_chat_request_invalid_agent_id() {
        let req = CreateChatRequest {
            title: None,
            agent_id: "bad-id".to_string(),
            owner_id: "f47ac10b-58cc-4372-a567-0e02b2c3d479".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_send_message_request_validation() {
        let req = SendMessageRequest {
            sender_type: "agent".to_string(),
            sender_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            content: "Hello!".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_send_message_request_invalid_sender_type() {
        let req = SendMessageRequest {
            sender_type: "robot".to_string(),
            sender_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            content: "Hello!".to_string(),
        };
        let err = req.validate().unwrap_err();
        assert_eq!(err.code(), "ERR_VALIDATION");
        assert!(err.message().contains("sender_type"));
    }

    #[test]
    fn test_send_message_request_empty_content() {
        let req = SendMessageRequest {
            sender_type: "owner".to_string(),
            sender_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            content: "".to_string(),
        };
        assert!(req.validate().is_err());
    }
}
