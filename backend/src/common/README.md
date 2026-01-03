# Common 基础设施 (Common Infrastructure)

`common` 模块是 Zhiyun 后端的基石，定义了所有其他模块共享的核心数据结构、抽象接口和基础设施。

## 核心子模块

- [change/](./change/): **CRDT 核心**。实现无冲突复制数据类型，管理版本化变更流。
- [meta/](./meta/): **元编程与插件注册**。定义元 AST 结构，管理全局插件与服务注册表。
- [endpoint/](./endpoint/): **LLM 通信**。提供统一的 LLM 访问协议，隐藏具体模型的 API 差异。
- [provider/](./provider/): **基础设施提供者**。提供统一的文件系统 (FS) 和进程管理接口，支持本地与远程透明操作。

## 设计原则

- **无依赖性**: `common` 严禁依赖任何其他业务模块（如 `syntax`, `agent` 等）。
- **接口优先**: 暴露 Trait 而非具体实现，确保基础设施的可替换性。

## 架构设计细节

### 1. 基础设施抽象
- **无依赖性**: `common` 严禁依赖任何其他业务模块（如 `syntax`, `agent` 等），确保其作为基石的稳定性。
- **接口优先**: 暴露 Trait 而非具体实现，确保底层基础设施（如文件系统、LLM 供应商）的可替换性。

### 2. 核心子模块职责
- **`change` (CRDT)**: 实现 `Operation` 原子操作、`Thread` 变更主线和 `MergeEngine` 三路合并算法，支撑系统的“变更核心”哲学。
- **`meta` (元编程)**: 定义 `MetaNode` 统一 AST，并提供 `PluginRegistry` 动态发现与加载各类插件。
- **`endpoint` (LLM)**: 通过 `LLMClient` 和 `ModelRouter` 提供统一的模型访问协议。
- **`provider` (环境)**: 提供统一的 FS 和进程管理接口，实现本地与远程环境的透明化操作。

---
更多技术细节请参阅源代码。
