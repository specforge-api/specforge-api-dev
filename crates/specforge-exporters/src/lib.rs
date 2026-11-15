//! SpecForge Polyglot Test Exporters: Jest / TypeScript and Pytest / Python.

pub mod jest;
pub mod pytest;

pub use jest::JestExporter;
pub use pytest::PytestExporter;

#[cfg(test)]
mod tests {
    use super::*;
    use specforge_fuzzer::invariants::TestCase;

    #[test]
    fn test_jest_exporter_generates_supertest_calls() {
        let tc = TestCase {
            id: "abc123".to_string(),
            endpoint: "/api/users".to_string(),
            method: "post".to_string(),
            field_name: "email".to_string(),
            mutation_category: "StringBoundary".to_string(),
            mutation_name: "Format: Invalid Email".to_string(),
            description: "Malformed email probe".to_string(),
            payload: Some(serde_json::json!({"email": "bad_email"})),
            query_params: None,
            headers: None,
            expected_status: 422,
            is_negative_test: true,
        };

        let jest_code = JestExporter::export(&[tc], "http://localhost:3000");
        assert!(jest_code.contains("import request from 'supertest';"));
        assert!(jest_code.contains("expect(res.status).toBeLessThan(500)"));
        assert!(jest_code.contains("expect([400, 422]).toContain(res.status)"));
    }

    #[test]
    fn test_pytest_exporter_generates_httpx_calls() {
        let tc = TestCase {
            id: "xyz789".to_string(),
            endpoint: "/api/users".to_string(),
            method: "post".to_string(),
            field_name: "email".to_string(),
            mutation_category: "StringBoundary".to_string(),
            mutation_name: "Format: Invalid Email".to_string(),
            description: "Malformed email probe".to_string(),
            payload: Some(serde_json::json!({"email": "bad_email"})),
            query_params: None,
            headers: None,
            expected_status: 422,
            is_negative_test: true,
        };

        let pytest_code = PytestExporter::export(&[tc], "http://localhost:8000");
        assert!(pytest_code.contains("import httpx"));
        assert!(pytest_code.contains("assert response.status_code != 500"));
        assert!(pytest_code.contains("assert response.status_code in (400, 422)"));
    }
}
