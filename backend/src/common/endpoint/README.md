# Endpoint 模块 (LLM Communication)

`endpoint` 模块提供了统一的 LLM (Large Language Model) 访问协议，隐藏了不同模型供应商（如 OpenAI, Anthropic, DeepSeek 等）之间的 API 差异。

## 核心组件

- [traits.rs](./traits.rs): 定义了 `LLMClient` 接口，包括 Chat、Embedding 等功能。
- [registry.rs](./registry.rs): 管理已配置的 LLM 端点和模型路由逻辑。
- [stream.rs](./stream.rs): 处理 LLM 的流式输出。
- [error.rs](./error.rs): 统一的错误处理机制。

## 关键功能

- **模型路由**: 根据任务的复杂度（如简单的代码格式化 vs 复杂的架构重构）自动选择最合适的模型。
- **协议统一**: 为上层模块（如 `agent`）提供一致的交互界面。
