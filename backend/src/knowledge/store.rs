use std::collections::HashMap;

/// 存储代码片段、文档和注释的嵌入向量
pub struct VectorStore {
    // Mock 存储：内容哈希 -> 向量
    store: HashMap<String, Vec<f32>>,
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorStore {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    /// 存储向量
    pub fn add(&mut self, id: &str, vector: Vec<f32>) {
        self.store.insert(id.to_string(), vector);
    }

    /// 搜索相似向量
    pub fn search(&self, _query: &[f32], _limit: usize) -> Vec<String> {
        // Mock 逻辑：返回前 limit 个 ID
        self.store.keys().take(_limit).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_store() {
        let mut store = VectorStore::new();
        store.add("doc1", vec![0.1, 0.2]);
        let results = store.search(&[0.1, 0.2], 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "doc1");
    }
}
