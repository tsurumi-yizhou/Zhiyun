use crate::common::change::change::Change;
use crate::common::change::operation::Operation;
use crate::common::change::version::Relation;
use crate::common::meta::ast::MetaNode;
use uuid::Uuid;

/// CRDT 合并引擎
/// 采用因果排序 (Causal Ordering) 和 LWW (Last-Write-Wins) 策略
pub struct MergeEngine {}

impl Default for MergeEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MergeEngine {
    pub fn new() -> Self {
        Self {}
    }

    /// 对变动列表进行因果排序
    pub fn sort_changes(&self, changes: Vec<Change>) -> Vec<Change> {
        let mut sorted = changes;
        sorted.sort_by(|a, b| {
            match a.version.compare(&b.version) {
                Relation::Before => std::cmp::Ordering::Less,
                Relation::After => std::cmp::Ordering::Greater,
                Relation::Equal => a.timestamp.cmp(&b.timestamp),
                Relation::Concurrent => {
                    // 并发冲突：使用时间戳作为第一决胜局，ID 作为第二决胜局
                    match a.timestamp.cmp(&b.timestamp) {
                        std::cmp::Ordering::Equal => a.id.cmp(&b.id),
                        other => other,
                    }
                }
            }
        });
        sorted
    }

    /// 合并变动序列并投影到 MetaNode 树
    pub fn merge(&self, initial_state: MetaNode, changes: &[Change]) -> anyhow::Result<MetaNode> {
        let mut root = initial_state;
        let sorted_changes = self.sort_changes(changes.to_vec());

        for change in sorted_changes {
            for op in &change.operations {
                self.apply_operation(&mut root, op)?;
            }
        }

        Ok(root)
    }

    fn apply_operation(&self, root: &mut MetaNode, op: &Operation) -> anyhow::Result<()> {
        match op {
            Operation::Insert {
                parent_id,
                index,
                node,
            } => {
                if let Some(pid) = parent_id {
                    if let Some(parent) = self.find_node_mut(root, *pid) {
                        self.insert_into_node(parent, *index, node.clone())?;
                    }
                } else {
                    // 如果没有 parent_id，尝试插入到根节点的子列表中（如果根节点支持子节点）
                    self.insert_into_node(root, *index, node.clone())?;
                }
            }
            Operation::Update { node_id, new_node } => {
                if let Some(node) = self.find_node_mut(root, *node_id) {
                    *node = new_node.clone();
                }
            }
            Operation::Delete { node_id } => {
                self.delete_node(root, *node_id);
            }
            Operation::Move {
                node_id,
                new_parent_id,
                new_index,
            } => {
                // 简化实现：先删除再插入
                if let Some(node) = self.take_node(root, *node_id) {
                    if let Some(pid) = new_parent_id {
                        if let Some(parent) = self.find_node_mut(root, *pid) {
                            self.insert_into_node(parent, *new_index, node)?;
                        }
                    } else {
                        self.insert_into_node(root, *new_index, node)?;
                    }
                }
            }
            Operation::Mock { .. } => {}
            Operation::FileWrite { .. } => {} // 文件系统操作在 Meta AST 合并中暂不处理
            Operation::FileDelete { .. } => {}
        }
        Ok(())
    }

    fn find_node_mut<'a>(&self, current: &'a mut MetaNode, id: Uuid) -> Option<&'a mut MetaNode> {
        if current.id() == id {
            return Some(current);
        }

        match current {
            MetaNode::Module { children, .. } => {
                for child in children {
                    if let Some(found) = self.find_node_mut(child, id) {
                        return Some(found);
                    }
                }
            }
            MetaNode::Function { params, body, .. } => {
                for param in params {
                    if let Some(found) = self.find_node_mut(param, id) {
                        return Some(found);
                    }
                }
                if let Some(b) = body
                    && let Some(found) = self.find_node_mut(b, id)
                {
                    return Some(found);
                }
            }
            MetaNode::Class { members, .. } => {
                for member in members {
                    if let Some(found) = self.find_node_mut(member, id) {
                        return Some(found);
                    }
                }
            }
            MetaNode::Block { statements, .. } => {
                for stmt in statements {
                    if let Some(found) = self.find_node_mut(stmt, id) {
                        return Some(found);
                    }
                }
            }
            _ => {}
        }
        None
    }

    fn insert_into_node(
        &self,
        parent: &mut MetaNode,
        index: usize,
        node: MetaNode,
    ) -> anyhow::Result<()> {
        match parent {
            MetaNode::Module { children, .. } => {
                let idx = index.min(children.len());
                children.insert(idx, node);
            }
            MetaNode::Block { statements, .. } => {
                let idx = index.min(statements.len());
                statements.insert(idx, node);
            }
            MetaNode::Class { members, .. } => {
                let idx = index.min(members.len());
                members.insert(idx, node);
            }
            _ => return Err(anyhow::anyhow!("Node does not support children")),
        }
        Ok(())
    }

    fn delete_node(&self, parent: &mut MetaNode, id: Uuid) -> bool {
        match parent {
            MetaNode::Module { children, .. } => {
                if let Some(pos) = children.iter().position(|c| c.id() == id) {
                    children.remove(pos);
                    return true;
                }
                for child in children {
                    if self.delete_node(child, id) {
                        return true;
                    }
                }
            }
            MetaNode::Block { statements, .. } => {
                if let Some(pos) = statements.iter().position(|c| c.id() == id) {
                    statements.remove(pos);
                    return true;
                }
                for stmt in statements {
                    if self.delete_node(stmt, id) {
                        return true;
                    }
                }
            }
            MetaNode::Class { members, .. } => {
                if let Some(pos) = members.iter().position(|c| c.id() == id) {
                    members.remove(pos);
                    return true;
                }
                for member in members {
                    if self.delete_node(member, id) {
                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn take_node(&self, parent: &mut MetaNode, id: Uuid) -> Option<MetaNode> {
        match parent {
            MetaNode::Module { children, .. } => {
                if let Some(pos) = children.iter().position(|c| c.id() == id) {
                    return Some(children.remove(pos));
                }
                for child in children {
                    if let Some(found) = self.take_node(child, id) {
                        return Some(found);
                    }
                }
            }
            MetaNode::Block { statements, .. } => {
                if let Some(pos) = statements.iter().position(|c| c.id() == id) {
                    return Some(statements.remove(pos));
                }
                for stmt in statements {
                    if let Some(found) = self.take_node(stmt, id) {
                        return Some(found);
                    }
                }
            }
            MetaNode::Class { members, .. } => {
                if let Some(pos) = members.iter().position(|c| c.id() == id) {
                    return Some(members.remove(pos));
                }
                for member in members {
                    if let Some(found) = self.take_node(member, id) {
                        return Some(found);
                    }
                }
            }
            _ => {}
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::change::version::VectorClock;
    use uuid::Uuid;

    #[test]
    fn test_merge_insert_update() {
        let engine = MergeEngine::new();
        let user_id = Uuid::new_v4();
        let root = MetaNode::module("root");
        let root_id = root.id();

        // 1. Insert a node
        let node = MetaNode::identifier("var_a");
        let node_id = node.id();
        let mut v1 = VectorClock::new();
        v1.increment(user_id);
        let c1 = Change::new(
            user_id,
            vec![Operation::insert(Some(root_id), 0, node)],
            v1.clone(),
            vec![],
        );

        // 2. 更新节点
        let mut v2 = v1.clone();
        v2.increment(user_id);
        let updated_node = MetaNode::identifier("var_b");
        // 确保更新时 ID 保持不变
        let updated_node = match updated_node {
            MetaNode::Identifier { name, scope_id, .. } => MetaNode::Identifier {
                id: node_id,
                name,
                scope_id,
            },
            other => other,
        };
        let c2 = Change::new(
            user_id,
            vec![Operation::update(node_id, updated_node)],
            v2.clone(),
            vec![c1.id],
        );

        let result = engine.merge(root, &[c1, c2]).unwrap();

        if let MetaNode::Module { children, .. } = result {
            assert_eq!(children.len(), 1);
            if let MetaNode::Identifier { name, .. } = &children[0] {
                assert_eq!(name, "var_b");
            } else {
                panic!("Expected Identifier");
            }
        }
    }

    #[test]
    fn test_merge_concurrent_conflicts() {
        let engine = MergeEngine::new();
        let user_a = Uuid::new_v4();
        let user_b = Uuid::new_v4();
        let root = MetaNode::module("root");
        let root_id = root.id();

        // 用户 A 和用户 B 同时插入一个节点
        let node_a = MetaNode::identifier("node_a");
        let mut v_a = VectorClock::new();
        v_a.increment(user_a);
        let c_a = Change::new(
            user_a,
            vec![Operation::insert(Some(root_id), 0, node_a)],
            v_a,
            vec![],
        );

        let node_b = MetaNode::identifier("node_b");
        let mut v_b = VectorClock::new();
        v_b.increment(user_b);
        // 用户 B 的变动具有较晚的时间戳
        let mut c_b = Change::new(
            user_b,
            vec![Operation::insert(Some(root_id), 0, node_b)],
            v_b,
            vec![],
        );
        c_b.timestamp = c_a.timestamp + chrono::Duration::seconds(1);

        // 无论输入顺序如何，合并都应该是确定性的
        let result1 = engine
            .merge(root.clone(), &[c_a.clone(), c_b.clone()])
            .unwrap();
        let result2 = engine
            .merge(root.clone(), &[c_b.clone(), c_a.clone()])
            .unwrap();

        assert_eq!(result1, result2);

        if let MetaNode::Module { children, .. } = result1 {
            assert_eq!(children.len(), 2);
            // 根据 LWW 和我们的排序，c_b（较晚的时间戳）应该在 c_a 之后应用
            // 1. c_a 在索引 0 处插入 node_a -> [node_a]
            // 2. c_b 在索引 0 处插入 node_b -> [node_b, node_a]
            if let MetaNode::Identifier { name, .. } = &children[0] {
                assert_eq!(name, "node_b");
            }
            if let MetaNode::Identifier { name, .. } = &children[1] {
                assert_eq!(name, "node_a");
            }
        }
    }
}
