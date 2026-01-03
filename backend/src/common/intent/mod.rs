//! Intent 模块负责系统内各组件间的异步通信。
//!
//! 它基于“意图（Intent）”模式，将操作请求与其具体执行逻辑解耦。
//! 主要组件包括：
//! - `types`: 定义了系统中所有的意图类型及其分类。
//! - `handler`: 定义了处理意图的统一接口。
//! - `dispatcher`: 实现了意图的分发路由逻辑。
//!
//! 该模块的设计目标是支持智能体（Agent）和 UI 操作发出统一的意图，
//! 并通过异步等待机制确保操作执行的顺序性和一致性。

pub mod dispatcher;
pub mod handler;
pub mod traits;

// 重新导出常用类型，方便外部调用
pub use dispatcher::IntentDispatcher;
pub use handler::IntentHandler;
pub use traits::{AgentIntent, EditorIntent, IntentCategory, SystemIntent};
