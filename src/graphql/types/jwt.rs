use crate::errors::AppError;
use async_graphql::{InputObject, SimpleObject};
use chrono::{Duration, Local};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

struct Keys {
    encoding: EncodingKey,
    _decoding: DecodingKey,
}

static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = &crate::config::CONFIG.jwt_secret;
    Keys::new(secret.as_bytes())
});

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            _decoding: DecodingKey::from_secret(secret),
        }
    }
}

/// Claims struct
#[derive(Serialize, Deserialize)]
pub struct Claims {
    /// ID from the user model
    pub user_id: i32,
    /// Expiration timestamp
    exp: usize,
}

impl Claims {
    /// Create a new Claim using the `user_id` and the current timestamp + 2 days
    pub fn new(user_id: i32) -> Self {
        let expiration = Local::now() + Duration::days(2);

        Self {
            user_id,
            exp: expiration.timestamp() as usize,
        }
    }

    /// Returns the token as a string. If a token is not encoded, raises an
    /// `AppError::TokenCreation`
    pub fn get_token(&self) -> Result<String, AppError> {
        let token = encode(&Header::default(), &self, &KEYS.encoding)
            .map_err(|_| AppError::TokenCreation)?;

        Ok(token)
    }
}

#[derive(InputObject, Debug)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

/// Body used as response to login
#[derive(Serialize, SimpleObject)]
pub struct AuthBody {
    /// Access token string
    access_token: String,
    /// "Bearer" string
    token_type: String,
}

impl AuthBody {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}
