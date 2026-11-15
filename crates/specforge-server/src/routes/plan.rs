//! Endpoint returning synthesized boundary test plans for all operations.

use axum::extract::State;
use axum::response::{IntoResponse, Json};
use std::sync::Arc;
use specforge_fuzzer::planner::TestPlanner;
use crate::app::AppState;

pub async fn get_test_plans(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let spec = state.spec.read().await;
    let mut plans = Vec::new();

    for op in &spec.operations {
        let plan = TestPlanner::plan_operation_tests(op);
        plans.push(plan);
    }

    Json(serde_json::json!({
        "title": spec.title,
        "version": spec.version,
        "total_operations": spec.operations.len(),
        "plans": plans,
    }))
}

