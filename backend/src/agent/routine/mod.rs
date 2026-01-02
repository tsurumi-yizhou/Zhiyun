use crate::common::change::thread::ThreadId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type RoutineId = Uuid;

/// Agent 运行时的“进程”抽象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Routine {
    pub id: RoutineId,
    pub parent: Option<RoutineId>,
    pub active_thread: ThreadId,
    pub status: RoutineStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoutineStatus {
    Running,
    Paused,
    Completed,
    Failed(String),
}

impl Routine {
    pub fn new(active_thread: ThreadId) -> Self {
        Self {
            id: Uuid::new_v4(),
            parent: None,
            active_thread,
            status: RoutineStatus::Running,
        }
    }
}

pub mod executor;
