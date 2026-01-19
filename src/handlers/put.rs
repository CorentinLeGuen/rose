use axum::{
    Json, 
    body::Bytes, 
    extract::{Path, State}, 
    http::{HeaderMap, StatusCode}, 
    response::IntoResponse
};
use mime_guess;
use uuid::Uuid;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use serde_json::json;
use crate::AppState;
use crate::error::AppError;
use crate::entities::{user, file};

pub async fn put_object(
    State(state): State<AppState>,
    Path(key): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    // Extract user ID from headers (assuming some authentication is done)
    let user_id: Uuid = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or(AppError::BadRequest("Missing or invalid x-user-id header".to_string()))?;

    // Extract Content-Type and Content-Length from headers
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or(mime_guess::from_path(&key).first_or_octet_stream().to_string().as_str())
        .to_string();
    let content_size = body.len() as i64;
    let file_name = key.split('/').last().unwrap_or(&key).to_string();

    tracing::info!("PUT request from user {} for key {} ({} bytes)", user_id, key, content_size);

    // Create or update user record if not exists
    let user_exists = user::Entity::find_by_id(user_id).one(&state.db).await?;
    // If user does not exist, we create a new user record
    if user_exists.is_none() {
        let new_user = user::ActiveModel::new(
            user_id,
            0,
        );
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
        s3_version_id.clone()
    );
    new_file_entry.insert(&txn).await?;
    
    // commit transaction
    txn.commit().await?;
    

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "New object created successfully",
            "file_path": key,
            "file_key": s3_key_string,
            "version": s3_version_id,
        }))
    ))
}