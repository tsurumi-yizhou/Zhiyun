pub use crate::agent::AgentIntent;
pub use crate::editor::EditorIntent;

/// 意图类别，用于路由分发。
///
/// 每个类别对应系统中一个主要的逻辑模块。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntentCategory {
    /// 编辑器相关的操作意图
    Editor,
    /// 智能体（Agent）相关的操作意图
    Agent,
}

/// 系统统一意图包装器。
///
/// 它是 `IntentDispatcher` 处理的原子单位，封装了各模块的具体意图。
#[derive(Debug, Clone)]
pub enum SystemIntent {
    /// 编辑器意图分支
    Editor(EditorIntent),
    /// 智能体意图分支
    Agent(AgentIntent),
}

impl SystemIntent {
    /// 获取该意图所属的类别，用于分发路由。
    pub fn category(&self) -> IntentCategory {
        match self {
            SystemIntent::Editor(_) => IntentCategory::Editor,
            SystemIntent::Agent(_) => IntentCategory::Agent,
        }
    }
}
