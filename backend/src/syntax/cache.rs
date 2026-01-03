use crate::common::meta::MetaNode;
use std::collections::HashMap;
use uuid::Uuid;

/// 管理增量解析的缓存
pub struct IncrementalCache {
    cache: HashMap<Uuid, MetaNode>,
}

impl Default for IncrementalCache {
    fn default() -> Self {
        Self::new()
    }
}

impl IncrementalCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// 更新缓存
    pub fn update(&mut self, file_id: Uuid, node: MetaNode) {
        self.cache.insert(file_id, node);
    }

    /// 获取缓存的节点
    pub fn get(&self, file_id: &Uuid) -> Option<&MetaNode> {
        self.cache.get(file_id)
    }

    /// 清除缓存
    pub fn invalidate(&mut self, file_id: &Uuid) {
        self.cache.remove(file_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incremental_cache() {
        let mut cache = IncrementalCache::new();
        let file_id = Uuid::new_v4();
        let node = MetaNode::module("test");

        cache.update(file_id, node.clone());
        assert_eq!(cache.get(&file_id).unwrap(), &node);

        cache.invalidate(&file_id);
        assert!(cache.get(&file_id).is_none());
    }
}
