use crate::common::change::version::VectorClock;
use crate::common::meta::ast::MetaNode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 快照数据结构，表示某一时刻的完整状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: Uuid,
    pub root: MetaNode,
    pub version: VectorClock,
}

impl Snapshot {
    /// 创建一个新的快照
    pub fn new(root: MetaNode, version: VectorClock) -> Self {
        Self {
            id: Uuid::new_v4(),
            root,
            version,
        }
    }

    /// 从快照中获取指定 ID 的节点
    pub fn find_node(&self, id: Uuid) -> Option<&MetaNode> {
        self.find_node_recursive(&self.root, id)
    }

    fn find_node_recursive<'a>(&self, current: &'a MetaNode, id: Uuid) -> Option<&'a MetaNode> {
        if current.id() == id {
            return Some(current);
        }

        match current {
            MetaNode::Module { children, .. } => {
                for child in children {
                    if let Some(found) = self.find_node_recursive(child, id) {
                        return Some(found);
                    }
                }
            }
            MetaNode::Function { params, body, .. } => {
                for param in params {
                    if let Some(found) = self.find_node_recursive(param, id) {
                        return Some(found);
                    }
                }
                if let Some(b) = body {
                    if let Some(found) = self.find_node_recursive(b, id) {
                        return Some(found);
                    }
                }
            }
            MetaNode::Class { members, .. } => {
                for member in members {
                    if let Some(found) = self.find_node_recursive(member, id) {
                        return Some(found);
                    }
                }
            }
            MetaNode::Block { statements, .. } => {
                for stmt in statements {
                    if let Some(found) = self.find_node_recursive(stmt, id) {
                        return Some(found);
                    }
                }
            }
            _ => {}
        }
        None
    }

    /// Mock 创建快照
    pub fn mock(root: MetaNode) -> Self {
        Self::new(root, VectorClock::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::meta::ast::MetaNode;

    #[test]
    fn test_snapshot_find_node() {
        let node = MetaNode::identifier("test");
        let node_id = node.id();
        let mut root = MetaNode::module("root");
        if let MetaNode::Module { children, .. } = &mut root {
            children.push(node);
        }
        
        let snapshot = Snapshot::mock(root);
        let found = snapshot.find_node(node_id).unwrap();
        
        if let MetaNode::Identifier { name, .. } = found {
            assert_eq!(name, "test");
        } else {
            panic!("Expected Identifier");
        }
    }
}
