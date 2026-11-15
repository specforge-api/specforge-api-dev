//! Array boundary and constraint fuzzer (empty array, minItems, maxItems, uniqueItems).

use serde_json::Value;
use specforge_core::models::JSONSchema;
use crate::string_fuzzer::StringFuzzer;

pub struct ArrayMutation {
    pub name: String,
    pub description: String,
    pub value: Value,
    pub is_negative: bool,
}

pub struct ArrayFuzzer;

impl ArrayFuzzer {
    pub fn generate_mutations(schema: &JSONSchema) -> Vec<ArrayMutation> {
        let mut mutations = Vec::new();

        // 1. Empty Array
        let min_items = schema.min_items.unwrap_or(0);
        mutations.push(ArrayMutation {
            name: "Boundary: Empty Array ([])".to_string(),
            description: "Supplying empty array []".to_string(),
            value: Value::Array(Vec::new()),
            is_negative: min_items > 0,
        });

        // 2. Under minItems
        if min_items > 1 {
            let mut under_arr = Vec::new();
            for _ in 0..(min_items - 1) {
                under_arr.push(Self::generate_item_baseline(schema.items.as_deref()));
            }
            mutations.push(ArrayMutation {
                name: format!("Boundary: Below minItems ({})", min_items - 1),
                description: format!("Supplied {} items when minItems is {}", min_items - 1, min_items),
                value: Value::Array(under_arr),
                is_negative: true,
            });
        }

        // 3. Exceed maxItems
        if let Some(max) = schema.max_items {
            let mut over_arr = Vec::new();
            for _ in 0..=(max + 1) {
                over_arr.push(Self::generate_item_baseline(schema.items.as_deref()));
            }
            mutations.push(ArrayMutation {
                name: format!("Boundary: Exceed maxItems ({})", max + 1),
                description: format!("Supplied {} items when maxItems is {}", max + 1, max),
                value: Value::Array(over_arr),
                is_negative: true,
            });
        }

        // 4. Duplicate Items on uniqueItems = true
        if schema.unique_items == Some(true) {
            let duplicate = Self::generate_item_baseline(schema.items.as_deref());
            mutations.push(ArrayMutation {
                name: "Constraint Violation: Duplicate Items (uniqueItems)".to_string(),
                description: "Array containing identical elements".to_string(),
                value: Value::Array(vec![duplicate.clone(), duplicate]),
                is_negative: true,
            });
        }

        mutations
    }

    pub fn generate_valid_baseline(schema: &JSONSchema) -> Value {
        if let Some(ref ex) = schema.example {
            return ex.clone();
        }
        if let Some(ref def) = schema.default {
            return def.clone();
        }

        let item = Self::generate_item_baseline(schema.items.as_deref());
        let count = schema.min_items.unwrap_or(1).max(1);
        Value::Array(vec![item; count])
    }

    fn generate_item_baseline(item_schema: Option<&JSONSchema>) -> Value {
        if let Some(s) = item_schema {
            if s.is_string() {
                StringFuzzer::generate_valid_baseline(s)
            } else if s.is_number() {
                Value::from(42)
            } else if s.is_boolean() {
                Value::Bool(true)
            } else {
                Value::String("item".to_string())
            }
        } else {
            Value::String("item".to_string())
        }
    }
}
