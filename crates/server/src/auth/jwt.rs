use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, DecodingKey, EncodingKey, Header, Validation,
};
use north_domain::UserRole;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub role: String,
    pub exp: usize,
}

pub fn create_token(
    user_id: i64,
    role: &UserRole,
    secret: &str,
) -> Result<String, AppError> {
    let role_str = match role {
        UserRole::Admin => "admin",
        UserRole::User => "user",
    };

    let exp = Utc::now() + Duration::days(7);
    let claims = Claims {
        sub: user_id,
        role: role_str.to_string(),
        exp: exp.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to create token: {e}")))
}

pub fn validate_token(
    token: &str,
    secret: &str,
) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {e}")))
}
