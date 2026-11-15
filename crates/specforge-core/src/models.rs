//! Normalized AST data models for OpenAPI specifications and JSON Schemas.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterLocation {
    Query,
    Path,
    Header,
    Cookie,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub location: ParameterLocation,
    pub required: bool,
    pub description: Option<String>,
    pub schema: Option<JSONSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    pub description: Option<String>,
    pub required: bool,
    pub content_type: String,
    pub schema: Option<JSONSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseDefinition {
    pub status_code: String,
    pub description: Option<String>,
    pub schema: Option<JSONSchema>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JSONSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<serde_json::Value>>,

    // String constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    // Numeric constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_minimum: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_maximum: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<f64>,

    // Array constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<JSONSchema>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,

    // Object constraints
    #[serde(default)]
    pub properties: BTreeMap<String, JSONSchema>,
    
    #[serde(default)]
    pub required: Vec<String>,
    
    #[serde(default = "default_true")]
    pub additional_properties: bool,

    // Nullability / Polymorphism
    #[serde(default)]
    pub nullable: bool,
    
    #[serde(default)]
    pub all_of: Vec<JSONSchema>,
    
    #[serde(default)]
    pub any_of: Vec<JSONSchema>,
    
    #[serde(default)]
    pub one_of: Vec<JSONSchema>,

    #[serde(default)]
    pub raw_schema: serde_json::Value,
}

fn default_true() -> bool {
    true
}

impl JSONSchema {
    pub fn is_string(&self) -> bool {
        self.schema_type.as_deref() == Some("string")
    }

    pub fn is_integer(&self) -> bool {
        self.schema_type.as_deref() == Some("integer")
    }

    pub fn is_number(&self) -> bool {
        matches!(self.schema_type.as_deref(), Some("number") | Some("integer"))
    }

    pub fn is_boolean(&self) -> bool {
        self.schema_type.as_deref() == Some("boolean")
    }

    pub fn is_array(&self) -> bool {
        self.schema_type.as_deref() == Some("array")
    }

    pub fn is_object(&self) -> bool {
        self.schema_type.as_deref() == Some("object") || !self.properties.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub key: String,
    pub method: String,
    pub path: String,
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    pub request_body: Option<RequestBody>,
    #[serde(default)]
    pub responses: BTreeMap<String, ResponseDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSpec {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
    #[serde(default)]
    pub servers: Vec<String>,
    #[serde(default)]
    pub operations: Vec<Operation>,
    pub raw_spec: serde_json::Value,
}
