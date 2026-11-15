//! Dynamic schema-aware mock data generator for unbuilt endpoints and AI tools.

use serde_json::{Map, Value};
use specforge_core::models::JSONSchema;
use uuid::Uuid;

pub struct MockGenerator;

impl MockGenerator {
    /// Generates a valid JSON mock response body from an OpenAPI JSONSchema
    pub fn generate_response(schema: &JSONSchema) -> Value {
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

        if schema.is_string() {
            Self::generate_mock_string(schema)
        } else if schema.is_integer() {
            let min = schema.minimum.unwrap_or(1.0) as i64;
            let max = schema.maximum.unwrap_or(100.0) as i64;
            Value::from((min + max) / 2)
        } else if schema.is_number() {
            let min = schema.minimum.unwrap_or(1.0);
            let max = schema.maximum.unwrap_or(100.0);
            serde_json::json!((min + max) / 2.0)
        } else if schema.is_boolean() {
            Value::Bool(true)
        } else if schema.is_array() {
            let count = schema.min_items.unwrap_or(2).clamp(1usize, 5usize);
            let mut arr = Vec::new();
            if let Some(ref items_schema) = schema.items {
                for _ in 0..count {
                    arr.push(Self::generate_response(items_schema));
                }
            } else {
                arr.push(Value::String("mock_item_1".to_string()));
                arr.push(Value::String("mock_item_2".to_string()));
            }
            Value::Array(arr)
        } else if schema.is_object() {
            let mut obj_map = Map::new();
            for (prop_name, prop_schema) in &schema.properties {
                obj_map.insert(prop_name.clone(), Self::generate_response(prop_schema));
            }
            Value::Object(obj_map)
        } else {
            Value::String("mock_data".to_string())
        }
    }

    fn generate_mock_string(schema: &JSONSchema) -> Value {
        if let Some(ref format) = schema.format {
            return match format.as_str() {
                "uuid" => Value::String(Uuid::new_v4().to_string()),
                "email" => Value::String("developer@specforge.dev".to_string()),
                "date-time" => Value::String("2026-11-15T12:00:00Z".to_string()),
                "date" => Value::String("2026-11-15".to_string()),
                "uri" | "url" => Value::String("https://specforge.dev".to_string()),
                "ipv4" => Value::String("192.168.1.100".to_string()),
                _ => Value::String("sample_value".to_string()),
            };
        }

        if let Some(min) = schema.min_length {
            let base = "mock_string_value";
            if base.len() < min {
                return Value::String("a".repeat(min));
            }
        }

        Value::String("mock_string_value".to_string())
    }
}

