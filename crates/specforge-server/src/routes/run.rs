//! Test execution dispatcher hitting target backend and evaluating invariants.

use axum::extract::State;
use axum::response::{IntoResponse, Json};
use std::sync::Arc;
use std::time::Instant;
use specforge_fuzzer::invariants::{evaluate_test_result, TestCase, TestResult, TestStatus};
use specforge_fuzzer::planner::TestPlanner;
use crate::app::AppState;

pub async fn run_all_tests(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let spec = state.spec.read().await;
    let target_base = state.target_url.clone();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_default();

    let mut all_test_cases = Vec::new();
    for op in &spec.operations {
        let plan = TestPlanner::plan_operation_tests(op);
        all_test_cases.extend(plan.test_cases);
    }

    let mut results = Vec::new();
    let mut pass_count = 0;
    let mut crash_count = 0;
    let mut leak_count = 0;

    for tc in all_test_cases {
        let res = execute_single_test(&client, &target_base, &tc).await;
        match res.status {
            TestStatus::Pass => pass_count += 1,
            TestStatus::Crash500 => crash_count += 1,
            TestStatus::WarnLeak => leak_count += 1,
            _ => {}
        }
        results.push(res);
    }

    Json(serde_json::json!({
        "total_executed": results.len(),
        "passed": pass_count,
        "crashes_500": crash_count,
        "schema_leaks": leak_count,
        "results": results,
    }))
}

async fn execute_single_test(
    client: &reqwest::Client,
    base_url: &str,
    tc: &TestCase,
) -> TestResult {
    let url = format!("{}{}", base_url.trim_end_matches('/'), tc.endpoint);
    let start = Instant::now();

    let method = match tc.method.to_uppercase().as_str() {
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        "PATCH" => reqwest::Method::PATCH,
        _ => reqwest::Method::GET,
    };

    let mut req = client.request(method, &url);
    if let Some(ref payload) = tc.payload {
        req = req.json(payload);
    }

    let resp_attempt = req.send().await;
    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

    match resp_attempt {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            evaluate_test_result(tc, status, &text, latency_ms)
        }
        Err(e) => {
            evaluate_test_result(tc, 599, &e.to_string(), latency_ms)
        }
    }
}
