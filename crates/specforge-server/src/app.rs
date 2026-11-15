//! Axum application builder and state container.

use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

use specforge_core::models::ApiSpec;
use crate::routes::{docs, export, mock, plan, run};

pub struct AppState {
    pub spec: RwLock<ApiSpec>,
    pub target_url: String,
    pub export_target: String, // "jest" or "pytest"
    pub theme: String,
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
        // Test Planning & Execution APIs
        .route("/specforge/api/plan", get(plan::get_test_plans))
        .route("/specforge/api/run-tests", post(run::run_all_tests))
        .route("/specforge/api/export", get(export::export_test_suite))
        // Dynamic Mock Server Routes
        .route("/mock", get(mock::handle_mock_root).post(mock::handle_mock_root))
        .route("/mock/*path", get(mock::handle_mock_request).post(mock::handle_mock_request))
        .layer(cors)
        .with_state(state)
}

async fn serve_raw_spec(axum::extract::State(state): axum::extract::State<Arc<AppState>>) -> impl axum::response::IntoResponse {
    let spec = state.spec.read().await;
    axum::response::Json(spec.raw_spec.clone())
}

