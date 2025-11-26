use axum::{
    extract::{Path, State},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use sea_orm::*;
use serde_json::json;
use crate::{AppState, error::AppError, entities::file};

pub async fn delete_object(
    State(client): State<AppState>,
    Path(key): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    // Extract user ID from headers (assuming some form of authentication is done)
    let user_id: Uuid = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or(AppError::BadRequest("Missing or invalid x-user-id header".to_string()))?;
    tracing::info!("DELETE request for user {} for key {}", user_id, key);

    // Retrieve file metadata from the database
    let file = file::Entity::find()
        .filter(
            Condition::all()
                .add(file::Column::UserId.eq(user_id))
                .add(file::Column::FilePath.eq(key.clone()))
        )
        .one(&client.db)
        .await?
        .ok_or(AppError::NotFound("File not found".to_string()))?;

    // Delete the object from the Object Storage
    client.store_client.delete(file.file_key.to_string().as_str()).await?;

    // Delete the file metadata from the database
    file.delete(&client.db).await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Object deleted successfully",
            "key": key,
        }))
    ))
}