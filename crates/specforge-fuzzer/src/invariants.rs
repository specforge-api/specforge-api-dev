//! Invariants and evaluation engine ("Zero 500s" rule, Clean 400/422, Warn on 200 Schema Leak).

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TestStatus {
    Pass,
    Crash500,
    WarnLeak,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub id: String,
    pub endpoint: String,
    pub method: String,
    pub field_name: String,
    pub mutation_category: String,
    pub mutation_name: String,
    pub description: String,
    pub payload: Option<Value>,
    #[serde(default)]
    pub query_params: Option<Value>,
    #[serde(default)]
    pub headers: Option<Value>,
    pub expected_status: u16,
    pub is_negative_test: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_case_id: String,
    pub status: TestStatus,
    pub actual_status: u16,
    pub message: String,
    pub curl_command: String,
    pub latency_ms: f64,
}

/// Evaluates test outcome against SpecForge Invariants
pub fn evaluate_test_result(
    test_case: &TestCase,
    actual_status: u16,
    _response_body: &str,
    latency_ms: f64,
) -> TestResult {
    let curl = build_curl_command(test_case);

    // INVARIANT 1: Zero 500s (Highest Priority)
    if actual_status >= 500 {
        return TestResult {
            test_case_id: test_case.id.clone(),
            status: TestStatus::Crash500,
            actual_status,
            message: format!(
                "CRASH DETECTED: Endpoint returned unhandled {} Internal Server Error on invalid input: '{}'",
                actual_status, test_case.mutation_name
            ),
            curl_command: curl,
            latency_ms,
        };
    }

    // Happy-path verification
    if !test_case.is_negative_test {
        if actual_status >= 200 && actual_status < 300 {
            return TestResult {
                test_case_id: test_case.id.clone(),
                status: TestStatus::Pass,
                actual_status,
                message: "Baseline valid request succeeded as expected.".to_string(),
                curl_command: curl,
                latency_ms,
            };
        } else {
            return TestResult {
                test_case_id: test_case.id.clone(),
                status: TestStatus::Error,
                actual_status,
                message: format!("Baseline valid request failed with HTTP {}", actual_status),
                curl_command: curl,
                latency_ms,
            };
        }
    }

    // INVARIANT 2: Negative Test Evaluation
    if actual_status == 400 || actual_status == 422 {
        TestResult {
            test_case_id: test_case.id.clone(),
            status: TestStatus::Pass,
            actual_status,
            message: format!("Handled cleanly with HTTP {}", actual_status),
            curl_command: curl,
            latency_ms,
        }
    } else if actual_status >= 200 && actual_status < 300 {
        // Schema leak: malformed input was accepted!
        TestResult {
            test_case_id: test_case.id.clone(),
            status: TestStatus::WarnLeak,
            actual_status,
            message: format!(
                "SCHEMA LEAK: Server erroneously accepted invalid input with HTTP {} (expected 400 or 422)",
                actual_status
            ),
            curl_command: curl,
            latency_ms,
        }
    } else {
        TestResult {
            test_case_id: test_case.id.clone(),
            status: TestStatus::Pass,
            actual_status,
            message: format!("Rejected with status {}", actual_status),
            curl_command: curl,
            latency_ms,
        }
    }
}

pub fn build_curl_command(test_case: &TestCase) -> String {
    let mut parts = vec![format!("curl -X {}", test_case.method.to_uppercase())];
    
    if let Some(ref payload) = test_case.payload {
        let json_str = serde_json::to_string(payload).unwrap_or_default();
        let escaped = json_str.replace('"', "\\\"");
        parts.push("-H \"Content-Type: application/json\"".to_string());
        parts.push(format!("-d \"{}\"", escaped));
    }

    parts.push(format!("\"http://localhost:8000{}\"", test_case.endpoint));
    parts.join(" ")
}
