use crate::common::meta::ast::MetaNode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 操作类型定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Operation {
    /// 插入节点
    Insert {
        parent_id: Option<Uuid>,
        index: usize,
        node: MetaNode,
    },
    /// 更新节点
    Update { node_id: Uuid, new_node: MetaNode },
    /// 删除节点
    Delete { node_id: Uuid },
    /// 移动节点
    Move {
        node_id: Uuid,
        new_parent_id: Option<Uuid>,
        new_index: usize,
    },
    /// 自定义 Mock 操作
    Mock { kind: String, data: String },
}

impl Operation {
    /// 创建插入操作
    pub fn insert(parent_id: Option<Uuid>, index: usize, node: MetaNode) -> Self {
        Operation::Insert { parent_id, index, node }
    }

    /// 创建更新操作
    pub fn update(node_id: Uuid, new_node: MetaNode) -> Self {
        Operation::Update { node_id, new_node }
    }

    /// 创建删除操作
    pub fn delete(node_id: Uuid) -> Self {
        Operation::Delete { node_id }
    }

    /// 创建移动操作
    pub fn r#move(node_id: Uuid, new_parent_id: Option<Uuid>, new_index: usize) -> Self {
        Operation::Move { node_id, new_parent_id, new_index }
    }

    pub fn mock(kind: &str, data: &str) -> Self {
        Operation::Mock {
            kind: kind.to_string(),
            data: data.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_mock() {
        let op = Operation::mock("test", "data");
        if let Operation::Mock { kind, data } = op {
            assert_eq!(kind, "test");
            assert_eq!(data, "data");
        } else {
            panic!("Expected Mock operation");
        }
    }
}
