//! # Change module
//!
//! Core Change data structure representing an atomic operation in the system.

use crate::operation::Operation;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a Change
pub type ChangeId = Uuid;

/// Unique identifier for an Agent
pub type AgentId = String;

/// A Change represents an atomic operation in the CRDT system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Change {
    /// Unique identifier for this change
    pub id: ChangeId,
    /// Timestamp when the change was created
    pub timestamp: DateTime<Utc>,
    /// References to parent changes (direct dependencies)
    pub parents: Vec<ChangeId>,
    /// The agent that created this change
    pub author: AgentId,
    /// The operation this change performs
    pub operation: Operation,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Change {
    /// Create a new Change
    pub fn new(author: AgentId, operation: Operation, parents: Vec<ChangeId>) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            parents,
            author,
            operation,
            metadata: HashMap::new(),
        }
    }

    /// Create a Change with a specific ID (useful for testing or replication)
    pub fn with_id(
        id: ChangeId,
        author: AgentId,
        operation: Operation,
        parents: Vec<ChangeId>,
    ) -> Self {
        Self {
            id,
            timestamp: Utc::now(),
            parents,
            author,
            operation,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the change
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if this change depends on another change
    pub fn depends_on(&self, other: &ChangeId) -> bool {
        self.parents.contains(other)
    }

    /// Get a summary of the change
    pub fn summary(&self) -> String {
        format!(
            "Change {} by {}: {} at {}",
            self.id,
            self.author,
            self.operation.type_name(),
            self.timestamp.format("%Y-%m-%d %H:%M:%S")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_creation() {
        let op = Operation::Insert {
            position: 0,
            content: "hello".to_string(),
        };
        let change = Change::new("agent1".to_string(), op.clone(), vec![]);

        assert_eq!(change.author, "agent1");
        assert_eq!(change.operation, op);
        assert!(change.parents.is_empty());
        assert!(change.metadata.is_empty());
    }

    #[test]
    fn test_change_with_id() {
        let id = Uuid::new_v4();
        let op = Operation::Delete {
            position: 0,
            length: 5,
        };
        let change = Change::with_id(id, "agent2".to_string(), op.clone(), vec![]);

        assert_eq!(change.id, id);
        assert_eq!(change.author, "agent2");
        assert_eq!(change.operation, op);
    }

    #[test]
    fn test_change_with_metadata() {
        let op = Operation::Insert {
            position: 0,
            content: "test".to_string(),
        };
        let change = Change::new("agent1".to_string(), op, vec![])
            .with_metadata("key".to_string(), serde_json::json!("value"));

        assert_eq!(
            change.metadata.get("key"),
            Some(&serde_json::json!("value"))
        );
    }

    #[test]
    fn test_depends_on() {
        let parent_id = Uuid::new_v4();
        let op = Operation::Insert {
            position: 0,
            content: "test".to_string(),
        };
        let change = Change::new("agent1".to_string(), op, vec![parent_id]);

        assert!(change.depends_on(&parent_id));
        assert!(!change.depends_on(&Uuid::new_v4()));
    }

    #[test]
    fn test_change_summary() {
        let op = Operation::Insert {
            position: 0,
            content: "hello".to_string(),
        };
        let change = Change::new("agent1".to_string(), op, vec![]);
        let summary = change.summary();

        assert!(summary.contains("agent1"));
        assert!(summary.contains("insert"));
    }

    #[test]
    fn test_change_serialization() {
        let op = Operation::Insert {
            position: 0,
            content: "hello".to_string(),
        };
        let change = Change::new("agent1".to_string(), op, vec![]);

        let serialized = serde_json::to_string(&change).unwrap();
        let deserialized: Change = serde_json::from_str(&serialized).unwrap();

        assert_eq!(change.id, deserialized.id);
        assert_eq!(change.author, deserialized.author);
    }
}
