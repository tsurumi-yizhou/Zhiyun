# backend/agent

## 概述

提供Agent执行工具集，Agent作为"进程"可以fork、执行工具、生成SemanticOp。支持智能上下文管理、模型路由、工具选择。Agent具有元调用能力，可以调用fork指令创建子Agent。

## 核心职责

1. **Agent fork** - 创建子Agent（类似进程fork）
2. **上下文管理** - 模型驱动的智能上下文压缩和选择
3. **工具执行** - Agent调用其他模块的工具函数
4. **SemanticOp生成** - Agent生成语义化操作
5. **模型路由** - 路由模型选择LLM接口
6. **工具选择** - 模型决定可用工具集

## 文件结构

```text
src/
├── lib.rs              # 模块导出
├── routine.rs          # AgentRoutine定义（Agent执行例程）
├── context.rs          # 上下文管理
├── router.rs           # 模型路由器
├── selector.rs         # 工具选择器
└── tools.rs            # 工具函数集
```

---

## lib.rs

**职责**: 模块导出，不实现功能

导出AgentRoutine、ContextManager、ModelRouter、ToolSelector、fork等核心类型和函数。

---

## routine.rs

**职责**: AgentRoutine定义（Agent执行例程）

**核心概念**:

- AgentRoutine类似操作系统进程
- 每个Routine有自己的CRDTThread和上下文
- 可以fork创建子Routine
- 可以执行工具并生成SemanticOp

**核心类型**:

- `AgentRoutine`: Agent执行例程，包含id、parent_id、crdt_thread、ir_graph、context、state
- `RoutineState`: Running、Waiting、Completed、Failed
- `RoutineId`: 例程唯一标识符

**主要方法**:

- `fork()`: 创建子Routine
- `execute_step()`: 执行一步，返回SemanticOp
- `apply_ops()`: 应用SemanticOp到CRDTThread
- `get_state()`: 获取当前状态
- `wait_for_completion()`: 等待Routine完成

**工具执行**:

- 调用syntax、semantic、knowledge、project等工具
- 调用fork创建子Routine
- 工具结果用于生成SemanticOp

**测试要点**:

- Fork创建独立Routine
- 子Routine继承父Routine上下文
- 工具执行成功
- SemanticOp生成正确

---

## context.rs

**职责**: 上下文管理

**核心概念**:

- 模型驱动的智能上下文压缩
- 选择最近4次对话
- 主动探测必要的环境信息
- 从记忆库检索相关信息

**核心类型**:

- `ContextManager`: 上下文管理器
- `AgentContext`: Agent上下文，包含recent_conversation、environment、memory
- `ConversationTurn`: 对话轮次，包含role、content
- `ContextSnapshot`: 环境快照

**主要方法**:

- `build_context()`: 构建Agent上下文
- `select_recent_conversations()`: 选择最近4次对话（模型驱动）
- `probe_environment()`: 主动探测环境信息（模型驱动）
- `retrieve_memory()`: 从记忆库检索
- `compress()`: 压缩上下文

**上下文压缩流程**:

1. 模型分析任务，确定需要的信息
2. 选择最近相关的4次对话
3. 主动探测必要的符号、文件信息
4. 从记忆库检索相关知识
5. 组合成精简的上下文

**测试要点**:

- 上下文包含必要信息
- 压缩后不超出token限制
- 相关性排序准确

---

## router.rs

**职责**: 模型路由器

**核心概念**:

- 轻量级路由模型选择合适的LLM接口
- 基于任务类型、上下文大小、时间预算决策
- 支持多模型组合（快速模型+高质量模型）

**核心类型**:

- `ModelRouter`: 路由器
- `RoutingDecision`: 路由决策，包含endpoint_id、reason
- `RoutingRequirements`: 路由需求

**主要方法**:

- `route()`: 路由到合适的模型
- `register_endpoint()`: 注册LLM端点
- `set_router_model()`: 设置路由模型

**路由策略**:

1. 基于能力匹配过滤候选模型
2. 如果只有一个候选，直接返回
3. 如果有多个，使用路由模型决策
4. 如果没有路由模型，使用启发式规则

**测试要点**:

- 路由决策合理
- 能力匹配准确
- 性能可接受

---

## selector.rs

**职责**: 工具选择器

**核心概念**:

- 模型决定Agent的可用工具集
- 根据任务类型动态调整
- 避免工具过多导致决策困难

**核心类型**:

- `ToolSelector`: 工具选择器
- `ToolSet`: 工具集
- `ToolCapability`: 工具能力描述

**主要方法**:

- `select_tools()`: 选择工具集
- `get_available_tools()`: 获取可用工具列表
- `describe_tool()`: 获取工具描述

**选择策略**:

1. 分析任务类型
2. 匹配工具能力
3. 选择最相关的工具子集
4. 限制工具数量（避免过多）

**测试要点**:

- 工具选择相关
- 工具数量合理
- 描述清晰准确

---

## tools.rs

**职责**: 工具函数集

**核心概念**:

- 提供Agent可调用的工具函数
- 封装其他模块的功能
- 支持元调用（fork等）

**核心类型**:

- `ToolCall`: 工具调用，包含name、arguments
- `ToolOutput`: 工具输出
- `ToolError`: 工具错误

**主要方法**:

- `parse()`: 调用syntax::parse()
- `build_ir()`: 调用semantic::build_ir()
- `search_symbols()`: 调用knowledge::search_symbols()
- `build()`: 调用project::build()
- `fork()`: 创建子AgentRoutine

**工具分类**:

- 语法工具: parse、extract_symbols
- 语义工具: build_ir、infer_types
- 知识工具: search_symbols、find_references
- 项目工具: build、test
- Agent工具: fork

**测试要点**:

- 工具调用成功
- 参数解析正确
- 错误处理完善

---

## 依赖

```toml
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
async-trait = { workspace = true }
```

---

## Agent执行流程

1. Fork创建AgentRoutine
2. 构建上下文（模型驱动压缩）
3. 路由到合适的LLM
4. 选择可用工具集
5. LLM生成工具调用
6. 执行工具，收集结果
7. 生成SemanticOp
8. 应用到CRDTThread
9. 提交合并或继续执行

---

## fork指令

Agent可以调用fork创建子Routine:

- 子Routine继承父Routine的CRDTThread（fork）
- 子Routine继承压缩后的上下文
- 父Routine可以等待子Routine完成
- 支持并行执行多个子Routine

**使用场景**:

- 并行处理多个文件
- 分解复杂任务
- 尝试多种方案并选择最优

---

## 测试要点

- Fork创建独立Routine
- 上下文继承正确
- 工具执行成功
- SemanticOp生成准确
- 模型路由合理
- 并行执行不冲突
