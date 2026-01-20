use axum::{
    body::Body,
    extract::{Path, State}, 
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::IntoResponse
};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
use uuid::Uuid;
use tokio_util::io::ReaderStream;
use crate::AppState;
use crate::error::AppError; 
use crate::entities::file;

pub async fn get_object(
    State(state): State<AppState>,
    Path(key): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {

    // Extract user ID from headers (assuming some authentication is done)
    let user_id: Uuid = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or(AppError::BadRequest("Missing or invalid x-user-id header".to_string()))?;

    // Extract version ID from headers if provided
    let version_id: Option<String> = headers
        .get("x-version-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    tracing::info!("GET request for user {}, key {}:{:?}", user_id, key, version_id);

    let mut query = file::Entity::find()
        .filter(file::Column::UserId.eq(user_id))
        .filter(file::Column::FilePath.eq(key.clone()));

    if let Some(ref vid) = version_id {
        query = query.filter(file::Column::S3VersionId.eq(vid));
    } else {
        query = query.filter(file::Column::IsLatest.eq(true));
    }

    let file_meta = query.one(&state.db).await?.ok_or_else(|| AppError::NotFound("File not found".to_string()))?;
    let s3_output = state.store_client.get(&file_meta.file_key.to_string(), version_id.as_deref()).await?;
    // from AWS Stream errors to standard Axum I/O error
    let reader = s3_output.body.into_async_read();
    let stream = ReaderStream::new(reader);
    let body = Body::from_stream(stream);

    let mut response_headers = HeaderMap::new();
    let mut insert_header = |key, value: String| {
        if let Ok(val) = HeaderValue::from_str(&value) {
            response_headers.insert(key, val);
        }
    };
    insert_header(header::CONTENT_TYPE, file_meta.content_type);
    insert_header(header::CONTENT_LENGTH, file_meta.content_size.to_string());
    insert_header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", file_meta.file_name));
    insert_header(HeaderName::from_static("x-version-id"), file_meta.s3_version_id);

    if let Some(etag) = s3_output.e_tag {
        insert_header(header::ETAG, etag);
    }

    Ok((
        StatusCode::OK,
        response_headers,
        body
    ))
}