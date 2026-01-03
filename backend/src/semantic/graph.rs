use crate::common::meta::MetaNode;
use std::collections::HashMap;
use uuid::Uuid;

/// 从元 AST 提取语义关系并填充图谱
pub struct GraphBuilder {
    // 模拟图谱数据结构
    nodes: HashMap<Uuid, MetaNode>,
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// 构建图谱
    pub fn build(&mut self, node: MetaNode) {
        // Mock 构建逻辑：简单存储节点
        self.nodes.insert(node.id(), node);
    }

    /// 获取节点数量
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_builder() {
        let mut builder = GraphBuilder::new();
        let node = MetaNode::module("test");
        builder.build(node);
        assert_eq!(builder.node_count(), 1);
    }
}
