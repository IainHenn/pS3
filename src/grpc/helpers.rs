use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::Value;
use tonic::Status;
use uuid::Uuid;

pub fn parse_uuid(value: &str, field: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(value).map_err(|_| Status::invalid_argument(format!("invalid {field}")))
}

pub fn parse_uuid_list(csv: &str, field: &str) -> Result<Vec<Uuid>, Status> {
    if csv.trim().is_empty() {
        return Ok(Vec::new());
    }

    csv.split(',')
        .map(|id| parse_uuid(id.trim(), field))
        .collect()
}

pub async fn response_to_json(response: Response<Body>) -> Result<(StatusCode, Value), Status> {
    let (parts, body) = response.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| Status::internal("failed to read response body"))?;

    if body_bytes.is_empty() {
        return Ok((parts.status, Value::Null));
    }

    let is_json = parts
        .headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.contains("application/json"))
        .unwrap_or(false)
        || matches!(body_bytes.first(), Some(b'{') | Some(b'['));

    if is_json {
        serde_json::from_slice(&body_bytes)
            .map(|json| (parts.status, json))
            .map_err(|_| Status::internal("failed to parse response body"))
    } else {
        Ok((parts.status, Value::Null))
    }
}

pub async fn service_json_response<R>(future: R) -> Result<(StatusCode, Value), Status>
where
    R: std::future::Future,
    R::Output: IntoResponse,
{
    response_to_json(future.await.into_response()).await
}

pub fn json_str(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|entry| entry.as_str())
        .unwrap_or_default()
        .to_string()
}
