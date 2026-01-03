use crate::common::change::Change;
use crate::common::change::thread::ThreadId;
use anyhow::Result;

/// 协调 Routine 产生的变更合并到对应的 Thread
pub struct MergerBridge;

impl Default for MergerBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl MergerBridge {
    pub fn new() -> Self {
        Self
    }

    /// 提议合并变更
    pub async fn propose_merge(
        &self,
        _from_thread: ThreadId,
        _to_thread: ThreadId,
        _changes: Vec<Change>,
    ) -> Result<()> {
        // Mock 逻辑：始终成功
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_merger_bridge() {
        let bridge = MergerBridge::new();
        let t1 = Uuid::new_v4();
        let t2 = Uuid::new_v4();
        assert!(bridge.propose_merge(t1, t2, vec![]).await.is_ok());
    }
}
