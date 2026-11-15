//! Dynamic mock router serving schema-compliant responses.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use std::sync::Arc;
use specforge_mock::MockGenerator;
use crate::app::AppState;

pub async fn handle_mock_root(State(state): State<Arc<AppState>>) -> Response {
    handle_mock_request(State(state), Path(String::new())).await
}

pub async fn handle_mock_request(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Response {
    let full_path = format!("/{}", path);
    let spec = state.spec.read().await;

    // Find operation matching the path
    let matched_op = spec.operations.iter().find(|op| op.path == full_path || op.path.ends_with(&path));

    if let Some(op) = matched_op {
        // Look for 200 or 201 response schema
        let response_schema = op.responses.get("200")
            .or_else(|| op.responses.get("201"))
            .and_then(|r| r.schema.as_ref());

        if let Some(schema) = response_schema {
            let mock_body = MockGenerator::generate_response(schema);
            return (StatusCode::OK, Json(mock_body)).into_response();
        }

        // Default mock success object
        let default_body = serde_json::json!({
            "status": "mock_success",
            "operation": op.key,
            "message": "Dynamic mock response synthesized by SpecForge Dev"
        });
        (StatusCode::OK, Json(default_body)).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Endpoint not found in schema",
                "path": full_path
            })),
        ).into_response()
    }
}

