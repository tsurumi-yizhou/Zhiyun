use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use uuid::Uuid;

/// 因果关系定义
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    /// 发生在前 (self < other)
    Before,
    /// 发生在后 (self > other)
    After,
    /// 相等 (self == other)
    Equal,
    /// 并发/冲突 (无法确定顺序)
    Concurrent,
}

/// 向量时钟，用于因果追踪
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct VectorClock {
    pub clocks: HashMap<Uuid, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self::default()
    }

    /// 增加指定节点的时钟计数
    pub fn increment(&mut self, node_id: Uuid) {
        let count = self.clocks.entry(node_id).or_insert(0);
        *count += 1;
    }

    /// 获取指定节点的时钟值
    pub fn get(&self, node_id: &Uuid) -> u64 {
        *self.clocks.get(node_id).unwrap_or(&0)
    }

    /// 合并另一个向量时钟（取各节点最大值）
    pub fn merge(&mut self, other: &VectorClock) {
        for (node_id, clock) in &other.clocks {
            let entry = self.clocks.entry(*node_id).or_insert(0);
            if *clock > *entry {
                *entry = *clock;
            }
        }
    }

    /// 比较两个向量时钟的因果关系
    pub fn compare(&self, other: &VectorClock) -> Relation {
        let mut self_has_greater = false;
        let mut other_has_greater = false;

        // 获取所有出现过的节点 ID
        let mut all_nodes: std::collections::HashSet<&Uuid> = self.clocks.keys().collect();
        all_nodes.extend(other.clocks.keys());

        for node_id in all_nodes {
            let self_val = self.get(node_id);
            let other_val = other.get(node_id);

            match self_val.cmp(&other_val) {
                Ordering::Greater => self_has_greater = true,
                Ordering::Less => other_has_greater = true,
                Ordering::Equal => {}
            }
        }

        match (self_has_greater, other_has_greater) {
            (true, true) => Relation::Concurrent,
            (true, false) => Relation::After,
            (false, true) => Relation::Before,
            (false, false) => Relation::Equal,
        }
    }

    /// 检查 self 是否在因果上先于 other
    pub fn is_before(&self, other: &VectorClock) -> bool {
        self.compare(other) == Relation::Before
    }

    /// 检查 self 是否在因果上后于 other
    pub fn is_after(&self, other: &VectorClock) -> bool {
        self.compare(other) == Relation::After
    }

    /// 检查 self 是否与 other 并发
    pub fn is_concurrent(&self, other: &VectorClock) -> bool {
        self.compare(other) == Relation::Concurrent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_clock_causality() {
        let node_a = Uuid::new_v4();
        let node_b = Uuid::new_v4();

        let mut v1 = VectorClock::new();
        v1.increment(node_a); // v1: {A:1}

        let mut v2 = v1.clone();
        v2.increment(node_a); // v2: {A:2}

        assert_eq!(v1.compare(&v2), Relation::Before);
        assert_eq!(v2.compare(&v1), Relation::After);

        let mut v3 = v1.clone();
        v3.increment(node_b); // v3: {A:1, B:1}

        // v2 和 v3 是并发的
        assert_eq!(v2.compare(&v3), Relation::Concurrent);
        assert!(v2.is_concurrent(&v3));

        let mut v4 = v2.clone();
        v4.merge(&v3); // v4: {A:2, B:1}
        assert_eq!(v2.compare(&v4), Relation::Before);
        assert_eq!(v3.compare(&v4), Relation::Before);
    }
}
