use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;
use crate::AppState;
use crate::error::AppError;
use crate::entities::file;

pub async fn head_object(
    State(state): State<AppState>,
    Path(key): Path<String>,
    headers: HeaderMap
) -> Result<impl IntoResponse, AppError> {
    let user_id: Uuid = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or(AppError::BadRequest("Missing or invalid x-user-id header".to_string()))?;
    let version_id = headers
        .get("x-version-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    tracing::info!("HEAD request for user {} and key {}:{:?}", user_id, key, version_id);

    let mut query = file::Entity::find()
        .filter(file::Column::UserId.eq(user_id))
        .filter(file::Column::FilePath.eq(&key));

    // if there is a version_id set, we take this version, otherwise the latest version
    if let Some(v_id) = version_id {
        query = query.filter(file::Column::Version.eq(v_id));
    } else {
        query = query.filter(file::Column::IsLatest.eq(true));
    }

    let file = query.one(&state.db).await?.ok_or(AppError::NotFound("File not found".to_string()))?;

    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::CONTENT_LENGTH, HeaderValue::from(file.content_size as u64));

    if let Ok(val) = HeaderValue::from_str(&file.content_type) {
        response_headers.insert(header::CONTENT_TYPE, val);
    }

    let last_modified = file.added_at.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
    if let Ok(val) = HeaderValue::from_str(&last_modified) {
        response_headers.insert(header::LAST_MODIFIED, val);
    }

    let etag_val = format!("\"{}\"", file.version);
    if let Ok(val) = HeaderValue::from_str(&etag_val) {
        response_headers.insert(header::ETAG, val);
    }

    if let Ok(val) = HeaderValue::from_str(&file.version) {
        response_headers.insert("x-amz-version-id", val);
    }

    Ok((
        StatusCode::OK,
        response_headers
    ))
}