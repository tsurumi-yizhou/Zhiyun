use std::collections::HashMap;

/// 维护项目的高层架构关系
pub struct KnowledgeGraph {
    // Mock 图结构：节点 -> 邻接列表
    edges: HashMap<String, Vec<String>>,
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    /// 添加关系
    pub fn add_relation(&mut self, from: &str, to: &str) {
        self.edges
            .entry(from.to_string())
            .or_default()
            .push(to.to_string());
    }

    /// 获取受影响的节点
    pub fn get_affected(&self, node: &str) -> Vec<String> {
        self.edges.get(node).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_graph() {
        let mut graph = KnowledgeGraph::new();
        graph.add_relation("Auth", "User");
        let affected = graph.get_affected("Auth");
        assert_eq!(affected, vec!["User".to_string()]);
    }
}
