use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use crate::{error::AppError, storage::OSClient};

pub async fn get_object(
    State(client): State<OSClient>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("GET request for key {}", key);

    let data = client.get(&key).await?;

    Ok((
        StatusCode::OK,
        data
    ))
}