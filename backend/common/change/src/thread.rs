//! # Thread module
//!
//! Manages threads - sequences of changes that can fork and merge.

use crate::change::{Change, ChangeId};
use crate::version::VectorClock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a Thread
pub type ThreadId = Uuid;

/// Metadata associated with a thread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadMetadata {
    /// Human-readable name for the thread
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Thread that this thread was forked from (if any)
    pub parent_thread: Option<ThreadId>,
    /// Additional custom metadata
    pub extra: HashMap<String, serde_json::Value>,
}

impl ThreadMetadata {
    /// Create new thread metadata
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            parent_thread: None,
            extra: HashMap::new(),
        }
    }

    /// Set the parent thread
    pub fn with_parent(mut self, parent: ThreadId) -> Self {
        self.parent_thread = Some(parent);
        self
    }

    /// Set the description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

/// A Thread represents a sequence of changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    /// Unique identifier for this thread
    pub id: ThreadId,
    /// The head (most recent) change in this thread
    pub head: Option<ChangeId>,
    /// The change where this thread was forked (if applicable)
    pub fork_point: Option<ChangeId>,
    /// Metadata about this thread
    pub metadata: ThreadMetadata,
    /// The vector clock tracking causality for this thread
    pub vector_clock: VectorClock,
}

impl Thread {
    /// Create a new empty thread
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            head: None,
            fork_point: None,
            metadata: ThreadMetadata::new(name),
            vector_clock: VectorClock::new(),
        }
    }

    /// Create a thread with a specific ID
    pub fn with_id(id: ThreadId, name: String) -> Self {
        Self {
            id,
            head: None,
            fork_point: None,
            metadata: ThreadMetadata::new(name),
            vector_clock: VectorClock::new(),
        }
    }

    /// Set the fork point
    pub fn with_fork_point(mut self, fork_point: ChangeId) -> Self {
        self.fork_point = Some(fork_point);
        self
    }

    /// Add a change to this thread
    pub fn add_change(&mut self, change: &Change) {
        self.head = Some(change.id);
        self.vector_clock.increment(&change.author);
    }

    /// Set the head change directly
    pub fn set_head(&mut self, change_id: ChangeId) {
        self.head = Some(change_id);
    }

    /// Check if this thread is empty (no changes)
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    /// Get the thread's parent (if any)
    pub fn parent(&self) -> Option<ThreadId> {
        self.metadata.parent_thread
    }

    /// Fork this thread at a specific change
    pub fn fork(&self, name: String, at_change: ChangeId) -> Thread {
        Thread {
            id: Uuid::new_v4(),
            head: Some(at_change),
            fork_point: Some(at_change),
            metadata: ThreadMetadata::new(name).with_parent(self.id),
            vector_clock: self.vector_clock.clone(),
        }
    }

    /// Merge another thread into this one
    pub fn merge_from(&mut self, other: &Thread, merge_change: ChangeId) {
        self.vector_clock.merge(&other.vector_clock);
        self.head = Some(merge_change);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::Operation;

    #[test]
    fn test_thread_creation() {
        let thread = Thread::new("main".to_string());

        assert!(thread.is_empty());
        assert_eq!(thread.metadata.name, "main");
        assert!(thread.fork_point.is_none());
        assert!(thread.parent().is_none());
    }

    #[test]
    fn test_thread_with_id() {
        let id = Uuid::new_v4();
        let thread = Thread::with_id(id, "test".to_string());

        assert_eq!(thread.id, id);
        assert_eq!(thread.metadata.name, "test");
    }

    #[test]
    fn test_thread_with_fork_point() {
        let fork_point = Uuid::new_v4();
        let thread = Thread::new("branch".to_string()).with_fork_point(fork_point);

        assert_eq!(thread.fork_point, Some(fork_point));
    }

    #[test]
    fn test_add_change() {
        let mut thread = Thread::new("main".to_string());

        let change = Change::new(
            "agent1".to_string(),
            Operation::Insert {
                position: 0,
                content: "test".to_string(),
            },
            vec![],
        );
        let change_id = change.id;

        thread.add_change(&change);

        assert_eq!(thread.head, Some(change_id));
        assert_eq!(thread.vector_clock.get(&"agent1".to_string()), Some(&1));
    }

    #[test]
    fn test_multiple_changes_increment_clock() {
        let mut thread = Thread::new("main".to_string());

        let change1 = Change::new(
            "agent1".to_string(),
            Operation::Insert {
                position: 0,
                content: "a".to_string(),
            },
            vec![],
        );
        let change2 = Change::new(
            "agent1".to_string(),
            Operation::Insert {
                position: 1,
                content: "b".to_string(),
            },
            vec![],
        );

        thread.add_change(&change1);
        thread.add_change(&change2);

        assert_eq!(thread.vector_clock.get(&"agent1".to_string()), Some(&2));
    }

    #[test]
    fn test_fork() {
        let mut parent = Thread::new("main".to_string());

        let change = Change::new(
            "agent1".to_string(),
            Operation::Insert {
                position: 0,
                content: "test".to_string(),
            },
            vec![],
        );
        let change_id = change.id;
        parent.add_change(&change);

        let child = parent.fork("branch".to_string(), change_id);

        assert_eq!(child.fork_point, Some(change_id));
        assert_eq!(child.head, Some(change_id));
        assert_eq!(child.parent(), Some(parent.id));
        assert_eq!(child.metadata.name, "branch");
    }

    #[test]
    fn test_merge_from() {
        let mut thread1 = Thread::new("main".to_string());
        let mut thread2 = Thread::new("branch".to_string());

        let change1 = Change::new(
            "agent1".to_string(),
            Operation::Insert {
                position: 0,
                content: "a".to_string(),
            },
            vec![],
        );
        let change2 = Change::new(
            "agent2".to_string(),
            Operation::Insert {
                position: 1,
                content: "b".to_string(),
            },
            vec![],
        );

        thread1.add_change(&change1);
        thread2.add_change(&change2);

        let merge_change_id = Uuid::new_v4();
        thread1.merge_from(&thread2, merge_change_id);

        assert_eq!(thread1.head, Some(merge_change_id));
        assert_eq!(thread1.vector_clock.get(&"agent1".to_string()), Some(&1));
        assert_eq!(thread1.vector_clock.get(&"agent2".to_string()), Some(&1));
    }

    #[test]
    fn test_thread_metadata() {
        let metadata = ThreadMetadata::new("test".to_string())
            .with_description("A test thread".to_string())
            .with_parent(Uuid::new_v4());

        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.description, Some("A test thread".to_string()));
        assert!(metadata.parent_thread.is_some());
    }

    #[test]
    fn test_thread_serialization() {
        let thread = Thread::new("main".to_string());

        let serialized = serde_json::to_string(&thread).unwrap();
        let deserialized: Thread = serde_json::from_str(&serialized).unwrap();

        assert_eq!(thread.id, deserialized.id);
        assert_eq!(thread.metadata.name, deserialized.metadata.name);
    }

    #[test]
    fn test_set_head() {
        let mut thread = Thread::new("main".to_string());
        let change_id = Uuid::new_v4();

        thread.set_head(change_id);

        assert_eq!(thread.head, Some(change_id));
    }
}
