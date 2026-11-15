//! Boolean boundary & type coercion fuzzer.

use serde_json::Value;
use specforge_core::models::JSONSchema;

pub struct BooleanMutation {
    pub name: String,
    pub description: String,
    pub value: Value,
    pub is_negative: bool,
}

pub struct BooleanFuzzer;

impl BooleanFuzzer {
    pub fn generate_mutations(_schema: &JSONSchema) -> Vec<BooleanMutation> {
        vec![
            BooleanMutation {
                name: "Type Coercion: String 'true'".to_string(),
                description: "Loose string boolean coercion check: \"true\"".to_string(),
                value: Value::String("true".to_string()),
                is_negative: true,
            },
            BooleanMutation {
                name: "Type Coercion: Integer 1".to_string(),
                description: "Integer representation of true: 1".to_string(),
                value: Value::from(1),
                is_negative: true,
            },
            BooleanMutation {
                name: "Type Violation: Null on Non-Nullable Boolean".to_string(),
                description: "Passing null to non-nullable boolean".to_string(),
                value: Value::Null,
                is_negative: true,
            },
        ]
    }

    pub fn generate_valid_baseline(schema: &JSONSchema) -> Value {
        if let Some(ref ex) = schema.example {
            return ex.clone();
        }
        if let Some(ref def) = schema.default {
            return def.clone();
        }
        Value::Bool(true)
    }
}
