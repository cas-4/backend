use crate::errors::AppError;
use async_graphql::{InputObject, SimpleObject};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    typed_header::TypedHeader,
};
use chrono::{Duration, Local};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = &crate::config::CONFIG.jwt_secret;
    Keys::new(secret.as_bytes())
});

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

/// Claims struct.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Claims {
    user_id: i32,
    exp: usize,
}

/// Authentication enum
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Authentication {
    Logged(Claims),
    NotLogged,
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

#[async_trait]
impl<S> FromRequestParts<S> for Authentication
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        // Extract the Authorization header
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, &()).await {
            Ok(TypedHeader(Authorization(bearer))) => {
                // Decode the token
                let token_data =
                    decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
                        .map_err(|err| match err.kind() {
                            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                                AppError::InvalidToken
                            }
                            _ => {
                                eprintln!("{err:?}");
                                return AppError::Unauthorized;
                            }
                        })?;

                Ok(Self::Logged(token_data.claims))
            }
            Err(_) => Ok(Self::NotLogged),
        }
    }
}
