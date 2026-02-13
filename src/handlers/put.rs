use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use mime_guess;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::entities::{file, user};
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

fn content_type_from_headers_or_path(headers: &HeaderMap, key: &str) -> String {
    headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| mime_guess::from_path(key).first_or_octet_stream().to_string())
}

fn file_name_from_key(key: &str) -> String {
    key.split('/').last().unwrap_or(key).to_string()
}

fn build_created_response(
    key: String,
    s3_key_string: String,
    s3_version_id: String,
) -> (StatusCode, Json<Value>) {
    (
        StatusCode::CREATED,
        Json(json!({
            "message": "New object created successfully",
            "file_path": key,
            "file_key": s3_key_string,
            "version": s3_version_id,
        })),
    )
}

pub async fn put_object(
    State(state): State<AppState>,
    Path(key): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    // Extract user ID from headers (assuming some authentication is done)
    let user_id: Uuid = extract_user_id(&headers)?;

    // Extract Content-Type and Content-Length from headers
    let content_type = content_type_from_headers_or_path(&headers, &key);
    let content_size = body.len() as i64;
    let file_name = file_name_from_key(&key);

    tracing::info!(
        "PUT request from user {} for key {} ({} bytes)",
        user_id,
        key,
        content_size
    );

    // Create or update user record if not exists
    let user_exists = user::Entity::find_by_id(user_id).one(&state.db).await?;
    // If user does not exist, we create a new user record
    if user_exists.is_none() {
        let new_user = user::ActiveModel::new(user_id, 0);
        new_user.insert(&state.db).await?;
        tracing::info!("Created new user profile {}", user_id);
    }

    let new_file_uuid = Uuid::now_v7();
    let s3_key_string = new_file_uuid.to_string();

    let s3_output = state.store_client.put(&s3_key_string, body).await?;
    let s3_version_id = s3_output.version_id.unwrap_or_else(|| "null".to_string());

    // transactionnal update
    let txn = state.db.begin().await?;

    let existing_active_file = file::Entity::find()
        .filter(file::Column::UserId.eq(user_id))
        .filter(file::Column::FilePath.eq(key.clone()))
        .filter(file::Column::IsLatest.eq(true))
        .one(&txn)
        .await?;
    if let Some(old_file) = existing_active_file {
        let mut old_active: file::ActiveModel = old_file.into();
        old_active.is_latest = Set(false);
        old_active.update(&txn).await?;
    }
    let new_file_entry = file::ActiveModel::new(
        new_file_uuid,
        user_id,
        file_name,
        key.clone(),
        content_type,
        content_size,
        s3_version_id.clone(),
    );
    new_file_entry.insert(&txn).await?;

    // commit transaction
    txn.commit().await?;

    Ok(build_created_response(key, s3_key_string, s3_version_id))
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
    fn content_type_from_headers_or_path_prefers_header() {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/custom"));

        let ct = content_type_from_headers_or_path(&headers, "path/to/file.txt");
        assert_eq!(ct, "application/custom");
    }

    #[test]
    fn content_type_from_headers_or_path_falls_back_to_mime_guess() {
        let headers = HeaderMap::new();

        let ct = content_type_from_headers_or_path(&headers, "path/to/file.txt");
        assert_eq!(ct, "text/plain");
    }

    #[test]
    fn file_name_from_key_extracts_last_segment() {
        assert_eq!(file_name_from_key("a/b/c.txt"), "c.txt");
        assert_eq!(file_name_from_key("single"), "single");
    }

    #[test]
    fn build_created_response_has_expected_shape() {
        let (status, Json(body)) = build_created_response(
            "path/to/file.txt".to_string(),
            "s3-key-1".to_string(),
            "ver-123".to_string(),
        );

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body["message"], "New object created successfully");
        assert_eq!(body["file_path"], "path/to/file.txt");
        assert_eq!(body["file_key"], "s3-key-1");
        assert_eq!(body["version"], "ver-123");
    }
}