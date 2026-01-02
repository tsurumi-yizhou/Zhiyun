//! # Merge module
//!
//! CRDT merge engine for combining changes from multiple threads.

use crate::common::change::change::{Change, ChangeId};
use crate::common::change::operation::Operation;
use std::collections::{HashMap, HashSet};

/// Result of a merge operation
#[derive(Debug, Clone, PartialEq)]
pub enum MergeResult {
    /// Changes merged successfully
    Success { changes: Vec<Change> },
    /// Conflict detected but resolved
    ConflictResolved {
        changes: Vec<Change>,
        conflicts: usize,
    },
    /// Merge failed due to unresolvable conflict
    Failed { reason: String },
}

/// Engine for merging changes using CRDT algorithms
#[derive(Debug, Clone)]
pub struct MergeEngine;

impl MergeEngine {
    /// Create a new merge engine
    pub fn new() -> Self {
        Self
    }

    /// Merge two sequences of changes
    ///
    /// Takes two lists of changes and merges them based on their causal relationships
    pub fn merge(&mut self, left: &[Change], right: &[Change]) -> MergeResult {
        let mut result = Vec::new();
        let mut conflicts = 0;

        // Index changes by ID for lookup
        let left_index: HashMap<ChangeId, &Change> = left.iter().map(|c| (c.id, c)).collect();
        let right_index: HashMap<ChangeId, &Change> = right.iter().map(|c| (c.id, c)).collect();

        // Track which changes we've included
        let mut included = HashSet::new();

        // Collect all unique change IDs
        let all_ids: HashSet<ChangeId> = left
            .iter()
            .map(|c| c.id)
            .chain(right.iter().map(|c| c.id))
            .collect();

        // Sort changes by dependencies using topological sort
        let sorted = self.topological_sort(&left_index, &right_index, &all_ids);

        for change_id in sorted {
            // Skip if already included
            if included.contains(&change_id) {
                continue;
            }

            // Check if this change exists in both branches (concurrent modification)
            let left_change = left_index.get(&change_id);
            let right_change = right_index.get(&change_id);

            match (left_change, right_change) {
                (Some(&lc), Some(_rc)) => {
                    // Same change in both - include once
                    result.push(lc.clone());
                    included.insert(change_id);
                }
                (Some(&lc), None) | (None, Some(&lc)) => {
                    // Change only in one branch - include
                    result.push(lc.clone());
                    included.insert(change_id);
                }
                (None, None) => {
                    // Shouldn't happen if we sorted correctly
                    continue;
                }
            }
        }

        // Check for operation-level conflicts
        for (i, change1) in result.iter().enumerate() {
            for change2 in result.iter().skip(i + 1) {
                if self.has_operation_conflict(change1, change2) {
                    conflicts += 1;
                }
            }
        }

        if conflicts > 0 {
            MergeResult::ConflictResolved {
                changes: result,
                conflicts,
            }
        } else {
            MergeResult::Success { changes: result }
        }
    }

    /// Find the lowest common ancestor of two changes
    pub fn find_common_ancestor(
        &self,
        change1: &Change,
        change2: &Change,
        all_changes: &HashMap<ChangeId, Change>,
    ) -> Option<ChangeId> {
        // Collect ancestors of change1
        let mut ancestors1 = HashSet::new();
        let mut to_visit = vec![change1.id];
        while let Some(id) = to_visit.pop() {
            if ancestors1.contains(&id) {
                continue;
            }
            ancestors1.insert(id);
            if let Some(c) = all_changes.get(&id) {
                to_visit.extend(c.parents.iter());
            }
        }

        // Find first ancestor of change2 that's also in ancestors1
        let mut to_visit = vec![change2.id];
        while let Some(id) = to_visit.pop() {
            if ancestors1.contains(&id) {
                return Some(id);
            }
            if let Some(c) = all_changes.get(&id) {
                to_visit.extend(c.parents.iter());
            }
        }

        None
    }

    /// Check if two operations conflict
    fn has_operation_conflict(&self, change1: &Change, change2: &Change) -> bool {
        match (&change1.operation, &change2.operation) {
            // Concurrent inserts at the same position conflict
            (Operation::Insert { position: p1, .. }, Operation::Insert { position: p2, .. }) => {
                p1 == p2
            }
            // Deletes that overlap conflict
            (
                Operation::Delete {
                    position: p1,
                    length: l1,
                },
                Operation::Delete {
                    position: p2,
                    length: l2,
                },
            ) => *p1 < *p2 + *l2 && *p2 < *p1 + *l1,
            // Updates to the same path conflict
            (Operation::Update { path: p1, .. }, Operation::Update { path: p2, .. }) => p1 == p2,
            // Other combinations are considered non-conflicting for now
            _ => false,
        }
    }

    /// Topological sort of changes based on dependencies
    fn topological_sort(
        &self,
        left_index: &HashMap<ChangeId, &Change>,
        right_index: &HashMap<ChangeId, &Change>,
        all_ids: &HashSet<ChangeId>,
    ) -> Vec<ChangeId> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();

        let all_changes: HashMap<ChangeId, &Change> = left_index
            .iter()
            .chain(right_index.iter())
            .map(|(&id, &change)| (id, change))
            .collect();

        // Build children map (reverse of parents)
        let mut children: HashMap<ChangeId, Vec<ChangeId>> = HashMap::new();
        let mut in_degree: HashMap<ChangeId, usize> = HashMap::new();

        for &id in all_ids {
            in_degree.insert(id, 0);
            children.insert(id, Vec::new());
        }

        for change in all_changes.values() {
            for &parent in &change.parents {
                if all_ids.contains(&parent) {
                    *in_degree.entry(change.id).or_insert(0) += 1;
                    children.entry(parent).or_default().push(change.id);
                }
            }
        }

        // Start with nodes that have no parents (in_degree = 0)
        let mut queue: Vec<ChangeId> = in_degree
            .iter()
            .filter(|(_, degree)| **degree == 0)
            .map(|(&id, _)| id)
            .collect();

        while let Some(id) = queue.pop() {
            if visited.contains(&id) {
                continue;
            }
            visited.insert(id);
            result.push(id);

            // Add children that now have all parents processed
            if let Some(child_list) = children.get(&id) {
                for &child in child_list {
                    if !visited.contains(&child) {
                        queue.push(child);
                    }
                }
            }
        }

        result
    }

    /// Create a merge change representing the merge operation itself
    pub fn create_merge_change(
        &self,
        author: String,
        left_head: ChangeId,
        right_head: ChangeId,
        merged_changes: Vec<ChangeId>,
    ) -> Change {
        Change::new(
            author,
            Operation::Custom {
                operation_type: "merge".to_string(),
                data: vec![
                    ("left_head".to_string(), serde_json::json!(left_head)),
                    ("right_head".to_string(), serde_json::json!(right_head)),
                    (
                        "merged_changes".to_string(),
                        serde_json::json!(merged_changes),
                    ),
                ]
                .into_iter()
                .collect(),
            },
            vec![left_head, right_head],
        )
    }
}

impl Default for MergeEngine {
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
    fn test_merge_empty_lists() {
        let mut engine = MergeEngine::new();
        let result = engine.merge(&[], &[]);

        assert!(matches!(result, MergeResult::Success { changes } if changes.is_empty()));
    }

    #[test]
    fn test_merge_no_conflict() {
        let mut engine = MergeEngine::new();

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
            vec![],
            Operation::Insert {
                position: 5,
                content: " world".to_string(),
            },
        );

        let result = engine.merge(&[change1.clone()], &[change2.clone()]);

        assert!(matches!(result, MergeResult::Success { .. }));
        if let MergeResult::Success { changes } = result {
            assert_eq!(changes.len(), 2);
        }
    }

    #[test]
    fn test_merge_with_conflict() {
        let mut engine = MergeEngine::new();

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
            vec![],
            Operation::Insert {
                position: 0,
                content: "world".to_string(),
            },
        );

        let result = engine.merge(&[change1], &[change2]);

        // Same position inserts should be reported as conflict
        assert!(matches!(result, MergeResult::ConflictResolved { .. }));
    }

    #[test]
    fn test_find_common_ancestor() {
        let engine = MergeEngine::new();

        let root = create_test_change(
            "agent1",
            vec![],
            Operation::Insert {
                position: 0,
                content: "root".to_string(),
            },
        );

        let branch1 = create_test_change(
            "agent2",
            vec![root.id],
            Operation::Insert {
                position: 4,
                content: "1".to_string(),
            },
        );

        let branch2 = create_test_change(
            "agent3",
            vec![root.id],
            Operation::Insert {
                position: 4,
                content: "2".to_string(),
            },
        );

        let mut all_changes = HashMap::new();
        all_changes.insert(root.id, root.clone());
        all_changes.insert(branch1.id, branch1.clone());
        all_changes.insert(branch2.id, branch2.clone());

        let ancestor = engine.find_common_ancestor(&branch1, &branch2, &all_changes);

        // The common ancestor should be the root
        assert_eq!(ancestor, Some(root.id));
    }

    #[test]
    fn test_create_merge_change() {
        let engine = MergeEngine::new();

        let left_head = Uuid::new_v4();
        let right_head = Uuid::new_v4();
        let merged_changes = vec![left_head, right_head];

        let merge_change = engine.create_merge_change(
            "merger".to_string(),
            left_head,
            right_head,
            merged_changes.clone(),
        );

        assert_eq!(merge_change.author, "merger");
        assert_eq!(merge_change.parents, vec![left_head, right_head]);

        if let Operation::Custom {
            operation_type,
            data,
        } = merge_change.operation
        {
            assert_eq!(operation_type, "merge");
            assert!(data.contains_key("left_head"));
            assert!(data.contains_key("right_head"));
        } else {
            panic!("Expected Custom operation");
        }
    }

    #[test]
    fn test_delete_conflict_detection() {
        let engine = MergeEngine::new();

        let change1 = create_test_change(
            "agent1",
            vec![],
            Operation::Delete {
                position: 0,
                length: 5,
            },
        );
        let change2 = create_test_change(
            "agent2",
            vec![],
            Operation::Delete {
                position: 3,
                length: 5,
            },
        );

        assert!(engine.has_operation_conflict(&change1, &change2));
    }

    #[test]
    fn test_update_conflict_detection() {
        let engine = MergeEngine::new();

        let change1 = create_test_change(
            "agent1",
            vec![],
            Operation::Update {
                path: "key".to_string(),
                value: serde_json::json!("value1"),
            },
        );
        let change2 = create_test_change(
            "agent2",
            vec![],
            Operation::Update {
                path: "key".to_string(),
                value: serde_json::json!("value2"),
            },
        );

        assert!(engine.has_operation_conflict(&change1, &change2));
    }

    #[test]
    fn test_topological_sort() {
        let engine = MergeEngine::new();

        let root = create_test_change(
            "agent1",
            vec![],
            Operation::Insert {
                position: 0,
                content: "a".to_string(),
            },
        );
        let child1 = create_test_change(
            "agent2",
            vec![root.id],
            Operation::Insert {
                position: 1,
                content: "b".to_string(),
            },
        );
        let child2 = create_test_change(
            "agent3",
            vec![child1.id],
            Operation::Insert {
                position: 2,
                content: "c".to_string(),
            },
        );

        let mut left_index = HashMap::new();
        left_index.insert(root.id, &root);
        left_index.insert(child1.id, &child1);
        left_index.insert(child2.id, &child2);

        let all_ids = vec![root.id, child1.id, child2.id].into_iter().collect();

        let sorted = engine.topological_sort(&left_index, &HashMap::new(), &all_ids);

        // Check that all changes are included
        assert_eq!(sorted.len(), 3);

        // Root should come before children
        let root_pos = sorted.iter().position(|&id| id == root.id).unwrap();
        let child1_pos = sorted.iter().position(|&id| id == child1.id).unwrap();
        let child2_pos = sorted.iter().position(|&id| id == child2.id).unwrap();

        assert!(root_pos < child1_pos, "root should come before child1");
        assert!(child1_pos < child2_pos, "child1 should come before child2");
    }
}

