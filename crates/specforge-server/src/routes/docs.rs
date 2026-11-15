//! Serves the interactive SpecForge Studio Web UI with injected runtime configuration.

use axum::extract::State;
use axum::response::{Html, IntoResponse, Response};
use axum::http::{header, HeaderValue, StatusCode};
use std::sync::Arc;
use crate::app::AppState;
use crate::embedded::Asset;

pub async fn serve_studio_docs(State(state): State<Arc<AppState>>) -> Response {
    let raw_html = match Asset::get_index_html() {
        Some(html) => html,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "SpecForge Studio UI asset was not bundled into the binary",
            ).into_response();
        }
    };

    let spec_guard = state.spec.read().await;
    let config_data = serde_json::json!({
        "title": format!("{} | SpecForge Studio", spec_guard.title),
        "path": "/docs",
        "spec_url": "/openapi.json",
        "theme": state.theme,
        "enable_tests": true,
        "embedded_spec": spec_guard.raw_spec,
    });

    let config_json = serde_json::to_string(&config_data).unwrap_or_default()
        .replace("<", "\\u003c")
        .replace(">", "\\u003e")
        .replace("&", "\\u0026");

    let injection = format!("<script>window.__SPECFORGE_CONFIG__ = {};</script>", config_json);

    let rendered = if raw_html.contains("<!-- __SPECFORGE_CONFIG_INJECTION__ -->") {
        raw_html.replace("<!-- __SPECFORGE_CONFIG_INJECTION__ -->", &injection)
    } else if raw_html.contains("</head>") {
        raw_html.replace("</head>", &format!("{}\n</head>", injection))
    } else {
        format!("{}\n{}", injection, raw_html)
    };

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8")),
            (header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")),
            (header::X_FRAME_OPTIONS, HeaderValue::from_static("SAMEORIGIN")),
        ],
        Html(rendered),
    ).into_response()
}

