use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use crate::{AppState, error::AppError};

pub async fn head_object(
    State(client): State<AppState>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("HEAD request for key {}", key);

    let metadata = client.store_client.head(&key).await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "location": metadata.location.to_string(),
            "size": metadata.size,
            "last_modified": metadata.last_modified.to_rfc3339(),
            "e_tag": metadata.e_tag,
        }))
    ))
}