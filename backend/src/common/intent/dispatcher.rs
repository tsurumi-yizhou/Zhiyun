use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::common::intent::handler::IntentHandler;
use crate::common::intent::traits::{IntentCategory, SystemIntent};

/// 意图分发器。
///
/// 负责维护 `IntentCategory` 到 `IntentHandler` 的映射关系，
/// 并提供统一的 `dispatch` 接口将意图路由到正确的处理器。
///
/// 该分发器使用异步锁（RwLock）以支持并发的意图分发和处理器注册。
pub struct IntentDispatcher {
    /// 处理器注册表，按类别路由。
    handlers: RwLock<HashMap<IntentCategory, Arc<dyn IntentHandler>>>,
}

impl Default for IntentDispatcher {
    /// 创建一个新的默认分发器。
    fn default() -> Self {
        Self::new()
    }
}

impl IntentDispatcher {
    /// 创建一个新的分发器实例。
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
        }
    }

    /// 注册一个意图处理器。
    ///
    /// # 参数
    /// - `category`: 处理器负责的意图类别。
    /// - `handler`: 实现了 `IntentHandler` 的处理器实例，包装在 `Arc` 中以支持共享。
    pub async fn register(&self, category: IntentCategory, handler: Arc<dyn IntentHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers.insert(category, handler);
    }

    /// 分发一个系统意图。
    ///
    /// 此方法会查找与意图类别匹配的处理器，并异步调用其 `handle` 方法。
    /// 该过程会阻塞直到处理器执行完成，从而实现请求-响应模式的异步流。
    ///
    /// # 参数
    /// - `intent`: 要分发的系统意图。
    ///
    /// # 返回
    /// - `Result<()>`: 分发及处理成功返回 `Ok(())`，若无对应处理器或处理出错则返回 `Err`。
    pub async fn dispatch(&self, intent: SystemIntent) -> Result<()> {
        let category = intent.category();
        let handler = {
            let handlers = self.handlers.read().await;
            handlers.get(&category).cloned()
        };

        if let Some(handler) = handler {
            // 在当前异步上下文中直接 await 处理器的执行，等待其返回结果
            handler.handle(intent).await
        } else {
            Err(anyhow::anyhow!(
                "No handler registered for category: {:?}",
                category
            ))
        }
    }
}
