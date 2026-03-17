// backend/src/auth.rs
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};
use shared::UserRole;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub:   Uuid,
    pub email: String,
    pub role:  UserRole,
    pub exp:   usize,
}

pub fn create_token(secret: &str, user_id: Uuid, email: &str, role: UserRole) -> anyhow::Result<String> {
    let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;
    let claims = Claims { sub: user_id, email: email.to_string(), role, exp };
    Ok(encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))?)
}

pub fn verify_token(secret: &str, token: &str) -> anyhow::Result<Claims> {
    Ok(decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?.claims)
}

pub fn generate_otp() -> String {
    format!("{:06}", rand::rng().random_range(0..=999999u32))
}
