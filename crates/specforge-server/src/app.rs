//! Axum application builder and state container.

use axum::routing::{get, post};
use axum::Router;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

use specforge_core::models::ApiSpec;
use crate::routes::{docs, export, mock, plan, proxy, run};

pub struct AppState {
    pub spec: RwLock<ApiSpec>,
    pub spec_source: String,
    pub target_url: String,
    pub export_target: String, // "jest" or "pytest"
    pub theme: String,
    pub service_name: Option<String>,
    pub version: AtomicU64,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Studio Documentation UI
        .route("/", get(docs::serve_studio_docs))
        .route("/docs", get(docs::serve_studio_docs))
        // Raw Spec JSON
        .route("/openapi.json", get(serve_raw_spec))
        // Live Spec Status & Version check for Hot Reload
        .route("/specforge/api/status", get(serve_spec_status))
        // Test Planning & Execution APIs
        .route("/specforge/api/plan", get(plan::get_test_plans))
        .route("/specforge/api/run-tests", post(run::run_all_tests))
        .route("/specforge/api/export", get(export::export_test_suite))
        // Dynamic Mock Server Routes
        .route("/mock", get(mock::handle_mock_root).post(mock::handle_mock_root))
        .route("/mock/*path", get(mock::handle_mock_request).post(mock::handle_mock_request))
        // Transparent Gateway Proxy: forwards all API calls (/api/...) directly to TARGET_URL
        .fallback(proxy::forward_to_target)
        .layer(cors)
        .with_state(state)
}

async fn serve_raw_spec(axum::extract::State(state): axum::extract::State<Arc<AppState>>) -> impl axum::response::IntoResponse {
    let spec = state.spec.read().await;
    axum::response::Json(spec.raw_spec.clone())
}

async fn serve_spec_status(axum::extract::State(state): axum::extract::State<Arc<AppState>>) -> impl axum::response::IntoResponse {
    let spec = state.spec.read().await;
    let version = state.version.load(Ordering::SeqCst);
    axum::response::Json(serde_json::json!({
        "version": version,
        "title": spec.title,
        "total_operations": spec.operations.len(),
        "target_url": state.target_url,
        "spec_source": state.spec_source,
    }))
}

