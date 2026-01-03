/// 智能体特定的意图。
#[derive(Debug, Clone)]
pub enum AgentIntent {
    /// 执行工具调用
    CallTool { name: String, args: String },
    /// 终止当前任务
    Abort,
}
