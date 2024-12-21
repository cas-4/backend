use core::fmt;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// All errors raised by the web app
#[derive(Debug)]
pub enum AppError {
    /// Database error
    Database(String),
    /// Generic bad request. It is handled with a message value
    BadRequest(String),
    /// Not found error
    NotFound(String),
    /// Raised when a token is not good created
    TokenCreation,
    /// Raised when a passed token is not valid
    InvalidToken,
    /// Raised if an user wants to do something can't do
    Unauthorized,
}

/// Use `AppError` as response for an endpoint
impl IntoResponse for AppError {
    /// Matches `AppError` into a tuple of status and error message.
    /// The response will be a JSON in the format of:
    /// ```json
    /// { "error": "<message>" }
    /// ```
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(value) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error with database connection: {value}"),
            ),
            AppError::BadRequest(value) => (StatusCode::BAD_REQUEST, value),
            AppError::NotFound(value) => (StatusCode::NOT_FOUND, value),
            AppError::TokenCreation => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Token creation error".to_string(),
            ),
            AppError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token".to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

/// Raise a generic error from a string
impl From<std::string::String> for AppError {
    fn from(error: std::string::String) -> AppError {
        AppError::BadRequest(error)
    }
}

/// Raise a generic io error
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::BadRequest(error.to_string())
    }
}

/// Implementation of the `{}` marker for AppError
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::Database(value) => write!(f, "Database: {}", value),
            AppError::BadRequest(value) => write!(f, "BadRequest: {}", value),
            AppError::NotFound(value) => write!(f, "Not found: {}", value),
            AppError::TokenCreation => write!(f, "Token creation"),
            AppError::InvalidToken => write!(f, "Invalid Token"),
            AppError::Unauthorized => write!(f, "Unauthorized"),
        }
    }
}

/// A tokio_postgres error is mapped to an `AppError::Database`
impl From<tokio_postgres::Error> for AppError {
    fn from(value: tokio_postgres::Error) -> Self {
        AppError::Database(value.to_string())
    }
}

/// A async_graphql error is mapped to an `AppError::BadRequest`
impl From<async_graphql::Error> for AppError {
    fn from(value: async_graphql::Error) -> Self {
        AppError::BadRequest(value.message)
    }
}

/// A expo_push_notification_client::ValidationError is mapped to an `AppError::BadRequest`
impl From<expo_push_notification_client::ValidationError> for AppError {
    fn from(value: expo_push_notification_client::ValidationError) -> Self {
        AppError::BadRequest(value.to_string())
    }
}
