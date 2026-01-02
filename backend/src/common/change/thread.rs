use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type ThreadId = Uuid;

/// 线程（分支）管理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: ThreadId,
    pub name: String,
    pub head_change_id: Uuid,
}

impl Thread {
    /// Mock 创建新线程
    pub fn mock(name: &str, head_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            head_change_id: head_id,
        }
    }
}

pub struct ThreadManager {
    // Mock
}

impl ThreadManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create_branch(&self, _parent_id: ThreadId, _name: &str) -> anyhow::Result<ThreadId> {
        // Mock
        Ok(Uuid::new_v4())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_mock() {
        let head_id = Uuid::new_v4();
        let thread = Thread::mock("main", head_id);

        assert_eq!(thread.name, "main");
        assert_eq!(thread.head_change_id, head_id);
    }
}
