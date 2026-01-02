use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use crate::common::change::operation::Operation;
use crate::common::change::version::VectorClock;

/// 变动数据结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Change {
    /// 变动唯一 ID
    pub id: Uuid,
    /// 作者 ID
    pub author_id: Uuid,
    /// 变动发生的时间戳
    pub timestamp: DateTime<Utc>,
    /// 包含的操作列表
    pub operations: Vec<Operation>,
    /// 变动发生时的向量时钟（用于因果排序）
    pub version: VectorClock,
    /// 父变动 ID 列表（支持 DAG 结构）
    pub parents: Vec<Uuid>,
    /// 内容哈希，用于完整性校验
    pub hash: String,
}

impl Change {
    /// 创建一个新的变动
    pub fn new(
        author_id: Uuid,
        operations: Vec<Operation>,
        version: VectorClock,
        parents: Vec<Uuid>,
    ) -> Self {
        let mut change = Self {
            id: Uuid::new_v4(),
            author_id,
            timestamp: Utc::now(),
            operations,
            version,
            parents,
            hash: String::new(),
        };
        change.hash = change.calculate_hash();
        change
    }

    /// 计算变动的哈希值
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        
        // 序列化关键字段进行哈希
        hasher.update(self.author_id.as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        
        // 序列化操作列表
        if let Ok(ops_json) = serde_json::to_string(&self.operations) {
            hasher.update(ops_json.as_bytes());
        }
        
        // 序列化版本和父节点
        if let Ok(version_json) = serde_json::to_string(&self.version) {
            hasher.update(version_json.as_bytes());
        }
        for parent in &self.parents {
            hasher.update(parent.as_bytes());
        }

        format!("{:x}", hasher.finalize())
    }

    /// 校验变动哈希是否正确
    pub fn verify_hash(&self) -> bool {
        self.hash == self.calculate_hash()
    }

    /// Mock 创建一个新的变动
    pub fn mock(author_id: Uuid, operations: Vec<Operation>) -> Self {
        Self::new(author_id, operations, VectorClock::new(), Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::change::operation::Operation;

    #[test]
    fn test_change_hash_verification() {
        let author_id = Uuid::new_v4();
        let op = Operation::mock("test", "data");
        let change = Change::new(author_id, vec![op], VectorClock::new(), Vec::new());
        
        assert!(change.verify_hash());
        
        // 篡改数据应导致校验失败
        let mut tampered = change.clone();
        tampered.hash = "invalid_hash".to_string();
        assert!(!tampered.verify_hash());
    }
}
