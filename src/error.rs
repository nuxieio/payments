use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Store API error: {0}")]
    StoreApiError(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::DatabaseError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, message.clone()),
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message.clone()),
            AppError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message.clone()),
            AppError::ValidationError(message) => (StatusCode::BAD_REQUEST, message.clone()),
            AppError::StoreApiError(message) => (StatusCode::BAD_GATEWAY, message.clone()),
            AppError::InternalServerError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message.clone()),
            AppError::Other(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        let body = Json(json!({
            "error": {
                "message": message,
                "status": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
