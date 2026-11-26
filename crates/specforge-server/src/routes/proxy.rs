//! Reverse proxy that transparently forwards API and probe requests from Studio UI to TARGET_URL.

use axum::body::{Body, Bytes};
use axum::extract::{Request, State};
use axum::http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::Arc;
use crate::app::AppState;

const MAX_RELAY_BODY_BYTES: usize = 10 * 1024 * 1024;

#[derive(Debug, Deserialize)]
pub struct RelayRequestPayload {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub body: Option<String>,
}

fn is_hop_by_hop_or_managed(name_lower: &str) -> bool {
    matches!(
        name_lower,
        "host" | "connection" | "keep-alive" | "transfer-encoding" | "te" | "upgrade" | "content-length"
    )
}

async fn execute_target_forward(
    target_url: &str,
    method: Method,
    header_pairs: Vec<(String, Vec<u8>)>,
    body_bytes: Bytes,
) -> Response {
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

    let mut forward_req = client.request(method, target_url);

    for (name, val) in header_pairs {
        if !is_hop_by_hop_or_managed(&name.to_ascii_lowercase()) {
            forward_req = forward_req.header(name.as_str(), val);
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

pub async fn forward_to_target(
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("");
    let target_url = format!("{}{}", state.target_url.trim_end_matches('/'), path_and_query);

    let (parts, body) = req.into_parts();
    let body_bytes = match axum::body::to_bytes(body, MAX_RELAY_BODY_BYTES).await {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("Failed to read request body: {}", e),
            ).into_response();
        }
    };

    let header_pairs = parts
        .headers
        .into_iter()
        .filter_map(|(name, val)| name.map(|n| (n.as_str().to_string(), val.as_bytes().to_vec())))
        .collect();

    execute_target_forward(&target_url, method, header_pairs, body_bytes).await
}

/// Relays browser requests whose HTTP method/body combination is blocked by WHATWG Fetch (e.g. GET with JSON body).
pub async fn relay_request(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RelayRequestPayload>,
) -> Response {
    let method_str = payload.method.trim().to_ascii_uppercase();
    let method = match Method::from_bytes(method_str.as_bytes()) {
        Ok(m) => m,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid HTTP method").into_response();
        }
    };

    // Require relative path starting with single '/' to prevent target host override
    if !payload.path.starts_with('/') || payload.path.starts_with("//") {
        return (StatusCode::BAD_REQUEST, "Relay path must be a relative path starting with '/'").into_response();
    }

    let body_bytes = match payload.body {
        Some(b) if b.len() <= MAX_RELAY_BODY_BYTES => Bytes::from(b),
        Some(_) => {
            return (StatusCode::PAYLOAD_TOO_LARGE, "Relay body exceeds 10 MiB limit").into_response();
        }
        None => Bytes::new(),
    };

    let target_url = format!("{}{}", state.target_url.trim_end_matches('/'), payload.path);
    let header_pairs = payload
        .headers
        .into_iter()
        .map(|(k, v)| (k, v.into_bytes()))
        .collect();

    execute_target_forward(&target_url, method, header_pairs, body_bytes).await
}
