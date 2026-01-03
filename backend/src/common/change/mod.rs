//! # CRDT 变动文件系统
//!
//! 一个基于 CRDT 的分布式变动文件系统，支持多代理并发协作。
//!
//! ## 模块
//!
//! - [`change`] - 核心变动数据结构
//! - [`operation`] - 不同变动的操作类型
//! - [`version`] - 用于因果追踪的向量时钟（版本）
//! - [`thread`] - 线程管理（分叉、合并）
//! - [`merge`] - CRDT 合并引擎
//! - [`snapshot`] - 从变动序列生成快照

#[allow(clippy::module_inception)]
pub mod change;
pub mod merge;
pub mod operation;
pub mod snapshot;
pub mod thread;
pub mod version;

// 为了方便重新导出主要类型
pub use change::Change;
pub use merge::MergeEngine;
pub use operation::Operation;
pub use snapshot::Snapshot;
pub use thread::Thread;
pub use version::VectorClock;
