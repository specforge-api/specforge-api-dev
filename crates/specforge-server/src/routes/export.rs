//! Test suite code export endpoint for Jest/Vitest (TypeScript) and Pytest (Python).

use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use std::sync::Arc;
use specforge_exporters::{JestExporter, PytestExporter};
use specforge_fuzzer::planner::TestPlanner;
use crate::app::AppState;

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>, // "jest", "vitest", or "pytest"
}

pub async fn export_test_suite(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Response {
    let spec = state.spec.read().await;
    let mut all_test_cases = Vec::new();

    for op in &spec.operations {
        let plan = TestPlanner::plan_operation_tests(op);
        all_test_cases.extend(plan.test_cases);
    }

    let format_str = params.format.unwrap_or_else(|| state.export_target.clone());
    let target_base = &state.target_url;

    match format_str.to_lowercase().as_str() {
        "jest" | "vitest" => {
            let code = JestExporter::export(&all_test_cases, target_base);
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "text/typescript; charset=utf-8"),
                    (header::CONTENT_DISPOSITION, "attachment; filename=\"api_contracts.test.ts\""),
                ],
                code,
            ).into_response()
        }
        _ => {
            // Default to Pytest
            let code = PytestExporter::export(&all_test_cases, target_base);
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "text/x-python; charset=utf-8"),
                    (header::CONTENT_DISPOSITION, "attachment; filename=\"test_api_contracts.py\""),
                ],
                code,
            ).into_response()
        }
    }
}
