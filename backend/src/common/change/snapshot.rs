//! # Snapshot module
//!
//! Generates snapshots from sequences of changes for real-time viewing.

use crate::common::change::change::{Change, ChangeId};
use crate::common::change::operation::Operation;
use crate::common::change::thread::Thread;
use crate::common::change::version::VectorClock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A snapshot represents the complete state at a specific point in time
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    /// The change ID that this snapshot represents
    pub change_id: Option<ChangeId>,
    /// The state content (e.g., document text)
    pub content: String,
    /// Key-value pairs for structured data
    pub data: HashMap<String, serde_json::Value>,
    /// Files and their contents
    pub files: HashMap<String, String>,
    /// The vector clock at this point
    pub vector_clock: VectorClock,
    /// Metadata about the snapshot
    pub metadata: SnapshotMetadata,
}

/// Metadata about a snapshot
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// When this snapshot was generated
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// The thread this snapshot belongs to
    pub thread_id: Option<uuid::Uuid>,
    /// Additional metadata
    pub extra: HashMap<String, serde_json::Value>,
}

impl Snapshot {
    /// Create a new empty snapshot
    pub fn new() -> Self {
        Self {
            change_id: None,
            content: String::new(),
            data: HashMap::new(),
            files: HashMap::new(),
            vector_clock: VectorClock::new(),
            metadata: SnapshotMetadata {
                generated_at: chrono::Utc::now(),
                thread_id: None,
                extra: HashMap::new(),
            },
        }
    }

    /// Create a snapshot with a specific change ID
    pub fn with_change_id(change_id: ChangeId) -> Self {
        let mut snapshot = Self::new();
        snapshot.change_id = Some(change_id);
        snapshot
    }

    /// Get the content as a string
    pub fn as_str(&self) -> &str {
        &self.content
    }

    /// Get a value from the data map
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }

    /// Get a file's content
    pub fn get_file(&self, path: &str) -> Option<&str> {
        self.files.get(path).map(|s| s.as_str())
    }
}

impl Default for Snapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot generation engine
#[derive(Debug, Clone)]
pub struct SnapshotGenerator {
    /// Cache for incremental updates
    cache: HashMap<ChangeId, Snapshot>,
}

impl SnapshotGenerator {
    /// Create a new snapshot generator
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Generate a snapshot from a thread's head change
    ///
    /// This method traverses the change history and applies all operations
    pub fn generate_from_thread(
        &mut self,
        thread: &Thread,
        changes: &HashMap<ChangeId, Change>,
    ) -> Snapshot {
        if let Some(head) = thread.head {
            self.generate_from_change(head, changes)
        } else {
            Snapshot::new()
        }
    }

    /// Generate a snapshot starting from a specific change
    ///
    /// Traverses backwards through parent changes and applies operations in order
    pub fn generate_from_change(
        &mut self,
        change_id: ChangeId,
        changes: &HashMap<ChangeId, Change>,
    ) -> Snapshot {
        // Collect all changes in reverse order (from root to tip)
        let mut change_sequence = Vec::new();
        let mut to_visit = vec![change_id];
        let mut visited = std::collections::HashSet::new();

        while let Some(id) = to_visit.pop() {
            if visited.contains(&id) {
                continue;
            }
            visited.insert(id);

            if let Some(change) = changes.get(&id) {
                change_sequence.push(change.clone());
                to_visit.extend(change.parents.iter());
            }
        }

        // Reverse to get root-to-tip order
        change_sequence.reverse();

        // Apply changes in order
        let mut snapshot = Snapshot::new();
        snapshot.change_id = Some(change_id);

        for change in &change_sequence {
            self.apply_change(&mut snapshot, change);
            snapshot.vector_clock.increment(&change.author);
        }

        // Cache the result
        self.cache.insert(change_id, snapshot.clone());

        snapshot
    }

    /// Generate a snapshot incrementally from a previous snapshot
    ///
    /// Only applies new changes since the previous snapshot's change
    pub fn generate_incremental(
        &mut self,
        from_change: ChangeId,
        to_change: ChangeId,
        changes: &HashMap<ChangeId, Change>,
    ) -> Snapshot {
        // Try to use cached snapshot as base
        let base_snapshot = self.cache.get(&from_change).cloned().unwrap_or_default();

        // Find the path from from_change to to_change
        let mut new_changes = Vec::new();
        let mut to_visit = vec![to_change];
        let mut visited = std::collections::HashSet::new();

        while let Some(id) = to_visit.pop() {
            if id == from_change || visited.contains(&id) {
                continue;
            }
            visited.insert(id);

            if let Some(change) = changes.get(&id) {
                new_changes.push(change.clone());
                to_visit.extend(change.parents.iter());
            }
        }

        // Sort changes by timestamp to get correct order
        new_changes.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Apply new changes
        let mut snapshot = base_snapshot;
        snapshot.change_id = Some(to_change);

        for change in &new_changes {
            self.apply_change(&mut snapshot, change);
            snapshot.vector_clock.increment(&change.author);
        }

        // Cache the result
        self.cache.insert(to_change, snapshot.clone());

        snapshot
    }

    /// Apply a single change to a snapshot
    fn apply_change(&self, snapshot: &mut Snapshot, change: &Change) {
        match &change.operation {
            Operation::Insert { position, content } => {
                let pos = (*position).min(snapshot.content.len());
                snapshot.content.insert_str(pos, content);
            }
            Operation::Delete { position, length } => {
                let start = (*position).min(snapshot.content.len());
                let end = (start + *length).min(snapshot.content.len());
                snapshot.content.replace_range(start..end, "");
            }
            Operation::Update { path, value } => {
                snapshot.data.insert(path.clone(), value.clone());
            }
            Operation::Create { path, content } => {
                snapshot.files.insert(path.clone(), content.clone());
            }
            Operation::Remove { path } => {
                snapshot.files.remove(path);
            }
            Operation::Move { from, to } => {
                if let Some(content) = snapshot.files.remove(from) {
                    snapshot.files.insert(to.clone(), content);
                }
            }
            Operation::Custom {
                operation_type: _,
                data,
            } => {
                // For custom operations, merge the data into the snapshot
                for (key, value) in data {
                    snapshot.data.insert(key.clone(), value.clone());
                }
            }
        }
    }

    /// Clear the snapshot cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get the number of cached snapshots
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for SnapshotGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_change(author: &str, parents: Vec<ChangeId>, operation: Operation) -> Change {
        Change::new(author.to_string(), operation, parents)
    }

    #[test]
    fn test_snapshot_creation() {
        let snapshot = Snapshot::new();

        assert!(snapshot.content.is_empty());
        assert!(snapshot.files.is_empty());
        assert!(snapshot.data.is_empty());
    }

    #[test]
    fn test_snapshot_with_change_id() {
        let id = Uuid::new_v4();
        let snapshot = Snapshot::with_change_id(id);

        assert_eq!(snapshot.change_id, Some(id));
    }

    #[test]
    fn test_apply_insert_operation() {
        let generator = SnapshotGenerator::new();
        let mut snapshot = Snapshot::new();

        let change = create_test_change(
            "agent1",
            vec![],
            Operation::Insert {
                position: 0,
                content: "hello".to_string(),
            },
        );

        generator.apply_change(&mut snapshot, &change);

        assert_eq!(snapshot.content, "hello");
    }

    #[test]
    fn test_apply_multiple_inserts() {
        let generator = SnapshotGenerator::new();
        let mut snapshot = Snapshot::new();

        let change1 = create_test_change(
            "agent1",
            vec![],
            Operation::Insert {
                position: 0,
                content: "hello".to_string(),
            },
        );
        let change2 = create_test_change(
            "agent2",
            vec![change1.id],
            Operation::Insert {
                position: 5,
                content: " world".to_string(),
            },
        );

        generator.apply_change(&mut snapshot, &change1);
        generator.apply_change(&mut snapshot, &change2);

        assert_eq!(snapshot.content, "hello world");
    }

    #[test]
    fn test_apply_delete_operation() {
        let generator = SnapshotGenerator::new();
        let mut snapshot = Snapshot::new();

        // First insert some content
        snapshot.content = "hello world".to_string();

        let change = create_test_change(
            "agent1",
            vec![],
            Operation::Delete {
                position: 5,
                length: 6,
            },
        );

        generator.apply_change(&mut snapshot, &change);

        assert_eq!(snapshot.content, "hello");
    }

    #[test]
    fn test_apply_update_operation() {
        let generator = SnapshotGenerator::new();
        let mut snapshot = Snapshot::new();

        let change = create_test_change(
            "agent1",
            vec![],
            Operation::Update {
                path: "key".to_string(),
                value: serde_json::json!("value"),
            },
        );

        generator.apply_change(&mut snapshot, &change);

        assert_eq!(snapshot.get("key"), Some(&serde_json::json!("value")));
    }

    #[test]
    fn test_apply_file_operations() {
        let generator = SnapshotGenerator::new();
        let mut snapshot = Snapshot::new();

        let create = create_test_change(
            "agent1",
            vec![],
            Operation::Create {
                path: "/test.txt".to_string(),
                content: "content".to_string(),
            },
        );

        generator.apply_change(&mut snapshot, &create);

        assert_eq!(snapshot.get_file("/test.txt"), Some("content"));

        let move_op = create_test_change(
            "agent2",
            vec![create.id],
            Operation::Move {
                from: "/test.txt".to_string(),
                to: "/renamed.txt".to_string(),
            },
        );

        generator.apply_change(&mut snapshot, &move_op);

        assert_eq!(snapshot.get_file("/test.txt"), None);
        assert_eq!(snapshot.get_file("/renamed.txt"), Some("content"));
    }

    #[test]
    fn test_generate_from_change() {
        let mut generator = SnapshotGenerator::new();
        let mut changes = HashMap::new();

        let change1 = create_test_change(
            "agent1",
            vec![],
            Operation::Insert {
                position: 0,
                content: "hello".to_string(),
            },
        );
        let change2 = create_test_change(
            "agent2",
            vec![change1.id],
            Operation::Insert {
                position: 5,
                content: " world".to_string(),
            },
        );

        changes.insert(change1.id, change1.clone());
        changes.insert(change2.id, change2.clone());

        let snapshot = generator.generate_from_change(change2.id, &changes);

        assert_eq!(snapshot.content, "hello world");
        assert_eq!(snapshot.change_id, Some(change2.id));
    }

    #[test]
    fn test_generate_incremental() {
        let mut generator = SnapshotGenerator::new();
        let mut changes = HashMap::new();

        let change1 = create_test_change(
            "agent1",
            vec![],
            Operation::Insert {
                position: 0,
                content: "hello".to_string(),
            },
        );
        let change2 = create_test_change(
            "agent2",
            vec![change1.id],
            Operation::Insert {
                position: 5,
                content: " world".to_string(),
            },
        );

        changes.insert(change1.id, change1.clone());
        changes.insert(change2.id, change2.clone());

        // Generate base snapshot
        generator.generate_from_change(change1.id, &changes);

        // Generate incremental snapshot
        let snapshot = generator.generate_incremental(change1.id, change2.id, &changes);

        assert_eq!(snapshot.content, "hello world");
    }

    #[test]
    fn test_cache_functionality() {
        let mut generator = SnapshotGenerator::new();

        assert_eq!(generator.cache_size(), 0);

        generator.cache.insert(Uuid::new_v4(), Snapshot::new());

        assert_eq!(generator.cache_size(), 1);

        generator.clear_cache();

        assert_eq!(generator.cache_size(), 0);
    }

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = Snapshot::new();

        let serialized = serde_json::to_string(&snapshot).unwrap();
        let deserialized: Snapshot = serde_json::from_str(&serialized).unwrap();

        assert_eq!(snapshot.content, deserialized.content);
    }
}

