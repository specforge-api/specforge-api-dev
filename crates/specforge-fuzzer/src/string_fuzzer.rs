//! Schema-aware string mutation fuzzer (whitespace, bounds, SQLi/XSS, format violations).

use serde_json::Value;
use specforge_core::models::JSONSchema;

pub struct StringMutation {
    pub name: String,
    pub description: String,
    pub value: Value,
    pub is_negative: bool,
}

pub struct StringFuzzer;

impl StringFuzzer {
    pub fn generate_mutations(schema: &JSONSchema) -> Vec<StringMutation> {
        let mut mutations = Vec::new();

        // 1. Whitespace Only
        mutations.push(StringMutation {
            name: "Whitespace: Leading/Trailing Spaces".to_string(),
            description: "Probe unstripped whitespace-only string: '   '".to_string(),
            value: Value::String("   ".to_string()),
            is_negative: true,
        });

        // 2. Empty String
        let min_len = schema.min_length.unwrap_or(0);
        mutations.push(StringMutation {
            name: "Boundary: Empty String".to_string(),
            description: "Probe empty string: ''".to_string(),
            value: Value::String("".to_string()),
            is_negative: min_len > 0,
        });

        // 3. Min Length Violation
        if let Some(min) = schema.min_length {
            if min > 1 {
                let under = "a".repeat(min - 1);
                mutations.push(StringMutation {
                    name: format!("Boundary: Under Min Length ({})", min - 1),
                    description: format!("String of length {} (minimum required is {})", min - 1, min),
                    value: Value::String(under),
                    is_negative: true,
                });
            }
        }

        // 4. Max Length Violation
        if let Some(max) = schema.max_length {
            let over = "a".repeat(max + 1);
            mutations.push(StringMutation {
                name: format!("Boundary: Exceed Max Length ({})", max + 1),
                description: format!("String of length {} (maximum allowed is {})", max + 1, max),
                value: Value::String(over),
                is_negative: true,
            });
        }

        // 5. Format-Specific Violations
        if let Some(ref format) = schema.format {
            match format.as_str() {
                "email" => {
                    mutations.push(StringMutation {
                        name: "Format: Invalid Email".to_string(),
                        description: "Malformed email without domain".to_string(),
                        value: Value::String("missing_at_sign.com".to_string()),
                        is_negative: true,
                    });
                }
                "uuid" => {
                    mutations.push(StringMutation {
                        name: "Format: Invalid UUID".to_string(),
                        description: "Non-UUID alphanumeric string".to_string(),
                        value: Value::String("not-a-valid-uuid-1234".to_string()),
                        is_negative: true,
                    });
                }
                "date-time" => {
                    mutations.push(StringMutation {
                        name: "Format: Invalid DateTime".to_string(),
                        description: "Non-ISO8601 date string".to_string(),
                        value: Value::String("2026-99-99T99:99:99Z".to_string()),
                        is_negative: true,
                    });
                }
                _ => {}
            }
        }

        // 6. Security Probes & Emojis
        mutations.push(StringMutation {
            name: "Unicode: 4-Byte Emojis".to_string(),
            description: "Surrogate pair / 4-byte UTF-8 emoji string: '🚀🔥⚡'".to_string(),
            value: Value::String("🚀🔥⚡".to_string()),
            is_negative: false, // Resilience check: only fails on 500
        });

        mutations.push(StringMutation {
            name: "Security: Basic SQL Injection Probe".to_string(),
            description: "Classic SQL quote probe: ' OR '1'='1".to_string(),
            value: Value::String("' OR '1'='1".to_string()),
            is_negative: false, // Resilience check: only fails on 500
        });

        mutations.push(StringMutation {
            name: "Security: XSS Script Tag Probe".to_string(),
            description: "HTML script injection payload: <script>alert(1)</script>".to_string(),
            value: Value::String("<script>alert(1)</script>".to_string()),
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
        if let Some(ref format) = schema.format {
            return match format.as_str() {
                "email" => Value::String("dev@example.com".to_string()),
                "uuid" => Value::String("123e4567-e89b-12d3-a456-426614174000".to_string()),
                "date-time" => Value::String("2026-01-01T00:00:00Z".to_string()),
                "uri" | "url" => Value::String("https://example.com".to_string()),
                _ => Value::String("valid_string".to_string()),
            };
        }

        let min = schema.min_length.unwrap_or(1);
        let base = "valid_sample_text";
        if base.len() < min {
            Value::String("a".repeat(min))
        } else {
            Value::String(base.to_string())
        }
    }
}
