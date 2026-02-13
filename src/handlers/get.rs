use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::IntoResponse,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::entities::file;
use crate::error::AppError;
use crate::AppState;

fn extract_user_id(headers: &HeaderMap) -> Result<Uuid, AppError> {
    headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or(AppError::BadRequest(
            "Missing or invalid x-user-id header".to_string(),
        ))
}

fn extract_version_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-version-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

fn build_response_headers(file_meta: &file::Model, etag: Option<String>) -> HeaderMap {
    let mut response_headers = HeaderMap::new();

    let mut insert_header = |key, value: String| {
        if let Ok(val) = HeaderValue::from_str(&value) {
            response_headers.insert(key, val);
        }
    };

    insert_header(header::CONTENT_TYPE, file_meta.content_type.clone());
    insert_header(
        header::CONTENT_LENGTH,
        file_meta.content_size.to_string(),
    );
    insert_header(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", file_meta.file_name),
    );
    insert_header(
        HeaderName::from_static("x-version-id"),
        file_meta.s3_version_id.clone(),
    );

    if let Some(etag) = etag {
        insert_header(header::ETAG, etag);
    }

    response_headers
}

pub async fn get_object(
    State(state): State<AppState>,
    Path(key): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    // Extract user ID from headers (assuming some authentication is done)
    let user_id = extract_user_id(&headers)?;

    // Extract version ID from headers if provided
    let version_id = extract_version_id(&headers);

    tracing::info!("GET request for user {}, key {}:{:?}", user_id, key, version_id);

    let mut query = file::Entity::find()
        .filter(file::Column::UserId.eq(user_id))
        .filter(file::Column::FilePath.eq(key.clone()));

    if let Some(ref vid) = version_id {
        query = query.filter(file::Column::S3VersionId.eq(vid));
    } else {
        query = query.filter(file::Column::IsLatest.eq(true));
    }

    let file_meta = query
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    let s3_output = state
        .store_client
        .get(&file_meta.file_key.to_string(), version_id.as_deref())
        .await?;

    // from AWS Stream errors to standard Axum I/O error
    let reader = s3_output.body.into_async_read();
    let stream = ReaderStream::new(reader);
    let body = Body::from_stream(stream);

    let response_headers = build_response_headers(&file_meta, s3_output.e_tag);

    Ok((StatusCode::OK, response_headers, body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn sample_file_model() -> file::Model {
        file::Model {
            id: Uuid::now_v7(),
            file_key: Uuid::now_v7(),
            user_id: Uuid::now_v7(),
            file_name: "hello.txt".to_string(),
            file_path: "docs/hello.txt".to_string(),
            content_type: "text/plain".to_string(),
            content_size: 123,
            s3_version_id: "ver-123".to_string(),
            is_latest: true,
            added_at: chrono::Utc::now().into(),
        }
    }

    #[test]
    fn extract_user_id_missing_is_bad_request() {
        let headers = HeaderMap::new();
        let err = extract_user_id(&headers).unwrap_err();

        match err {
            AppError::BadRequest(msg) => {
                assert!(msg.contains("x-user-id"));
            }
            other => panic!("expected BadRequest, got: {:?}", other),
        }
    }

    #[test]
    fn extract_user_id_invalid_is_bad_request() {
        let mut headers = HeaderMap::new();
        headers.insert("x-user-id", HeaderValue::from_static("not-a-uuid"));

        let err = extract_user_id(&headers).unwrap_err();
        match err {
            AppError::BadRequest(msg) => {
                assert!(msg.contains("invalid"));
            }
            other => panic!("expected BadRequest, got: {:?}", other),
        }
    }

    #[test]
    fn extract_user_id_valid() {
        let mut headers = HeaderMap::new();
        let u = Uuid::now_v7();
        headers.insert("x-user-id", HeaderValue::from_str(&u.to_string()).unwrap());

        let parsed = extract_user_id(&headers).unwrap();
        assert_eq!(parsed, u);
    }

    #[test]
    fn extract_version_id_absent_is_none() {
        let headers = HeaderMap::new();
        assert_eq!(extract_version_id(&headers), None);
    }

    #[test]
    fn extract_version_id_present() {
        let mut headers = HeaderMap::new();
        headers.insert("x-version-id", HeaderValue::from_static("v42"));

        assert_eq!(extract_version_id(&headers), Some("v42".to_string()));
    }

    #[test]
    fn build_response_headers_sets_expected_headers_without_etag() {
        let file_meta = sample_file_model();

        let headers = build_response_headers(&file_meta, None);

        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap(),
            &HeaderValue::from_str(&file_meta.content_type).unwrap()
        );
        assert_eq!(
            headers.get(header::CONTENT_LENGTH).unwrap(),
            &HeaderValue::from_str(&file_meta.content_size.to_string()).unwrap()
        );
        assert_eq!(
            headers.get(header::CONTENT_DISPOSITION).unwrap(),
            &HeaderValue::from_str(&format!(
                "attachment; filename=\"{}\"",
                file_meta.file_name
            ))
            .unwrap()
        );
        assert_eq!(
            headers.get("x-version-id").unwrap(),
            &HeaderValue::from_str(&file_meta.s3_version_id).unwrap()
        );
        assert!(headers.get(header::ETAG).is_none());
    }

    #[test]
    fn build_response_headers_includes_etag_when_present() {
        let file_meta = sample_file_model();

        let headers = build_response_headers(&file_meta, Some("\"etag-value\"".to_string()));

        assert_eq!(
            headers.get(header::ETAG).unwrap(),
            &HeaderValue::from_static("\"etag-value\"")
        );
    }
}