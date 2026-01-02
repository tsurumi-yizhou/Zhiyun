use crate::agent::routine::Routine;
use crate::common::change::thread::ThreadManager;
use anyhow::Result;
use std::sync::Arc;

pub struct RoutineExecutor {
    thread_manager: Arc<ThreadManager>,
}

impl RoutineExecutor {
    pub fn new(thread_manager: Arc<ThreadManager>) -> Self {
        Self { thread_manager }
    }

    /// 元调用：分支出一个新的子 Routine
    pub fn fork(&self, parent: &Routine, name: &str) -> Result<Routine> {
        // 1. 为子 Routine 创建新的 Thread 分支
        let child_thread = self
            .thread_manager
            .create_branch(parent.active_thread, name)?;

        // 2. 创建子 Routine 对象
        let mut child = Routine::new(child_thread);
        child.parent = Some(parent.id);

        Ok(child)
    }
}
