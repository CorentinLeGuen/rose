use axum::{
    extract::{Path, State},
    http::{StatusCode, header, HeaderMap},
    response::IntoResponse,
};
use uuid::Uuid;
use sea_orm::*;
use crate::{AppState, error::AppError, entities::file};

pub async fn head_object(
    State(client): State<AppState>,
    Path(key): Path<String>,
    headers: HeaderMap
) -> Result<impl IntoResponse, AppError> {
    let user_id: Uuid = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or(AppError::BadRequest("Missing or invalid x-user-id header".to_string()))?;
    tracing::info!("HEAD request for user {} and key {}", user_id, key);

    let file = file::Entity::find()
        .filter(
            Condition::all()
                .add(file::Column::UserId.eq(user_id))
                .add(file::Column::FilePath.eq(key.clone()))
                .add(file::Column::IsLatest.eq(true))
        )
        .one(&client.db)
        .await?
        .ok_or(AppError::NotFound("File not found".to_string()))?;
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_LENGTH, file.content_size.to_string().parse().unwrap());
    headers.insert(header::CONTENT_TYPE, file.content_type.parse().unwrap());
    headers.insert(header::CONTENT_LOCATION, file.file_path.parse().unwrap());
    headers.insert(header::DATE, file.added_at.to_string().parse().unwrap());

    Ok((
        StatusCode::OK,
        headers
    ))
}