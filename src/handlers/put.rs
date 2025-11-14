use axum::{
    Json, body::Bytes, extract::{Path, State}, http::{HeaderMap, StatusCode}, response::IntoResponse
};
use mime_guess;
use uuid::Uuid;
use sea_orm::*;
use rose::entities::file;
use serde_json::json;
use crate::{AppState, error::AppError, entities::user};

pub async fn put_object(
    State(client): State<AppState>,
    Path(key): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    // Extract user ID from headers (assuming some form of authentication is done)
    let user_id: Uuid = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or(AppError::BadRequest("Missing or invalid x-user-id header".to_string()))?;
    let content_type = mime_guess::from_path(&key).first_or_octet_stream().to_string();
    let content_size = body.len() as i64;

    tracing::info!("PUT request from user {} for key {} ({} bytes)", user_id, key, content_size);

    // Store the object in the Object Storage
    let put_result = client.store_client.put(&key, body).await?;

    let user_exists = user::Entity::find()
        .filter(user::Column::UserId.eq(user_id))
        .one(&client.db)
        .await?;
    // If user does not exist, we create a new user record
    if user_exists.is_none() {
        let new_user = user::ActiveModel::new(
            user_id,
            0,
        );
        new_user.insert(&client.db).await?;
        tracing::info!("Created new user with user_id {}", user_id);
    }

    // Save file metadata to the database
    let new_file = file::ActiveModel::new(
        user_id,
        key.split('/').last().unwrap_or(&key).to_string(),
        key.clone(),
        content_type,
        content_size,
        put_result.version.unwrap_or_default(),
    );
    let file_record = new_file.insert(&client.db).await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "Object created successfully",
            "key": key,
            "file_id": file_record.file_key,
        }))
    ))
}