# Change 模块 (CRDT Core)

`change` 模块实现了 Zhiyun 的核心哲学——**“变更核心 (Change-Centric)”**。它基于 CRDT (Conflict-free Replicated Data Types) 确保多方协作下的最终一致性。

## 核心组件

- [operation.rs](./operation.rs): 定义语言无关的原子操作（如 `InsertNode`, `RenameSymbol`）。
- [thread.rs](./thread.rs): 变更主线的抽象，代表一个版本化的更改序列。
- [merge.rs](./merge.rs): `MergeEngine` 实现了三路合并算法。
- [version.rs](./version.rs): 版本管理与矢量时钟逻辑。
- [snapshot.rs](./snapshot.rs): 状态快照，用于加速状态恢复。
- [change.rs](./change.rs): 单个变更包的定义。

## 关键概念

- **Thread (线程)**: 不是操作系统线程，而是逻辑上的变更分支。
- **Atomic Operations**: 禁止直接文本操作，所有修改必须封装为语义化的原子操作。
