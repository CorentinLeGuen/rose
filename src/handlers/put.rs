use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use crate::{error::AppError, storage::OSClient};

pub async fn put_object(
    State(client): State<OSClient>,
    Path(key): Path<String>,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("PUT request for key {} ({} bytes)", key, body.len());

    client.put(&key, body).await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "Object created successfully",
            "key": key,
        }))
    ))
}