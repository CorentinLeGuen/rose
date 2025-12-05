use axum::{
    body::Body, 
    extract::{Path, State}, 
    http::{ header, HeaderMap, StatusCode }, 
    response::IntoResponse
};
use uuid::Uuid;
use sea_orm::*;
use crate::{AppState, error::AppError, entities::file};

pub async fn get_object(
    State(client): State<AppState>,
    Path(key): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    // Extract user ID from headers (assuming some authentication is done)
    let user_id: Uuid = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or(AppError::BadRequest("Missing or invalid x-user-id header".to_string()))?;
    tracing::info!("GET request for user {} and key {}", user_id, key);

    // Retrieve file metadata from the database
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

    // Retrieve the object from the Object Storage
    let stream = client.store_client.get(file.file_key.to_string().as_str()).await?;
    let body = Body::from_stream(stream);

    // header management
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, file.content_type.parse().unwrap());
    headers.insert(header::CONTENT_LENGTH, file.content_size.to_string().parse().unwrap());
    headers.insert(header::CONTENT_DISPOSITION, format!("attachement; filename=\"{}\"", file.file_name).parse().unwrap());

    Ok((
        StatusCode::OK,
        headers,
        body
    ))
}