//! SpecForge Fuzzer: Combinatorial boundary mutations & Invariant testing.

pub mod invariants;
pub mod string_fuzzer;
pub mod numeric_fuzzer;
pub mod boolean_fuzzer;
pub mod array_fuzzer;
pub mod object_fuzzer;
pub mod planner;

pub use invariants::{evaluate_test_result, TestCase, TestResult, TestStatus};
pub use planner::{TestPlan, TestPlanner};

#[cfg(test)]
mod tests {
    use super::*;
    use specforge_core::models::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_planner_generates_combinatorial_cases() {
        let mut props = BTreeMap::new();
        props.insert("username".to_string(), JSONSchema {
            schema_type: Some("string".to_string()),
            min_length: Some(3),
            max_length: Some(20),
            ..Default::default()
        });
        props.insert("age".to_string(), JSONSchema {
            schema_type: Some("integer".to_string()),
            minimum: Some(18.0),
            maximum: Some(120.0),
            ..Default::default()
        });

        let body_schema = JSONSchema {
            schema_type: Some("object".to_string()),
            properties: props,
            required: vec!["username".to_string()],
            ..Default::default()
        };

        let op = Operation {
            key: "POST /users".to_string(),
            method: "POST".to_string(),
            path: "/users".to_string(),
            operation_id: Some("createUser".to_string()),
            summary: Some("Create user".to_string()),
            description: None,
            tags: vec![],
            parameters: vec![],
            request_body: Some(RequestBody {
                description: None,
                required: true,
                content_type: "application/json".to_string(),
                schema: Some(body_schema),
            }),
            responses: BTreeMap::new(),
        };

        let plan = TestPlanner::plan_operation_tests(&op);
        assert!(plan.total_tests > 10, "Should synthesize at least 10 boundary tests");
        
        // Assert we have whitespace and negative checks
        let has_whitespace = plan.test_cases.iter().any(|tc| tc.mutation_name.contains("Whitespace"));
        assert!(has_whitespace, "Must contain whitespace test case");

        let has_below_min = plan.test_cases.iter().any(|tc| tc.mutation_name.contains("Below Minimum"));
        assert!(has_below_min, "Must contain below minimum test case");
    }

    #[test]
    fn test_invariant_evaluator_catches_500() {
        let tc = TestCase {
            id: "test1".to_string(),
            endpoint: "/users".to_string(),
            method: "post".to_string(),
            field_name: "username".to_string(),
            mutation_category: "StringBoundary".to_string(),
            mutation_name: "Whitespace".to_string(),
            description: "Whitespace string".to_string(),
            payload: None,
            query_params: None,
            headers: None,
            expected_status: 422,
            is_negative_test: true,
        };

        let result = evaluate_test_result(&tc, 500, "Internal Server Error", 15.0);
        assert_eq!(result.status, TestStatus::Crash500);
        assert!(result.message.contains("CRASH DETECTED"));
    }
}

