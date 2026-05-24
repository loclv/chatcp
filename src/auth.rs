use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use worker::*;

use crate::db;
use crate::models::{Agent, AppError, Owner};

// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // owner_id
    pub exp: u64,    // Expiration timestamp (seconds since epoch)
}

/// Simple password hashing using SHA-256 + Salt
pub fn hash_password(password: &str, salt: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Generate a new JWT for an owner.
pub fn generate_jwt(owner_id: &str, secret: &str, expires_in_secs: u64) -> Result<String> {
    // Current time in seconds
    #[cfg(target_arch = "wasm32")]
    let now = Date::now().as_millis() / 1000;
    #[cfg(not(target_arch = "wasm32"))]
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let claims = Claims {
        sub: owner_id.to_string(),
        exp: now + expires_in_secs,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| worker::Error::from(format!("Failed to sign JWT: {}", e)))
}

/// Verify a JWT and extract the subject (owner_id).
pub fn verify_jwt(token: &str, secret: &str) -> std::result::Result<String, AppError> {
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| AppError::Validation(format!("Invalid token: {}", e)))?;

    Ok(token_data.claims.sub)
}

/// Authentication context for a request.
#[derive(Debug, Clone)]
pub enum AuthContext {
    Owner(Owner),
    Agent(Agent),
}

impl AuthContext {
    /// Retrieve the owner if this context is an Owner, or return a 401/403 error.
    pub fn get_owner(self) -> std::result::Result<Owner, AppError> {
        match self {
            AuthContext::Owner(owner) => Ok(owner),
            _ => Err(AppError::BadRequest(
                "Only owners are permitted to perform this action".to_string(),
            )),
        }
    }

    /// Retrieve the agent if this context is an Agent, or return a 401/403 error.
    #[allow(dead_code)]
    pub fn get_agent(self) -> std::result::Result<Agent, AppError> {
        match self {
            AuthContext::Agent(agent) => Ok(agent),
            _ => Err(AppError::BadRequest(
                "Only agents are permitted to perform this action".to_string(),
            )),
        }
    }
}

/// Authenticate an incoming request.
/// Supports both JWT ("Authorization: Bearer <JWT>") and API Key ("Authorization: ApiKey <KEY>").
pub async fn authenticate_request(
    req: &Request,
    d1: &D1Database,
    jwt_secret: &str,
) -> std::result::Result<AuthContext, AppError> {
    let headers = req.headers();
    let auth_header = match headers.get("Authorization") {
        Ok(Some(val)) => val,
        _ => {
            return Err(AppError::Validation(
                "Authorization header is required".to_string(),
            ))
        },
    };

    let parts: Vec<&str> = auth_header.splitn(2, ' ').collect();
    if parts.len() != 2 {
        return Err(AppError::Validation(
            "Invalid Authorization header format. Expected 'Bearer <JWT>' or 'ApiKey <KEY>'"
                .to_string(),
        ));
    }

    let auth_type = parts[0];
    let token = parts[1].trim();

    if auth_type.eq_ignore_ascii_case("Bearer") {
        // Owner JWT auth
        let owner_id = verify_jwt(token, jwt_secret)?;
        // Fetch owner from DB
        match db::get_owner_raw(d1, &owner_id).await? {
            Some(owner) => Ok(AuthContext::Owner(owner)),
            None => Err(AppError::NotFound("Owner not found".to_string())),
        }
    } else if auth_type.eq_ignore_ascii_case("ApiKey") {
        // Check owners table first
        if let Some(owner) = db::get_owner_by_api_key(d1, token).await? {
            return Ok(AuthContext::Owner(owner));
        }
        // Check agents table next
        if let Some(agent) = db::get_agent_by_api_key(d1, token).await? {
            return Ok(AuthContext::Agent(agent));
        }

        Err(AppError::Validation("Invalid API key".to_string()))
    } else {
        Err(AppError::Validation(format!(
            "Unsupported authorization type: '{}'",
            auth_type
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_consistency() {
        let password = "my_secure_password";
        let salt = "random_salt_123";
        let hash1 = hash_password(password, salt);
        let hash2 = hash_password(password, salt);
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, password);
    }

    #[test]
    fn test_hash_password_diff_salt() {
        let password = "my_secure_password";
        let hash1 = hash_password(password, "salt1");
        let hash2 = hash_password(password, "salt2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_jwt_generation_and_verification() {
        let owner_id = "owner_12345";
        let secret = "my_super_secret_key_987654321";

        let token = generate_jwt(owner_id, secret, 3600).unwrap();
        let verified_id = verify_jwt(&token, secret).unwrap();

        assert_eq!(owner_id, verified_id);
    }

    #[test]
    fn test_jwt_verification_wrong_secret() {
        let owner_id = "owner_12345";
        let secret = "my_super_secret_key_987654321";
        let wrong_secret = "wrong_secret_key_123";

        let token = generate_jwt(owner_id, secret, 3600).unwrap();
        let err = verify_jwt(&token, wrong_secret);

        assert!(err.is_err());
    }

    #[test]
    fn test_jwt_invalid_token() {
        let secret = "my_super_secret_key_987654321";
        let err = verify_jwt("invalid.jwt.token", secret);
        assert!(err.is_err());
    }
}
