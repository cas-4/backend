use crate::errors::AppError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

pub async fn page_404() -> impl IntoResponse {
    AppError::NotFound("Route not found".to_string())
}

/// Extension of `Json` which returns the CREATED status code
pub struct JsonCreate<T>(pub T);

impl<T> IntoResponse for JsonCreate<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        (StatusCode::CREATED, Json(self.0)).into_response()
    }
}
