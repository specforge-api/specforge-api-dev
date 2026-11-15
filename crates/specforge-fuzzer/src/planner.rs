//! Test Plan orchestrator that coordinates combinatorial boundary fuzzers.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use specforge_core::models::Operation;
use crate::invariants::TestCase;
use crate::object_fuzzer::ObjectFuzzer;
use crate::string_fuzzer::StringFuzzer;
use crate::numeric_fuzzer::NumericFuzzer;
use crate::boolean_fuzzer::BooleanFuzzer;
use crate::array_fuzzer::ArrayFuzzer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPlan {
    pub operation_key: String,
    pub endpoint: String,
    pub method: String,
    pub total_tests: usize,
    pub test_cases: Vec<TestCase>,
}

pub struct TestPlanner;

impl TestPlanner {
    pub fn plan_operation_tests(operation: &Operation) -> TestPlan {
        let mut test_cases = Vec::new();
        let endpoint = operation.path.clone();
        let method = operation.method.to_lowercase();

        // 1. Request Body Fuzzing
        if let Some(ref req_body) = operation.request_body {
            if let Some(ref schema) = req_body.schema {
                let baseline = ObjectFuzzer::generate_valid_baseline(schema);

                // 1.0 Baseline Happy Path
                test_cases.push(TestCase {
                    id: Uuid::new_v4().to_string()[..8].to_string(),
                    endpoint: endpoint.clone(),
                    method: method.clone(),
                    field_name: "[body_root]".to_string(),
                    mutation_category: "Baseline".to_string(),
                    mutation_name: "Happy Path: Valid Baseline Payload".to_string(),
                    description: "Verify endpoint responds successfully to schema-compliant payload".to_string(),
                    payload: Some(baseline.clone()),
                    query_params: None,
                    headers: None,
                    expected_status: 200,
                    is_negative_test: false,
                });

                // 1.1 Object-Level Mutations (missing required, extra properties, empty body)
                let obj_mutations = ObjectFuzzer::generate_object_mutations(schema, &baseline);
                for m in obj_mutations {
                    test_cases.push(TestCase {
                        id: Uuid::new_v4().to_string()[..8].to_string(),
                        endpoint: endpoint.clone(),
                        method: method.clone(),
                        field_name: "[body_root]".to_string(),
                        mutation_category: format!("Object:{}", m.category),
                        mutation_name: m.name,
                        description: m.description,
                        payload: Some(m.payload),
                        query_params: None,
                        headers: None,
                        expected_status: 422,
                        is_negative_test: true,
                    });
                }

                // 1.2 Property-Level Boundary Fuzzing
                if let Value::Object(ref base_map) = baseline {
                    for (prop_name, prop_schema) in &schema.properties {
                        // String properties
                        if prop_schema.is_string() {
                            let str_muts = StringFuzzer::generate_mutations(prop_schema);
                            for sm in str_muts {
                                let mut field_payload = base_map.clone();
                                field_payload.insert(prop_name.clone(), sm.value);
                                test_cases.push(TestCase {
                                    id: Uuid::new_v4().to_string()[..8].to_string(),
                                    endpoint: endpoint.clone(),
                                    method: method.clone(),
                                    field_name: prop_name.clone(),
                                    mutation_category: "StringBoundary".to_string(),
                                    mutation_name: sm.name,
                                    description: sm.description,
                                    payload: Some(Value::Object(field_payload)),
                                    query_params: None,
                                    headers: None,
                                    expected_status: if sm.is_negative { 422 } else { 200 },
                                    is_negative_test: sm.is_negative,
                                });
                            }
                        }

                        // Numeric properties
                        if prop_schema.is_number() {
                            let num_muts = NumericFuzzer::generate_mutations(prop_schema);
                            for nm in num_muts {
                                let mut field_payload = base_map.clone();
                                field_payload.insert(prop_name.clone(), nm.value);
                                test_cases.push(TestCase {
                                    id: Uuid::new_v4().to_string()[..8].to_string(),
                                    endpoint: endpoint.clone(),
                                    method: method.clone(),
                                    field_name: prop_name.clone(),
                                    mutation_category: "NumericBoundary".to_string(),
                                    mutation_name: nm.name,
                                    description: nm.description,
                                    payload: Some(Value::Object(field_payload)),
                                    query_params: None,
                                    headers: None,
                                    expected_status: if nm.is_negative { 422 } else { 200 },
                                    is_negative_test: nm.is_negative,
                                });
                            }
                        }

                        // Boolean properties
                        if prop_schema.is_boolean() {
                            let bool_muts = BooleanFuzzer::generate_mutations(prop_schema);
                            for bm in bool_muts {
                                let mut field_payload = base_map.clone();
                                field_payload.insert(prop_name.clone(), bm.value);
                                test_cases.push(TestCase {
                                    id: Uuid::new_v4().to_string()[..8].to_string(),
                                    endpoint: endpoint.clone(),
                                    method: method.clone(),
                                    field_name: prop_name.clone(),
                                    mutation_category: "BooleanBoundary".to_string(),
                                    mutation_name: bm.name,
                                    description: bm.description,
                                    payload: Some(Value::Object(field_payload)),
                                    query_params: None,
                                    headers: None,
                                    expected_status: if bm.is_negative { 422 } else { 200 },
                                    is_negative_test: bm.is_negative,
                                });
                            }
                        }

                        // Array properties
                        if prop_schema.is_array() {
                            let arr_muts = ArrayFuzzer::generate_mutations(prop_schema);
                            for am in arr_muts {
                                let mut field_payload = base_map.clone();
                                field_payload.insert(prop_name.clone(), am.value);
                                test_cases.push(TestCase {
                                    id: Uuid::new_v4().to_string()[..8].to_string(),
                                    endpoint: endpoint.clone(),
                                    method: method.clone(),
                                    field_name: prop_name.clone(),
                                    mutation_category: "ArrayBoundary".to_string(),
                                    mutation_name: am.name,
                                    description: am.description,
                                    payload: Some(Value::Object(field_payload)),
                                    query_params: None,
                                    headers: None,
                                    expected_status: if am.is_negative { 422 } else { 200 },
                                    is_negative_test: am.is_negative,
                                });
                            }
                        }
                    }
                }
            }
        }

        // 2. Query / Path Parameters Fuzzing (e.g. GET /users?limit=...)
        for param in &operation.parameters {
            if let Some(ref schema) = param.schema {
                if schema.is_number() {
                    let num_muts = NumericFuzzer::generate_mutations(schema);
                    for nm in num_muts {
                        let mut q_map = serde_json::Map::new();
                        q_map.insert(param.name.clone(), nm.value);
                        test_cases.push(TestCase {
                            id: Uuid::new_v4().to_string()[..8].to_string(),
                            endpoint: endpoint.clone(),
                            method: method.clone(),
                            field_name: param.name.clone(),
                            mutation_category: "QueryParamBoundary".to_string(),
                            mutation_name: nm.name,
                            description: nm.description,
                            payload: None,
                            query_params: Some(Value::Object(q_map)),
                            headers: None,
                            expected_status: if nm.is_negative { 422 } else { 200 },
                            is_negative_test: nm.is_negative,
                        });
                    }
                }
            }
        }

        let total_tests = test_cases.len();
        TestPlan {
            operation_key: operation.key.clone(),
            endpoint,
            method,
            total_tests,
            test_cases,
        }
    }
}
