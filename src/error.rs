use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    StorageError(object_store::Error),
    InternalError(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::StorageError(err) => {
                tracing::error!("Storage error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Storage error {}", err))
            }
            AppError::InternalError(err) => {
                tracing::error!("Internal server error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl From<object_store::Error> for AppError {
    fn from(err: object_store::Error) -> Self {
        match err {
            object_store::Error::NotFound { .. } => {
                AppError::NotFound("Object not found".to_string())
            }
            _ => AppError::StorageError(err),
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalError(err)
    }
}