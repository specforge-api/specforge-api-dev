//! Object mutation fuzzer (missing required fields, unexpected keys, baseline generator).

use serde_json::{Map, Value};
use specforge_core::models::JSONSchema;
use crate::array_fuzzer::ArrayFuzzer;
use crate::boolean_fuzzer::BooleanFuzzer;
use crate::numeric_fuzzer::NumericFuzzer;
use crate::string_fuzzer::StringFuzzer;

pub struct ObjectMutation {
    pub name: String,
    pub description: String,
    pub payload: Value,
    pub category: String,
}

pub struct ObjectFuzzer;

impl ObjectFuzzer {
    /// Recursively generate a valid baseline JSON payload matching all properties
    pub fn generate_valid_baseline(schema: &JSONSchema) -> Value {
        if let Some(ref ex) = schema.example {
            return ex.clone();
        }
        if let Some(ref def) = schema.default {
            return def.clone();
        }

        let mut map = Map::new();
        for (prop_name, prop_schema) in &schema.properties {
            let val = if prop_schema.is_string() {
                StringFuzzer::generate_valid_baseline(prop_schema)
            } else if prop_schema.is_number() {
                NumericFuzzer::generate_valid_baseline(prop_schema)
            } else if prop_schema.is_boolean() {
                BooleanFuzzer::generate_valid_baseline(prop_schema)
            } else if prop_schema.is_array() {
                ArrayFuzzer::generate_valid_baseline(prop_schema)
            } else if prop_schema.is_object() {
                Self::generate_valid_baseline(prop_schema)
            } else {
                Value::String("sample_value".to_string())
            };
            map.insert(prop_name.clone(), val);
        }

        Value::Object(map)
    }

    /// Generate object-level mutations (missing keys, extra keys, empty objects)
    pub fn generate_object_mutations(schema: &JSONSchema, baseline: &Value) -> Vec<ObjectMutation> {
        let mut mutations = Vec::new();

        // 1. Empty Object
        if !schema.required.is_empty() {
            mutations.push(ObjectMutation {
                name: "Object: Empty Body ({})".to_string(),
                description: "Supplying completely empty JSON object when required fields exist".to_string(),
                payload: Value::Object(Map::new()),
                category: "MissingBody".to_string(),
            });
        }

        // 2. Combinatorial Missing Required Fields
        if let Value::Object(ref base_map) = baseline {
            for req_key in &schema.required {
                let mut mutated_map = base_map.clone();
                mutated_map.remove(req_key);
                mutations.push(ObjectMutation {
                    name: format!("Missing Required Field: '{}'", req_key),
                    description: format!("Omission of declared required property '{}'", req_key),
                    payload: Value::Object(mutated_map),
                    category: "MissingRequired".to_string(),
                });
            }

            // 3. Unexpected Extra Properties (additionalProperties probe)
            let mut extra_map = base_map.clone();
            extra_map.insert(
                "__specforge_probe__".to_string(),
                Value::String("unexpected_extra_property_value".to_string()),
            );
            mutations.push(ObjectMutation {
                name: "Object: Unexpected Extra Property".to_string(),
                description: "Inject undeclared key '__specforge_probe__' to verify strict schema adherence".to_string(),
                payload: Value::Object(extra_map),
                category: "ExtraProperties".to_string(),
            });
        }

        // 4. Primitive Inversion
        mutations.push(ObjectMutation {
            name: "Type Inversion: String instead of Object".to_string(),
            description: "Supplying primitive string 'malformed_payload' when an object is expected".to_string(),
            payload: Value::String("malformed_payload".to_string()),
            category: "TypeInversion".to_string(),
        });

        mutations
    }
}

