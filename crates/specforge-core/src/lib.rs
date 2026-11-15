//! SpecForge Core: AST models, schema parsers, and pointer resolution.

pub mod models;
pub mod openapi;
pub mod openai;
pub mod resolver;

pub use models::{ApiSpec, JSONSchema, Operation, Parameter, ParameterLocation, RequestBody, ResponseDefinition};
pub use openapi::OpenAPIParser;
pub use openai::OpenAiToolParser;
pub use resolver::SchemaResolver;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_simple_openapi() {
        let spec_json = json!({
            "openapi": "3.0.0",
            "info": {
                "title": "Test Store",
                "version": "1.0.0"
            },
            "paths": {
                "/users": {
                    "post": {
                        "summary": "Create user",
                        "requestBody": {
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "username": { "type": "string", "minLength": 3 },
                                            "age": { "type": "integer", "minimum": 18 }
                                        },
                                        "required": ["username"]
                                    }
                                }
                            }
                        },
                        "responses": {
                            "201": { "description": "Created" }
                        }
                    }
                }
            }
        });

        let parser = OpenAPIParser::new(spec_json);
        let spec = parser.parse().expect("Failed to parse spec");

        assert_eq!(spec.title, "Test Store");
        assert_eq!(spec.operations.len(), 1);
        let op = &spec.operations[0];
        assert_eq!(op.method, "POST");
        assert_eq!(op.path, "/users");
        
        let body = op.request_body.as_ref().expect("Missing request body");
        let schema = body.schema.as_ref().expect("Missing body schema");
        assert!(schema.is_object());
        assert_eq!(schema.required, vec!["username"]);
    }

    #[test]
    fn test_parse_openai_tool_definitions() {
        let tools_json = json!([
            {
                "type": "function",
                "function": {
                    "name": "search_database",
                    "description": "Execute full-text search",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "query": { "type": "string", "minLength": 1 },
                            "limit": { "type": "integer", "maximum": 50 }
                        },
                        "required": ["query"]
                    }
                }
            }
        ]);

        let spec = OpenAiToolParser::parse(&tools_json).expect("Failed to parse OpenAI tools");
        assert_eq!(spec.operations.len(), 1);
        let op = &spec.operations[0];
        assert_eq!(op.path, "/tools/search_database");
        assert_eq!(op.method, "POST");
        assert_eq!(op.summary.as_deref(), Some("Execute full-text search"));
    }
}

