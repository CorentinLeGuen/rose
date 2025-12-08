use axum::{
    Json, body::Bytes, extract::{Path, State}, http::{HeaderMap, StatusCode}, response::IntoResponse
};
use mime_guess;
use uuid::Uuid;
use sea_orm::*;
use serde_json::json;
use crate::{AppState, error::AppError, entities::user, entities::file};

pub async fn put_object(
    State(client): State<AppState>,
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

    tracing::info!("PUT request from user {} for key {} ({} bytes)", user_id, key, content_size);

    // Create or update user record if not exists
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

    // Check if file with the same key already exists for the user
    let existing_file = file::Entity::find()
        .filter(file::Column::UserId.eq(user_id))
        .filter(file::Column::FilePath.eq(key.clone()))
        .filter(file::Column::IsLatest.eq(true))
        .one(&client.db)
        .await?;
    if existing_file.is_some() {
        // Extracting the existing file to updateselect 
        let mut existing_file: file::ActiveModel = existing_file.unwrap().into();

        // Create a new version of the existing file
        let new_file_version = file::ActiveModel::new(
            existing_file.file_key.clone().unwrap(),
            user_id,
            key.split('/').last().unwrap_or(&key).to_string(),
            key.clone(),
            content_type.clone(),
            content_size,
            "0".to_string(), // Placeholder for version, should be updated by Object Storage response
        );
        let file_record = new_file_version.insert(&client.db).await?;
        let file_key = file_record.file_key.to_string();

        // Store the object in the Object Storage
        let put_result = client.store_client.put(file_key.to_string().as_str(), body).await?;
        let file_version = put_result.version_id.unwrap_or_default();

        // Update file record with version info from Object Storage
        let mut file_updated: file::ActiveModel = file_record.into();
        file_updated.version = Set(file_version.clone());
        file_updated.update(&client.db).await?;

        // Mark the existing file as not latest
        existing_file.is_latest = Set(false);
        existing_file.update(&client.db).await?;

        Ok((
            StatusCode::CREATED,
            Json(json!({
                "message": "New object version created successfully",
                "file_path": key,
                "file_key": file_key,
                "version": file_version,
            }))
        ))
    } else {
        // Save file metadata to the database
        let new_file = file::ActiveModel::new(
            Uuid::now_v7(),
            user_id,
            key.split('/').last().unwrap_or(&key).to_string(),
            key.clone(),
            content_type,
            content_size,
            "0".to_string(), // Placeholder for version, should be updated by Object Storage response
        );
        let file_record = new_file.insert(&client.db).await?;
        let file_key = file_record.file_key.to_string();

        // Store the object in the Object Storage
        let put_result = client.store_client.put(file_key.to_string().as_str(), body).await?;
        let file_version = put_result.version_id.unwrap_or_default();

        // Update file record with version info from Object Storage
        let mut file_updated: file::ActiveModel = file_record.into();
        file_updated.version = Set(file_version.clone());
        file_updated.update(&client.db).await?;

        Ok((
            StatusCode::CREATED,
            Json(json!({
                "message": "New object created successfully",
                "file_path": key,
                "file_key": file_key,
                "version": file_version,
            }))
        ))
    }
}