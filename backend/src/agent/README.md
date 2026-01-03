# Agent 运行时 (Agent Runtime)

`agent` 模块负责管理智能 Agent 的生命周期、任务规划和 Routine 执行，它是系统的“智能大脑”。

## 核心功能

- **Routine 管理**: 每一个 Agent 任务运行在一个 Routine 中，拥有独立的对话历史和任务状态。
- **元调用 (Fork)**: Routine 能够派生子 Routine 来递归分解复杂任务。
- **任务规划与执行**: 
    - **Planner**: 使用 LLM 将用户意图分解为一系列 Skill 调用。
    - **Executor**: 顺序或并行执行计划中的步骤，并根据执行结果动态调整后续计划。

## 核心组件

- [manager.rs](./manager.rs): `RoutineManager` 跟踪所有活跃的 Routine 及其层级关系。
- [context.rs](./context.rs): `ContextManager` 负责对话上下文的智能压缩与窗口管理。
- [bridge.rs](./bridge.rs): `MergerBridge` 协调 Routine 产生的变更合并到对应的 Thread。
- [planner.rs](./planner.rs): 任务规划逻辑。
- [executor.rs](./executor.rs): 任务执行引擎。
- [routine.rs](./routine.rs): Routine 的具体实现。

## 设计原则

- **无需沙箱**: 依赖 Thread 分支机制实现修改的隔离与安全性。
- **异步与流式**: Agent 的思考和执行过程应该是异步的，支持流式结果输出。

## 架构设计细节

### 1. Routine 运行模型
- **Routine (进程)**: 每一个 Agent 任务运行在一个 Routine 中，拥有独立的对话历史和任务状态。
- **元调用 (Fork)**: Routine 能够派生子 Routine 来递归分解复杂任务。

### 2. 任务执行流
- **Planner**: 使用 LLM 将用户意图分解为一系列 Skill 调用。
- **Executor**: 顺序或并行执行计划中的步骤，并根据执行结果动态调整后续计划。

### 3. 安全与隔离
- **无需沙箱**: 所有的修改都在独立的分支 Thread 上进行，通过合并 (Merge) 而非沙箱隔离来确保系统安全。
- **异步流式**: Agent 的思考和执行过程完全异步，支持实时的流式结果回传。

---
更多技术细节请参阅源代码。
