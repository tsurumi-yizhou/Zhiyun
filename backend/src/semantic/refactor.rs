use crate::common::change::Change;
use crate::common::meta::MetaNode;
use uuid::Uuid;

/// 负责生成语义化的变更请求
pub struct RefactorEngine;

impl Default for RefactorEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RefactorEngine {
    pub fn new() -> Self {
        Self
    }

    /// 重命名符号
    pub fn rename(&self, _node_id: Uuid, _new_name: &str) -> anyhow::Result<Vec<Change>> {
        // Mock 逻辑：返回空变更列表
        Ok(vec![])
    }

    /// 提取函数
    pub fn extract_function(&self, _nodes: Vec<MetaNode>, _name: &str) -> anyhow::Result<Change> {
        // Mock 逻辑：报错
        Err(anyhow::anyhow!("Not implemented"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refactor_engine() {
        let engine = RefactorEngine::new();
        let id = Uuid::new_v4();
        let changes = engine.rename(id, "new_name").unwrap();
        assert!(changes.is_empty());
    }
}
