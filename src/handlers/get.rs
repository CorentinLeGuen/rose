use axum::{
    body::Body, extract::{Path, State}, http::StatusCode, response::IntoResponse
};
use crate::{error::AppError, storage::OSClient};

pub async fn get_object(
    State(client): State<OSClient>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("GET request for key {}", key);

    let stream = client.get(&key).await?;

    let body = Body::from_stream(stream);

    Ok((
        StatusCode::OK,
        body
    ))
}