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
    Database,
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
            AppError::Database => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error with database connection".to_string(),
            ),
            AppError::BadRequest(value) => (StatusCode::BAD_REQUEST, value),
            AppError::NotFound(value) => (StatusCode::NOT_FOUND, value),
            AppError::TokenCreation => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Token creation error".to_string(),
            ),
            AppError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token".to_string()),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Can't perform this action".to_string(),
            ),
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
