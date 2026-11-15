//! Reverse proxy that transparently forwards API and probe requests from Studio UI to TARGET_URL.

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use std::sync::Arc;
use crate::app::AppState;

pub async fn forward_to_target(
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("");
    let target_url = format!("{}{}", state.target_url.trim_end_matches('/'), path_and_query);

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to build HTTP proxy client: {}", e),
            ).into_response();
        }
    };

    let (parts, body) = req.into_parts();
    let body_bytes = match axum::body::to_bytes(body, 10 * 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("Failed to read request body: {}", e),
            ).into_response();
        }
    };

    let mut forward_req = client.request(method, &target_url);

    // Forward headers from client, omitting Host so reqwest sets target host
    for (name, val) in parts.headers {
        if let Some(header_name) = name {
            if header_name != header::HOST {
                forward_req = forward_req.header(header_name.as_str(), val.as_bytes());
            }
        }
    }

    if !body_bytes.is_empty() {
        forward_req = forward_req.body(body_bytes);
    }

    match forward_req.send().await {
        Ok(resp) => {
            let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::OK);
            let mut response_headers = HeaderMap::new();

            for (k, v) in resp.headers() {
                if let Ok(h_name) = HeaderName::from_bytes(k.as_str().as_bytes()) {
                    if let Ok(h_val) = HeaderValue::from_bytes(v.as_bytes()) {
                        response_headers.insert(h_name, h_val);
                    }
                }
            }

            // Ensure permissive CORS headers for the Studio client
            response_headers.insert(
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                HeaderValue::from_static("*"),
            );
            response_headers.insert(
                header::ACCESS_CONTROL_ALLOW_METHODS,
                HeaderValue::from_static("*"),
            );
            response_headers.insert(
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                HeaderValue::from_static("*"),
            );

            let resp_bytes = resp.bytes().await.unwrap_or_default();
            (status, response_headers, Body::from(resp_bytes)).into_response()
        }
        Err(err) => {
            (
                StatusCode::BAD_GATEWAY,
                format!(
                    "SpecForge Gateway Error: Unable to forward request to target API at '{}'. Error: {}",
                    target_url, err
                ),
            ).into_response()
        }
    }
}
