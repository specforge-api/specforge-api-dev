//! SpecForge Mock: Dynamic schema-aware mock data synthesizer.

pub mod generator;

pub use generator::MockGenerator;

#[cfg(test)]
mod tests {
    use super::*;
    use specforge_core::models::JSONSchema;
    use std::collections::BTreeMap;

    #[test]
    fn test_mock_generator_creates_valid_object() {
        let mut props = BTreeMap::new();
        props.insert("id".to_string(), JSONSchema {
            schema_type: Some("string".to_string()),
            format: Some("uuid".to_string()),
            ..Default::default()
        });
        props.insert("email".to_string(), JSONSchema {
            schema_type: Some("string".to_string()),
            format: Some("email".to_string()),
            ..Default::default()
        });
        props.insert("age".to_string(), JSONSchema {
            schema_type: Some("integer".to_string()),
            minimum: Some(18.0),
            maximum: Some(60.0),
            ..Default::default()
        });

        let schema = JSONSchema {
            schema_type: Some("object".to_string()),
            properties: props,
            ..Default::default()
        };

        let mock_val = MockGenerator::generate_response(&schema);
        assert!(mock_val.is_object());
        let obj = mock_val.as_object().unwrap();
        assert!(obj.contains_key("id"));
        assert!(obj.contains_key("email"));
        assert_eq!(obj.get("email").unwrap(), "developer@specforge.dev");
        assert!(obj.get("age").unwrap().as_i64().unwrap() >= 18);
    }
}
