use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::delete_object::DeleteObjectError;
use aws_sdk_s3::operation::put_object::PutObjectError;
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::operation::head_object::HeadObjectError;
use serde_json::json;
use tracing::error;

#[derive(Debug)]
pub enum AppError {
    // Client errors (4xx)
    BadRequest(String),
    NotFound(String),
    Unauthorized(String),

    // Server errors (5xx)
    TimeoutError(String),
    StorageError(String),
    DatabaseError(String),
    InternalError(String),

}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),

            AppError::TimeoutError(msg) => (StatusCode::REQUEST_TIMEOUT, msg),
            AppError::StorageError(err) => {
                error!("Storage error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Storage error {}", err))
            }
            AppError::DatabaseError(err) => {
                error!("Database error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error {}", err))
            }
            AppError::InternalError(err) => {
                error!("Internal server error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

// aws get object error mapper
impl From<SdkError<GetObjectError>> for AppError {
    fn from(err: SdkError<GetObjectError>) -> Self {
        match err {
            SdkError::ServiceError(e) if e.err().is_no_such_key() => {
                AppError::NotFound("File not found".to_string())
            }
            _ => AppError::InternalError(format!("S3 Error: {}", err))
        }
    }
}

// aws head object error mapper
impl From<SdkError<HeadObjectError>> for AppError {
    fn from(err: SdkError<HeadObjectError>) -> Self {
        match err {
            SdkError::ServiceError(e) if e.err().is_not_found() => {
                AppError::NotFound("Metadata not found".to_string())
            }
            _ => AppError::InternalError(format!("S3 Error: {}", err))
        }
    }
}

// aws put object error mapper
impl From<SdkError<PutObjectError>> for AppError {
    fn from(err: SdkError<PutObjectError>) -> Self {
        tracing::error!("S3 Put Error: {:?}", err);
        AppError::InternalError("Failed to upload object to storage".to_string())
    }
}

// aws delete object error mapper
impl From<SdkError<DeleteObjectError>> for AppError {
    fn from(err: SdkError<DeleteObjectError>) -> Self {
        tracing::error!("S3 Delete Error: {:?}", err);
        AppError::InternalError("Failed to delete object from storage".to_string())
    }
}

// anyhow error mapping
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalError(err.to_string())
    }
}

// sea_orm db error mapping
impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}