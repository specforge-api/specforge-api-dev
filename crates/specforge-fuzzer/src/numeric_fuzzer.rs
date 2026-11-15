//! Schema-aware numeric fuzzer (min/max boundaries, negative on unsigned, overflow, float pollution).

use serde_json::Value;
use specforge_core::models::JSONSchema;

pub struct NumericMutation {
    pub name: String,
    pub description: String,
    pub value: Value,
    pub is_negative: bool,
}

pub struct NumericFuzzer;

impl NumericFuzzer {
    pub fn generate_mutations(schema: &JSONSchema) -> Vec<NumericMutation> {
        let mut mutations = Vec::new();
        let is_int = schema.is_integer();

        // 1. Min Boundary Violations
        if let Some(min) = schema.minimum {
            let under = min - 1.0;
            mutations.push(NumericMutation {
                name: format!("Boundary: Below Minimum (min - 1 = {})", under),
                description: format!("Value {} violates minimum constraint of {}", under, min),
                value: if is_int { Value::from(under as i64) } else { serde_json::json!(under) },
                is_negative: true,
            });
        }

        // 2. Max Boundary Violations
        if let Some(max) = schema.maximum {
            let over = max + 1.0;
            mutations.push(NumericMutation {
                name: format!("Boundary: Exceed Maximum (max + 1 = {})", over),
                description: format!("Value {} violates maximum constraint of {}", over, max),
                value: if is_int { Value::from(over as i64) } else { serde_json::json!(over) },
                is_negative: true,
            });
        }

        // 3. Negative Value Check
        let min_val = schema.minimum.unwrap_or(-1000.0);
        if min_val >= 0.0 {
            mutations.push(NumericMutation {
                name: "Boundary: Negative on Non-Negative Field (-1)".to_string(),
                description: "Supplying -1 to a field declared with minimum >= 0".to_string(),
                value: Value::from(-1),
                is_negative: true,
            });
        }

        // 4. Float-into-Integer Pollution
        if is_int {
            mutations.push(NumericMutation {
                name: "Type Pollution: Floating-point into Integer (42.5)".to_string(),
                description: "Passing float 42.5 to an integer field".to_string(),
                value: serde_json::json!(42.5),
                is_negative: true,
            });
        }

        // 5. 32-bit Integer Overflow
        mutations.push(NumericMutation {
            name: "Overflow: 32-bit Signed Integer Limit (2147483648)".to_string(),
            description: "Supplying INT32_MAX + 1 to test buffer overflow".to_string(),
            value: Value::from(2147483648i64),
            is_negative: false, // Resilience check: only fails on 500
        });

        mutations
    }

    pub fn generate_valid_baseline(schema: &JSONSchema) -> Value {
        if let Some(ref ex) = schema.example {
            return ex.clone();
        }
        if let Some(ref def) = schema.default {
            return def.clone();
        }
        if let Some(ref enums) = schema.enum_values {
            if let Some(first) = enums.first() {
                return first.clone();
            }
        }

        let is_int = schema.is_integer();
        let min = schema.minimum.unwrap_or(1.0);
        let max = schema.maximum.unwrap_or(100.0);
        let valid_num = (min + max) / 2.0;

        if is_int {
            Value::from(valid_num.round() as i64)
        } else {
            serde_json::json!(valid_num)
        }
    }
}
