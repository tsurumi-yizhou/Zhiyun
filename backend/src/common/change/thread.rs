use crate::common::change::Change;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

pub type ThreadId = Uuid;

/// 线程（分支）管理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: ThreadId,
    pub name: String,
    pub head_change_id: Option<Uuid>,
}

pub struct ThreadManager {
    threads: RwLock<HashMap<ThreadId, Thread>>,
    changes: RwLock<HashMap<Uuid, Change>>,
}

impl Default for ThreadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThreadManager {
    pub fn new() -> Self {
        let mut threads = HashMap::new();
        let main_thread_id = Uuid::new_v4();
        threads.insert(
            main_thread_id,
            Thread {
                id: main_thread_id,
                name: "main".to_string(),
                head_change_id: None,
            },
        );

        Self {
            threads: RwLock::new(threads),
            changes: RwLock::new(HashMap::new()),
        }
    }

    pub fn create_branch(&self, parent_id: ThreadId, name: &str) -> anyhow::Result<ThreadId> {
        let mut threads = self.threads.write().unwrap();
        let parent = threads
            .get(&parent_id)
            .ok_or_else(|| anyhow::anyhow!("Parent thread not found"))?;

        let new_id = Uuid::new_v4();
        let new_thread = Thread {
            id: new_id,
            name: name.to_string(),
            head_change_id: parent.head_change_id,
        };

        threads.insert(new_id, new_thread);
        Ok(new_id)
    }

    /// 提交一个新的 Change 到指定 Thread
    pub fn commit_change(&self, thread_id: ThreadId, change: Change) -> anyhow::Result<()> {
        let mut threads = self.threads.write().unwrap();
        let mut changes = self.changes.write().unwrap();

        let thread = threads
            .get_mut(&thread_id)
            .ok_or_else(|| anyhow::anyhow!("Thread not found"))?;

        // 校验 Change 的合法性（MVP 简化：仅校验 Hash）
        if !change.verify_hash() {
            return Err(anyhow::anyhow!("Invalid change hash"));
        }

        let change_id = change.id;
        changes.insert(change_id, change);
        thread.head_change_id = Some(change_id);

        Ok(())
    }

    pub fn get_thread(&self, id: ThreadId) -> Option<Thread> {
        self.threads.read().unwrap().get(&id).cloned()
    }

    pub fn get_change(&self, id: Uuid) -> Option<Change> {
        self.changes.read().unwrap().get(&id).cloned()
    }

    pub fn get_thread_id_by_name(&self, name: &str) -> Option<ThreadId> {
        self.threads
            .read()
            .unwrap()
            .values()
            .find(|t| t.name == name)
            .map(|t| t.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_manager_basic() {
        let manager = ThreadManager::new();
        let main_id = manager.get_thread_id_by_name("main").unwrap();
        let thread = manager.get_thread(main_id).unwrap();

        assert_eq!(thread.name, "main");
        assert!(thread.head_change_id.is_none());
    }
}
