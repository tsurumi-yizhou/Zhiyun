# Common 模块设计 (Common Module Design)

`common` 模块是 Zhiyun 后端的基石，定义了所有其他模块共享的核心数据结构、抽象接口和基础设施。

## 1. 核心子模块

### 1.1 `change` (CRDT 核心)
- **职责**: 实现无冲突复制数据类型 (CRDT)，管理版本化变更流。
- **关键组件**:
  - `Operation`: 语言无关的原子操作。
  - `Thread`: 变更主线抽象。
  - `MergeEngine`: 执行三路合并的算法核心。

### 1.2 `meta` (元编程与插件注册)
- **职责**: 定义元 AST 结构，管理全局插件与服务注册表。
- **关键组件**:
  - `MetaNode`: 语言无关的统一 AST 节点。
  - `PluginRegistry`: 动态发现与加载 Parser, Skill, Compiler 插件。

### 1.3 `endpoint` (LLM 通信)
- **职责**: 提供统一的 LLM 访问协议，隐藏具体模型的 API 差异。
- **关键组件**:
  - `LLMClient`: 统一的 Chat/Embedding 接口。
  - `ModelRouter`: 根据任务复杂度选择模型。

### 1.4 `provider` (基础设施)
- **职责**: 提供统一的文件系统 (FS) 和进程管理接口，支持本地与远程透明操作。

## 2. 设计原则
- **无依赖性**: `common` 严禁依赖任何其他业务模块（如 `syntax`, `agent` 等）。
- **接口优先**: 暴露 Trait 而非具体实现，确保基础设施的可替换性。
