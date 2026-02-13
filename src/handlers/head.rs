use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
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

fn build_head_response_headers(file: &file::Model) -> HeaderMap {
    let mut response_headers = HeaderMap::new();

    response_headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from(file.content_size as u64),
    );

    if let Ok(val) = HeaderValue::from_str(&file.content_type) {
        response_headers.insert(header::CONTENT_TYPE, val);
    }

    let last_modified = file
        .added_at
        .format("%a, %d %b %Y %H:%M:%S GMT")
        .to_string();
    if let Ok(val) = HeaderValue::from_str(&last_modified) {
        response_headers.insert(header::LAST_MODIFIED, val);
    }

    let etag_val = format!("\"{}\"", file.s3_version_id);
    if let Ok(val) = HeaderValue::from_str(&etag_val) {
        response_headers.insert(header::ETAG, val);
    }

    if let Ok(val) = HeaderValue::from_str(&file.s3_version_id) {
        response_headers.insert("x-amz-version-id", val);
    }

    response_headers
}

pub async fn head_object(
    State(state): State<AppState>,
    Path(key): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let user_id: Uuid = extract_user_id(&headers)?;
    let version_id = extract_version_id(&headers);

    tracing::info!(
        "HEAD request for user {} and key {}:{:?}",
        user_id,
        key,
        version_id
    );

    let mut query = file::Entity::find()
        .filter(file::Column::UserId.eq(user_id))
        .filter(file::Column::FilePath.eq(&key));

    // if there is a version_id set, we take this version, otherwise the latest version
    if let Some(v_id) = version_id {
        query = query.filter(file::Column::S3VersionId.eq(v_id));
    } else {
        query = query.filter(file::Column::IsLatest.eq(true));
    }

    let file = query
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("File not found".to_string()))?;

    let response_headers = build_head_response_headers(&file);

    Ok((StatusCode::OK, response_headers))
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
            AppError::BadRequest(msg) => assert!(msg.contains("x-user-id")),
            other => panic!("expected BadRequest, got: {:?}", other),
        }
    }

    #[test]
    fn extract_user_id_invalid_is_bad_request() {
        let mut headers = HeaderMap::new();
        headers.insert("x-user-id", HeaderValue::from_static("not-a-uuid"));

        let err = extract_user_id(&headers).unwrap_err();
        match err {
            AppError::BadRequest(msg) => assert!(msg.contains("invalid")),
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
    fn build_head_response_headers_sets_expected_headers() {
        let file = sample_file_model();
        let headers = build_head_response_headers(&file);

        assert_eq!(
            headers.get(header::CONTENT_LENGTH).unwrap(),
            &HeaderValue::from(file.content_size as u64)
        );
        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap(),
            &HeaderValue::from_str(&file.content_type).unwrap()
        );
        assert_eq!(
            headers.get("x-amz-version-id").unwrap(),
            &HeaderValue::from_str(&file.s3_version_id).unwrap()
        );
        assert_eq!(
            headers.get(header::ETAG).unwrap(),
            &HeaderValue::from_str(&format!("\"{}\"", file.s3_version_id)).unwrap()
        );

        // Sanity check that it at least exists and is parseable as a header value.
        let lm = headers.get(header::LAST_MODIFIED).unwrap();
        assert!(!lm.as_bytes().is_empty());
    }
}