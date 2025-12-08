use aws_sdk_s3::{
    operation::delete_object::DeleteObjectError,
    error::SdkError,
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Error)]
pub enum StorageErrorKind {
    StorageTimeout(String),
}

#[derive(Error)]
pub enum AppError {
    NotFound(String),
    TimeoutError(String),
    #[error("Storage error: {0}")]
    StorageError(StorageErrorKind),
    DatabaseError(sea_orm::DbErr),
    InternalError(anyhow::Error),
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::TimeoutError(msg) => (StatusCode::REQUEST_TIMEOUT, msg),
            AppError::StorageError(err) => {

                tracing::error!("Storage error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Storage error {}", err))
            }
            AppError::DatabaseError(err) => {
                tracing::error!("Database error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error {}", err))
            }
            AppError::InternalError(err) => {
                tracing::error!("Internal server error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl From<aws_sdk_s3::Error> for AppError {
    fn from(err: aws_sdk_s3::Error) -> Self {
        match err {
            aws_sdk_s3::Error::NotFound { .. } => {
                AppError::NotFound("Object not found".to_string())
            }
            aws_sdk_s3::Error::NoSuchKey { .. } => {
                AppError::NotFound("No object found for key provided".to_string())
            }
            _ => AppError::StorageError(err),
        }
    }
}

// Mapping S3 errors
impl From<SdkError<DeleteObjectError>> for AppError {
    fn from(err: SdkError<DeleteObjectError>) -> Self {
        match &err {
            SdkError::TimeoutError(_e) => {
                AppError::TimeoutError("Deletion request went on timeout".to_string())
            }
            SdkError::ResponseError(_e) => {
                AppError::StorageError(_e)
            }
            _ => AppError::from(anyhow::anyhow!("S3 delete failed: {}", err))
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalError(err)
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        AppError::DatabaseError(err)
    }
}