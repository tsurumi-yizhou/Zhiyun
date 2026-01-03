use crate::agent::Routine;
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

    pub fn fork(&self, parent: &Routine, name: &str) -> Result<Routine> {
        let child_thread = self
            .thread_manager
            .create_branch(parent.active_thread, name)?;

        let mut child = Routine::new(child_thread);
        child.parent = Some(parent.id);

        Ok(child)
    }
}
