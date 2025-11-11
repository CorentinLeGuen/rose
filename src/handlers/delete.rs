use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use crate::{error::AppError, storage::OSClient};

pub async fn delete_object(
    State(client): State<OSClient>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("DELETE request for key {}", key);

    client.delete(&key).await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Object deleted successfully",
            "key": key,
        }))
    ))
}