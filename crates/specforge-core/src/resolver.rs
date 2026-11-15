//! JSON Pointer ($ref) resolver with circular reference detection.

use std::collections::HashSet;
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum ResolverError {
    #[error("Invalid JSON Pointer syntax: {0}")]
    InvalidPointer(String),
    #[error("Referenced schema not found at: {0}")]
    NotFound(String),
    #[error("Circular reference detected at pointer: {0}")]
    CircularReference(String),
}

pub struct SchemaResolver<'a> {
    root: &'a Value,
    visited: HashSet<String>,
}

impl<'a> SchemaResolver<'a> {
    pub fn new(root: &'a Value) -> Self {
        Self {
            root,
            visited: HashSet::new(),
        }
    }

    /// Resolve a JSON pointer reference like `#/components/schemas/User`
    pub fn resolve(&mut self, ref_path: &str) -> Result<Value, ResolverError> {
        if self.visited.contains(ref_path) {
            return Err(ResolverError::CircularReference(ref_path.to_string()));
        }
        self.visited.insert(ref_path.to_string());

        let target_val = self.resolve_pointer(ref_path)?;
        
        // If the resolved target is itself a $ref, follow it recursively
        if let Some(nested_ref) = target_val.get("$ref").and_then(|r| r.as_str()) {
            return self.resolve(nested_ref);
        }

        self.visited.remove(ref_path);
        Ok(target_val.clone())
    }

    /// Navigate the root JSON document using a standard RFC 6901 JSON pointer
    pub fn resolve_pointer(&self, pointer: &str) -> Result<&'a Value, ResolverError> {
        if !pointer.starts_with('#') {
            return Err(ResolverError::InvalidPointer(
                "Only local fragment pointers (#/...) are currently supported".to_string(),
            ));
        }

        let path = &pointer[1..]; // Strip leading '#'
        if path.is_empty() || path == "/" {
            return Ok(self.root);
        }

        let parts: Vec<&str> = path.split('/').skip(1).collect();
        let mut current = self.root;

        for part in parts {
            let unescaped = part.replace("~1", "/").replace("~0", "~");
            match current {
                Value::Object(map) => {
                    if let Some(next) = map.get(&unescaped) {
                        current = next;
                    } else {
                        return Err(ResolverError::NotFound(pointer.to_string()));
                    }
                }
                Value::Array(arr) => {
                    if let Ok(idx) = unescaped.parse::<usize>() {
                        if let Some(next) = arr.get(idx) {
                            current = next;
                        } else {
                            return Err(ResolverError::NotFound(pointer.to_string()));
                        }
                    } else {
                        return Err(ResolverError::InvalidPointer(pointer.to_string()));
                    }
                }
                _ => return Err(ResolverError::NotFound(pointer.to_string())),
            }
        }

        Ok(current)
    }
}

