//! OpenAPI 3.0, 3.1 and Swagger 2.0 parser.

use std::collections::BTreeMap;
use serde_json::Value;

use crate::models::{
    ApiSpec, JSONSchema, Operation, Parameter, ParameterLocation, RequestBody, ResponseDefinition,
};
use crate::resolver::SchemaResolver;

pub struct OpenAPIParser {
    raw: Value,
}

impl OpenAPIParser {
    pub fn new(raw: Value) -> Self {
        Self { raw }
    }

    pub fn parse(&self) -> Result<ApiSpec, String> {
        let title = self.raw.pointer("/info/title")
            .and_then(|v| v.as_str())
            .unwrap_or("API Specification")
            .to_string();

        let version = self.raw.pointer("/info/version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0")
            .to_string();

        let description = self.raw.pointer("/info/description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut servers = Vec::new();
        if let Some(servers_arr) = self.raw.get("servers").and_then(|v| v.as_array()) {
            for s in servers_arr {
                if let Some(url) = s.get("url").and_then(|u| u.as_str()) {
                    servers.push(url.to_string());
                }
            }
        } else if let Some(host) = self.raw.get("host").and_then(|h| h.as_str()) {
            let base_path = self.raw.get("basePath").and_then(|b| b.as_str()).unwrap_or("");
            let scheme = self.raw.get("schemes")
                .and_then(|s| s.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("http");
            servers.push(format!("{}://{}{}", scheme, host, base_path));
        }

        let mut operations = Vec::new();
        if let Some(paths) = self.raw.get("paths").and_then(|p| p.as_object()) {
            for (path_str, path_item) in paths {
                if let Some(methods_map) = path_item.as_object() {
                    for (method, op_val) in methods_map {
                        let method_upper = method.to_uppercase();
                        if !matches!(method_upper.as_str(), "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS") {
                            continue;
                        }

                        let op_key = format!("{} {}", method_upper, path_str);
                        let operation_id = op_val.get("operationId").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let summary = op_val.get("summary").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let desc = op_val.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                        
                        let mut tags = Vec::new();
                        if let Some(tags_arr) = op_val.get("tags").and_then(|v| v.as_array()) {
                            for t in tags_arr {
                                if let Some(ts) = t.as_str() {
                                    tags.push(ts.to_string());
                                }
                            }
                        }

                        let mut parameters = Vec::new();
                        if let Some(params_arr) = op_val.get("parameters").and_then(|v| v.as_array()) {
                            for p in params_arr {
                                if let Some(param) = self.parse_parameter(p) {
                                    parameters.push(param);
                                }
                            }
                        }

                        let request_body = self.parse_request_body(op_val.get("requestBody"));

                        let mut responses = BTreeMap::new();
                        if let Some(resp_map) = op_val.get("responses").and_then(|r| r.as_object()) {
                            for (status_code, resp_val) in resp_map {
                                let desc = resp_val.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                                let schema = self.extract_schema_from_response(resp_val);
                                responses.insert(status_code.clone(), ResponseDefinition {
                                    status_code: status_code.clone(),
                                    description: desc,
                                    schema,
                                });
                            }
                        }

                        operations.push(Operation {
                            key: op_key,
                            method: method_upper,
                            path: path_str.clone(),
                            operation_id,
                            summary,
                            description: desc,
                            tags,
                            parameters,
                            request_body,
                            responses,
                        });
                    }
                }
            }
        }

        Ok(ApiSpec {
            title,
            version,
            description,
            servers,
            operations,
            raw_spec: self.raw.clone(),
        })
    }

    fn parse_parameter(&self, p: &Value) -> Option<Parameter> {
        let name = p.get("name")?.as_str()?.to_string();
        let in_str = p.get("in")?.as_str()?;
        let location = match in_str {
            "query" => ParameterLocation::Query,
            "path" => ParameterLocation::Path,
            "header" => ParameterLocation::Header,
            "cookie" => ParameterLocation::Cookie,
            _ => return None,
        };
        let required = p.get("required").and_then(|v| v.as_bool()).unwrap_or(location == ParameterLocation::Path);
        let description = p.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
        
        let schema = if let Some(s) = p.get("schema") {
            Some(self.parse_schema(s))
        } else {
            // Swagger 2.0 parameter types
            Some(self.parse_schema(p))
        };

        Some(Parameter {
            name,
            location,
            required,
            description,
            schema,
        })
    }

    fn parse_request_body(&self, body_val: Option<&Value>) -> Option<RequestBody> {
        let body = body_val?;
        let description = body.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
        let required = body.get("required").and_then(|v| v.as_bool()).unwrap_or(false);

        let content = body.get("content")?.as_object()?;
        // Prefer application/json
        let (content_type, media_val) = if let Some(json_val) = content.get("application/json") {
            ("application/json".to_string(), json_val)
        } else if let Some((ct, val)) = content.iter().next() {
            (ct.clone(), val)
        } else {
            return None;
        };

        let schema = media_val.get("schema").map(|s| self.parse_schema(s));

        Some(RequestBody {
            description,
            required,
            content_type,
            schema,
        })
    }

    fn extract_schema_from_response(&self, resp: &Value) -> Option<JSONSchema> {
        if let Some(content) = resp.get("content").and_then(|c| c.as_object()) {
            if let Some(json_media) = content.get("application/json") {
                if let Some(schema) = json_media.get("schema") {
                    return Some(self.parse_schema(schema));
                }
            }
        }
        // Swagger 2.0 schema location
        if let Some(schema) = resp.get("schema") {
            return Some(self.parse_schema(schema));
        }
        None
    }

    pub fn parse_schema(&self, raw_schema: &Value) -> JSONSchema {
        let mut resolver = SchemaResolver::new(&self.raw);
        let resolved = if let Some(ref_ptr) = raw_schema.get("$ref").and_then(|r| r.as_str()) {
            resolver.resolve(ref_ptr).unwrap_or_else(|_| raw_schema.clone())
        } else {
            raw_schema.clone()
        };

        let schema_type = resolved.get("type").and_then(|v| v.as_str()).map(|s| s.to_string());
        let format = resolved.get("format").and_then(|v| v.as_str()).map(|s| s.to_string());
        let title = resolved.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());
        let description = resolved.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
        let default = resolved.get("default").cloned();
        let example = resolved.get("example").cloned();

        let enum_values = resolved.get("enum").and_then(|v| v.as_array()).map(|arr| arr.clone());
        let min_length = resolved.get("minLength").and_then(|v| v.as_u64()).map(|u| u as usize);
        let max_length = resolved.get("maxLength").and_then(|v| v.as_u64()).map(|u| u as usize);
        let pattern = resolved.get("pattern").and_then(|v| v.as_str()).map(|s| s.to_string());

        let minimum = resolved.get("minimum").and_then(|v| v.as_f64());
        let maximum = resolved.get("maximum").and_then(|v| v.as_f64());
        let exclusive_minimum = resolved.get("exclusiveMinimum").and_then(|v| v.as_f64());
        let exclusive_maximum = resolved.get("exclusiveMaximum").and_then(|v| v.as_f64());
        let multiple_of = resolved.get("multipleOf").and_then(|v| v.as_f64());

        let items = resolved.get("items").map(|it| Box::new(self.parse_schema(it)));
        let min_items = resolved.get("minItems").and_then(|v| v.as_u64()).map(|u| u as usize);
        let max_items = resolved.get("maxItems").and_then(|v| v.as_u64()).map(|u| u as usize);
        let unique_items = resolved.get("uniqueItems").and_then(|v| v.as_bool());

        let mut properties = BTreeMap::new();
        if let Some(props) = resolved.get("properties").and_then(|p| p.as_object()) {
            for (k, v) in props {
                properties.insert(k.clone(), self.parse_schema(v));
            }
        }

        let mut required = Vec::new();
        if let Some(req_arr) = resolved.get("required").and_then(|r| r.as_array()) {
            for r in req_arr {
                if let Some(rs) = r.as_str() {
                    required.push(rs.to_string());
                }
            }
        }

        let additional_properties = resolved.get("additionalProperties")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let nullable = resolved.get("nullable").and_then(|v| v.as_bool()).unwrap_or(false);

        JSONSchema {
            schema_type,
            format,
            title,
            description,
            default,
            example,
            enum_values,
            min_length,
            max_length,
            pattern,
            minimum,
            maximum,
            exclusive_minimum,
            exclusive_maximum,
            multiple_of,
            items,
            min_items,
            max_items,
            unique_items,
            properties,
            required,
            additional_properties,
            nullable,
            all_of: Vec::new(),
            any_of: Vec::new(),
            one_of: Vec::new(),
            raw_schema: resolved,
        }
    }
}
