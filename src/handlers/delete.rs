use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::entities::file;
use crate::error::AppError;
use crate::AppState;

fn extract_user_id(headers: &HeaderMap) -> Result<Uuid, AppError> {
    headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or(AppError::BadRequest(
            "Missing or invalid x-user-id header".to_string(),
        ))
}

fn extract_version_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-version-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

fn build_deleted_response(key: String, version_id: String) -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "message": "Object deleted successfully",
            "key": key,
            "version_id": version_id
        })),
    )
}

pub async fn delete_object(
    State(state): State<AppState>,
    Path(key): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    // Extract user ID from headers (assuming some form of authentication is done)
    let user_id: Uuid = extract_user_id(&headers)?;

    let version_id: Option<String> = extract_version_id(&headers);

    tracing::info!(
        "DELETE request for user {}, key: {}:{:?}",
        user_id,
        key,
        version_id
    );

    let mut query = file::Entity::find()
        .filter(file::Column::UserId.eq(user_id))
        .filter(file::Column::FilePath.eq(key.clone()));
    if let Some(ref vid) = version_id {
        query = query.filter(file::Column::S3VersionId.eq(vid));
    } else {
        query = query.filter(file::Column::IsLatest.eq(true));
    }

    let file_meta = query
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("File not found".to_string()))?;
    let file_version_id = file_meta.s3_version_id.to_string();

    // delete from s3 storage
    state
        .store_client
        .delete(&file_meta.file_key.to_string(), Some(&file_version_id))
        .await?;

    // delete from db
    file_meta.delete(&state.db).await?;

    tracing::info!("Deleted file {} (version: {})", key, file_version_id);

    Ok(build_deleted_response(key, file_version_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn extract_user_id_missing_is_bad_request() {
        let headers = HeaderMap::new();
        let err = extract_user_id(&headers).unwrap_err();

        match err {
            AppError::BadRequest(msg) => assert!(msg.contains("x-user-id")),
            other => panic!("expected BadRequest, got: {:?}", other),
        }
    }

    #[test]
    fn extract_user_id_invalid_is_bad_request() {
        let mut headers = HeaderMap::new();
        headers.insert("x-user-id", HeaderValue::from_static("not-a-uuid"));

        let err = extract_user_id(&headers).unwrap_err();
        match err {
            AppError::BadRequest(msg) => assert!(msg.contains("invalid")),
            other => panic!("expected BadRequest, got: {:?}", other),
        }
    }

    #[test]
    fn extract_user_id_valid() {
        let mut headers = HeaderMap::new();
        let u = Uuid::now_v7();
        headers.insert("x-user-id", HeaderValue::from_str(&u.to_string()).unwrap());

        let parsed = extract_user_id(&headers).unwrap();
        assert_eq!(parsed, u);
    }

    #[test]
    fn extract_version_id_absent_is_none() {
        let headers = HeaderMap::new();
        assert_eq!(extract_version_id(&headers), None);
    }

    #[test]
    fn extract_version_id_present() {
        let mut headers = HeaderMap::new();
        headers.insert("x-version-id", HeaderValue::from_static("v42"));

        assert_eq!(extract_version_id(&headers), Some("v42".to_string()));
    }

    #[test]
    fn build_deleted_response_has_expected_shape() {
        let (status, Json(body)) =
            build_deleted_response("path/to/file.txt".to_string(), "ver-123".to_string());

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["message"], "Object deleted successfully");
        assert_eq!(body["key"], "path/to/file.txt");
        assert_eq!(body["version_id"], "ver-123");
    }
}