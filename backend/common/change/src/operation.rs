//! # Operation types
//!
//! Defines the types of operations that can be performed in a change.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the type of operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    /// Insert text at a position
    Insert { position: usize, content: String },
    /// Delete text at a position
    Delete { position: usize, length: usize },
    /// Update a field value
    Update {
        path: String,
        value: serde_json::Value,
    },
    /// Create a new file/node
    Create { path: String, content: String },
    /// Remove a file/node
    Remove { path: String },
    /// Move/rename a file/node
    Move { from: String, to: String },
    /// Custom operation with arbitrary data
    Custom {
        operation_type: String,
        data: HashMap<String, serde_json::Value>,
    },
}

impl Operation {
    /// Returns the operation type name for serialization/metadata
    pub fn type_name(&self) -> &str {
        match self {
            Operation::Insert { .. } => "insert",
            Operation::Delete { .. } => "delete",
            Operation::Update { .. } => "update",
            Operation::Create { .. } => "create",
            Operation::Remove { .. } => "remove",
            Operation::Move { .. } => "move",
            Operation::Custom { operation_type, .. } => operation_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_type_name() {
        assert_eq!(
            Operation::Insert {
                position: 0,
                content: "hello".to_string()
            }
            .type_name(),
            "insert"
        );
        assert_eq!(
            Operation::Delete {
                position: 0,
                length: 5
            }
            .type_name(),
            "delete"
        );
        assert_eq!(
            Operation::Update {
                path: "key".to_string(),
                value: serde_json::json!("value")
            }
            .type_name(),
            "update"
        );
        assert_eq!(
            Operation::Create {
                path: "file.txt".to_string(),
                content: "content".to_string()
            }
            .type_name(),
            "create"
        );
        assert_eq!(
            Operation::Remove {
                path: "file.txt".to_string()
            }
            .type_name(),
            "remove"
        );
        assert_eq!(
            Operation::Move {
                from: "old.txt".to_string(),
                to: "new.txt".to_string()
            }
            .type_name(),
            "move"
        );
        assert_eq!(
            Operation::Custom {
                operation_type: "custom_op".to_string(),
                data: HashMap::new()
            }
            .type_name(),
            "custom_op"
        );
    }

    #[test]
    fn test_operation_serialization() {
        let op = Operation::Insert {
            position: 5,
            content: "test".to_string(),
        };
        let serialized = serde_json::to_string(&op).unwrap();
        let deserialized: Operation = serde_json::from_str(&serialized).unwrap();
        assert_eq!(op, deserialized);
    }

    #[test]
    fn test_custom_operation() {
        let mut data = HashMap::new();
        data.insert("key".to_string(), serde_json::json!("value"));
        let op = Operation::Custom {
            operation_type: "my_custom".to_string(),
            data,
        };
        assert_eq!(op.type_name(), "my_custom");
    }
}
