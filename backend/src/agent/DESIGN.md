# Agent 运行时设计 (Agent Runtime Design)

`agent` 模块负责管理智能 Agent 的生命周期、任务规划和 Routine 执行，它是系统的“智能大脑”。

## 1. 核心功能

### 1.1 Routine 管理
- **Routine (进程)**: 每一个 Agent 任务运行在一个 Routine 中，拥有独立的对话历史和任务状态。
- **元调用 (Fork)**: Routine 能够派生子 Routine 来递归分解复杂任务。

### 1.2 任务规划与执行
- **Planner**: 使用 LLM 将用户意图分解为一系列 Skill 调用。
- **Executor**: 顺序或并行执行计划中的步骤，并根据执行结果动态调整后续计划。

## 2. 核心组件
- `RoutineManager`: 跟踪所有活跃的 Routine 及其层级关系。
- `ContextManager`: 负责对话上下文的智能压缩与窗口管理。
- `MergerBridge`: 协调 Routine 产生的变更合并到对应的 Thread。

## 3. 设计原则
- **无需沙箱**: 依赖 Thread 分支机制实现修改的隔离与安全性。
- **异步与流式**: Agent 的思考和执行过程应该是异步的，支持流式结果输出。
