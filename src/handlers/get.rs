use axum::{
    body::Body, 
    extract::{Path, State}, 
    http::{ header, HeaderMap, StatusCode }, 
    response::IntoResponse
};
use mime_guess;
use crate::{AppState, error::AppError};

pub async fn get_object(
    State(client): State<AppState>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("GET request for key {}", key);

    let stream = client.store_client.get(&key).await?;

    let file_name = key.rsplit('/').next().unwrap_or(&key);
    let body = Body::from_stream(stream);

    // header management
    let mut headers = HeaderMap::new();
    let content_type = mime_guess::from_path(&key).first_or_octet_stream().to_string();
    headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
    headers.insert(header::CONTENT_DISPOSITION, format!("attachement; filename=\"{}\"", file_name).parse().unwrap());

    Ok((
        StatusCode::OK,
        headers,
        body
    ))
}