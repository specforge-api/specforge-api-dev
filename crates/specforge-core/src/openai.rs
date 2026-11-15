//! OpenAI Tool and Function Calling schema parser.
//!
//! Converts OpenAI tool declarations into standardized OpenAPI 3.1 operations
//! and JSON Schema models for boundary fuzzing and mock generation.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::{ApiSpec, Operation, RequestBody, ResponseDefinition};
use crate::openapi::OpenAPIParser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiFunction {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub parameters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiTool {
    pub r#type: String, // Usually "function"
    pub function: OpenAiFunction,
}

pub struct OpenAiToolParser;

impl OpenAiToolParser {
    /// Ingest an array of OpenAI tools or a root object containing `{"tools": [...]}`
    pub fn parse(raw: &Value) -> Result<ApiSpec, String> {
        let tools_array = if let Some(arr) = raw.as_array() {
            arr
        } else if let Some(arr) = raw.get("tools").and_then(|v| v.as_array()) {
            arr
        } else if let Some(arr) = raw.get("functions").and_then(|v| v.as_array()) {
            // Legacy OpenAI functions array
            return Self::parse_legacy_functions(arr, raw);
        } else {
            return Err("Expected an array of OpenAI tools or an object with a 'tools' key".to_string());
        };

        let dummy_root = serde_json::json!({});
        let schema_parser = OpenAPIParser::new(dummy_root);

        let mut operations = Vec::new();
        for item in tools_array {
            let tool: OpenAiTool = serde_json::from_value(item.clone())
                .map_err(|e| format!("Failed to parse OpenAI tool definition: {}", e))?;

            if tool.r#type != "function" {
                continue;
            }

            let path = format!("/tools/{}", tool.function.name);
            let op_key = format!("POST {}", path);
            let summary = tool.function.description.clone();
            let parsed_schema = schema_parser.parse_schema(&tool.function.parameters);

            let request_body = Some(RequestBody {
                description: Some("Parameters supplied by the LLM function call".to_string()),
                required: true,
                content_type: "application/json".to_string(),
                schema: Some(parsed_schema),
            });

            let mut responses = BTreeMap::new();
            responses.insert("200".to_string(), ResponseDefinition {
                status_code: "200".to_string(),
                description: Some("Function execution successful".to_string()),
                schema: None,
            });
            responses.insert("422".to_string(), ResponseDefinition {
                status_code: "422".to_string(),
                description: Some("Invalid function arguments from LLM".to_string()),
                schema: None,
            });

            operations.push(Operation {
                key: op_key,
                method: "POST".to_string(),
                path,
                operation_id: Some(tool.function.name.clone()),
                summary,
                description: None,
                tags: vec!["AI Tools".to_string()],
                parameters: Vec::new(),
                request_body,
                responses,
            });
        }

        Ok(ApiSpec {
            title: "OpenAI Tools API".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Auto-synthesized OpenAPI specification from OpenAI function definitions".to_string()),
            servers: vec!["http://localhost:4001".to_string()],
            operations,
            raw_spec: raw.clone(),
        })
    }

    fn parse_legacy_functions(functions: &[Value], raw: &Value) -> Result<ApiSpec, String> {
        let dummy_root = serde_json::json!({});
        let schema_parser = OpenAPIParser::new(dummy_root);
        let mut operations = Vec::new();

        for func in functions {
            let name = func.get("name").and_then(|v| v.as_str()).unwrap_or("unnamed_function");
            let desc = func.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
            let params = func.get("parameters").cloned().unwrap_or(serde_json::json!({"type": "object"}));

            let path = format!("/tools/{}", name);
            let op_key = format!("POST {}", path);
            let parsed_schema = schema_parser.parse_schema(&params);

            operations.push(Operation {
                key: op_key,
                method: "POST".to_string(),
                path,
                operation_id: Some(name.to_string()),
                summary: desc,
                description: None,
                tags: vec!["AI Tools".to_string()],
                parameters: Vec::new(),
                request_body: Some(RequestBody {
                    description: Some("Function arguments".to_string()),
                    required: true,
                    content_type: "application/json".to_string(),
                    schema: Some(parsed_schema),
                }),
                responses: BTreeMap::new(),
            });
        }

        Ok(ApiSpec {
            title: "OpenAI Legacy Functions API".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Auto-synthesized from OpenAI functions array".to_string()),
            servers: vec!["http://localhost:4001".to_string()],
            operations,
            raw_spec: raw.clone(),
        })
    }
}

