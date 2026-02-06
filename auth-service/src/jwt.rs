use common::models::Claims;
use jsonwebtoken::{EncodingKey, Header, encode};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct JwtService {
    encoding_key: EncodingKey,
}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
        }
    }

    pub fn generate_token(
        &self,
        user_id: &str,
        role: &str,
        permissions: Vec<String>,
        exp_hours: u64,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + (exp_hours * 3600);

        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp as usize,
            role: role.to_string(),
            permissions,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
    }
}
