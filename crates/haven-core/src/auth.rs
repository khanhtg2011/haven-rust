use crate::error::{HavenError, Result};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub is_admin: bool,
    pub is_guest: bool,
    pub exp: usize,
}

pub fn hash_password(password: &str) -> Result<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| HavenError::Internal(format!("Password hash failed: {}", e)))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}

pub fn create_token(
    user_id: i64,
    username: &str,
    is_admin: bool,
    is_guest: bool,
    jwt_secret: &[u8],
    duration_secs: u64,
) -> Result<String> {
    let now = chrono::Utc::now().timestamp() as u64;
    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        is_admin,
        is_guest,
        exp: (now + duration_secs) as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret),
    )
    .map_err(|e| HavenError::Auth(format!("JWT encode error: {}", e)))
}

pub fn verify_token(token: &str, jwt_secret: &[u8]) -> Result<Claims> {
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret),
        &validation,
    )
    .map_err(|e| HavenError::Unauthorized(format!("Invalid token: {}", e)))?;

    Ok(data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "MySecurePassword123!";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash));
        assert!(!verify_password("WrongPassword", &hash));
    }

    #[test]
    fn test_jwt_lifecycle() {
        let secret = b"super_secret_haven_jwt_key_12345";
        let token = create_token(1, "admin", true, false, secret, 3600).unwrap();
        let claims = verify_token(&token, secret).unwrap();

        assert_eq!(claims.sub, 1);
        assert_eq!(claims.username, "admin");
        assert!(claims.is_admin);
        assert!(!claims.is_guest);
    }
}
