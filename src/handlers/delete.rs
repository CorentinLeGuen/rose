use axum::{
    extract::{Path, State},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
    Json,
};
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};
use serde_json::json;
use uuid::Uuid;
use crate::AppState;
use crate::error::AppError;
use crate::entities::file;

pub async fn delete_object(
    State(state): State<AppState>,
    Path(key): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    // Extract user ID from headers (assuming some form of authentication is done)
    let user_id: Uuid = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or(AppError::BadRequest("Missing or invalid x-user-id header".to_string()))?;

    let version_id: Option<String> = headers
        .get("x-version-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    tracing::info!("DELETE request for user {}, key: {}:{:?}", user_id, key, version_id);

    let mut query = file::Entity::find()
        .filter(file::Column::UserId.eq(user_id))
        .filter(file::Column::FilePath.eq(key.clone()));
    if let Some(ref vid) = version_id {
        query = query.filter(file::Column::S3VersionId.eq(vid));
    } else {
        query = query.filter(file::Column::IsLatest.eq(true));
    }

    let file_meta = query.one(&state.db).await?.ok_or(AppError::NotFound("File not found".to_string()))?;
    let file_version_id = file_meta.s3_version_id.to_string();

    // delete from s3 storage
    state.store_client
        .delete(
            &file_meta.file_key.to_string(), 
            Some(&file_version_id)
        )
        .await
        .map_err(|e| {
            tracing::error!("S3 Delete error: {:?}", e);
            AppError::InternalError("Failed to delete object from storage".to_string())
        })?;

    // delete from db
    file_meta.delete(&state.db).await?;

    tracing::info!("Deleted file {} (version: {})", key, file_version_id);

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Object deleted successfully",
            "key": key,
            "version_id": file_version_id
        }))
    ))
}