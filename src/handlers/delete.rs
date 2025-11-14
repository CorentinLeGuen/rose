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

    client.store_client.delete(&key).await?;

    let file = file::Entity::find()
        .filter(file::Column::FilePath.eq(key.clone()))
        .one(&client.db)
        .await?
        .ok_or(AppError::NotFound("File metadata not found".to_string()))?;
    file.delete(&client.db).await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Object deleted successfully",
            "key": key,
        }))
    ))
}