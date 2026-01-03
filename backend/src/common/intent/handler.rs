use crate::common::intent::traits::SystemIntent;
use anyhow::Result;
use async_trait::async_trait;

/// 意图处理器接口。
///
/// 任何希望处理系统意图的组件（如 `EditorSession`）都需要实现此 Trait。
/// 处理器应该是线程安全的（Send + Sync），以便在异步分发器中使用。
#[async_trait]
pub trait IntentHandler: Send + Sync {
    /// 执行意图处理逻辑。
    ///
    /// # 参数
    /// - `intent`: 待处理的系统意图包装。
    ///
    /// # 返回
    /// - `Result<()>`: 处理成功返回 `Ok(())`，否则返回具体错误。
    async fn handle(&self, intent: SystemIntent) -> Result<()>;
}
