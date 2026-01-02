//! # Version module
//!
//! Implements vector clocks (versions) for tracking causality between changes.

pub use crate::common::change::change::AgentId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A vector clock tracks the causal history of changes across agents
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorClock {
    /// Map from agent ID to their logical timestamp
    clock: HashMap<AgentId, u64>,
}

impl VectorClock {
    /// Create a new empty vector clock
    pub fn new() -> Self {
        Self {
            clock: HashMap::new(),
        }
    }

    /// Increment the counter for a specific agent
    pub fn increment(&mut self, agent: &AgentId) {
        *self.clock.entry(agent.clone()).or_insert(0) += 1;
    }

    /// Set the counter for a specific agent
    pub fn set(&mut self, agent: &AgentId, value: u64) {
        self.clock.insert(agent.clone(), value);
    }

    /// Get the counter for a specific agent
    pub fn get(&self, agent: &AgentId) -> Option<&u64> {
        self.clock.get(agent)
    }

    /// Merge this vector clock with another (element-wise max)
    pub fn merge(&mut self, other: &VectorClock) {
        for (agent, &value) in &other.clock {
            let entry = self.clock.entry(agent.clone()).or_insert(0);
            *entry = (*entry).max(value);
        }
    }

    /// Compare two vector clocks to determine causal relationship
    ///
    /// Returns `None` if the clocks are concurrent (incomparable)
    /// Returns `Some(Ordering::Less)` if self < other (self happened before other)
    /// Returns `Some(Ordering::Greater)` if self > other (other happened before self)
    /// Returns `Some(Ordering::Equal)` if self == other (same causal history)
    pub fn compare(&self, other: &VectorClock) -> Option<std::cmp::Ordering> {
        let mut less = false;
        let mut greater = false;

        let all_agents: std::collections::HashSet<_> =
            self.clock.keys().chain(other.clock.keys()).collect();

        for agent in all_agents {
            let self_val = self.get(agent).copied().unwrap_or(0);
            let other_val = other.get(agent).copied().unwrap_or(0);

            if self_val < other_val {
                less = true;
            } else if self_val > other_val {
                greater = true;
            }

            // If both are true, they're concurrent
            if less && greater {
                return None;
            }
        }

        match (less, greater) {
            (true, false) => Some(std::cmp::Ordering::Less),
            (false, true) => Some(std::cmp::Ordering::Greater),
            (false, false) => Some(std::cmp::Ordering::Equal),
            (true, true) => unreachable!(),
        }
    }

    /// Check if this vector clock dominates another (self >= other)
    pub fn dominates(&self, other: &VectorClock) -> bool {
        matches!(
            self.compare(other),
            Some(std::cmp::Ordering::Greater) | Some(std::cmp::Ordering::Equal)
        )
    }

    /// Check if two vector clocks are concurrent
    pub fn is_concurrent_with(&self, other: &VectorClock) -> bool {
        self.compare(other).is_none()
    }

    /// Get the number of agents tracked
    pub fn len(&self) -> usize {
        self.clock.len()
    }

    /// Check if the vector clock is empty
    pub fn is_empty(&self) -> bool {
        self.clock.is_empty()
    }

    /// Get an iterator over the clock entries
    pub fn iter(&self) -> impl Iterator<Item = (&AgentId, &u64)> {
        self.clock.iter()
    }
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_clock_creation() {
        let vc = VectorClock::new();
        assert!(vc.is_empty());
    }

    #[test]
    fn test_increment() {
        let mut vc = VectorClock::new();
        vc.increment(&"agent1".to_string());

        assert_eq!(vc.get(&"agent1".to_string()), Some(&1));
        assert_eq!(vc.len(), 1);
    }

    #[test]
    fn test_multiple_increments() {
        let mut vc = VectorClock::new();
        vc.increment(&"agent1".to_string());
        vc.increment(&"agent1".to_string());
        vc.increment(&"agent2".to_string());

        assert_eq!(vc.get(&"agent1".to_string()), Some(&2));
        assert_eq!(vc.get(&"agent2".to_string()), Some(&1));
    }

    #[test]
    fn test_set() {
        let mut vc = VectorClock::new();
        vc.set(&"agent1".to_string(), 10);

        assert_eq!(vc.get(&"agent1".to_string()), Some(&10));
    }

    #[test]
    fn test_merge() {
        let mut vc1 = VectorClock::new();
        vc1.set(&"agent1".to_string(), 5);
        vc1.set(&"agent2".to_string(), 3);

        let mut vc2 = VectorClock::new();
        vc2.set(&"agent1".to_string(), 3);
        vc2.set(&"agent2".to_string(), 7);
        vc2.set(&"agent3".to_string(), 1);

        vc1.merge(&vc2);

        assert_eq!(vc1.get(&"agent1".to_string()), Some(&5)); // max(5, 3)
        assert_eq!(vc1.get(&"agent2".to_string()), Some(&7)); // max(3, 7)
        assert_eq!(vc1.get(&"agent3".to_string()), Some(&1)); // from vc2
    }

    #[test]
    fn test_compare_equal() {
        let mut vc1 = VectorClock::new();
        let mut vc2 = VectorClock::new();

        vc1.set(&"agent1".to_string(), 5);
        vc2.set(&"agent1".to_string(), 5);

        assert_eq!(vc1.compare(&vc2), Some(std::cmp::Ordering::Equal));
    }

    #[test]
    fn test_compare_less() {
        let mut vc1 = VectorClock::new();
        let mut vc2 = VectorClock::new();

        vc1.set(&"agent1".to_string(), 3);
        vc2.set(&"agent1".to_string(), 5);

        assert_eq!(vc1.compare(&vc2), Some(std::cmp::Ordering::Less));
    }

    #[test]
    fn test_compare_greater() {
        let mut vc1 = VectorClock::new();
        let mut vc2 = VectorClock::new();

        vc1.set(&"agent1".to_string(), 5);
        vc2.set(&"agent1".to_string(), 3);

        assert_eq!(vc1.compare(&vc2), Some(std::cmp::Ordering::Greater));
    }

    #[test]
    fn test_compare_concurrent() {
        let mut vc1 = VectorClock::new();
        let mut vc2 = VectorClock::new();

        vc1.set(&"agent1".to_string(), 5);
        vc2.set(&"agent2".to_string(), 5);

        // Concurrent: agent1 > agent2 in vc1, but agent2 > agent1 in vc2
        assert!(vc1.is_concurrent_with(&vc2));
        assert_eq!(vc1.compare(&vc2), None);
    }

    #[test]
    fn test_dominates() {
        let mut vc1 = VectorClock::new();
        let mut vc2 = VectorClock::new();

        vc1.set(&"agent1".to_string(), 5);
        vc1.set(&"agent2".to_string(), 3);

        vc2.set(&"agent1".to_string(), 3);
        vc2.set(&"agent2".to_string(), 2);

        assert!(vc1.dominates(&vc2));
        assert!(!vc2.dominates(&vc1));
    }

    #[test]
    fn test_serialization() {
        let mut vc = VectorClock::new();
        vc.set(&"agent1".to_string(), 5);

        let serialized = serde_json::to_string(&vc).unwrap();
        let deserialized: VectorClock = serde_json::from_str(&serialized).unwrap();

        assert_eq!(vc, deserialized);
    }
}

