# Zhiyun Backend Source

这是 Zhiyun 后端的源代码根目录。系统采用模块化设计，每个模块负责特定的领域。

## 目录结构

- [agent/](./agent/): **Agent 运行时**。管理 Routine 生命周期、任务规划与执行。
- [common/](./common/): **基础设施**。包含 CRDT、元 AST 定义、LLM 通信和环境抽象。
- [compiler/](./compiler/): **编译器集成**。提供基于真实编译器的权威诊断与验证。
- [editor/](./editor/): **编辑器运行时**。管理用户会话、Tab 状态与 Thread 同步。
- [knowledge/](./knowledge/): **知识层**。实现 RAG 流程、向量存储与知识图谱。
- [project/](./project/): **项目管理**。抽象构建系统、依赖关系与工作空间结构。
- [semantic/](./semantic/): **语义分析**。构建 Scope Graph，提供符号导航与重构支持。
- [skill/](./skill/): **技能系统**。将系统能力封装为 Agent 可调用的工具。
- [syntax/](./syntax/): **语法层**。基于 Tree-sitter 的插件化解析引擎。

## 核心设计原则

1. **变更核心 (Change-Centric)**: 所有修改通过 CRDT Thread 管理。
2. **语言无关 (Language-Agnostic)**: 通过元 AST 抽象支持多种编程语言。
3. **插件化**: 所有的语言支持、技能和编译器都是可插拔的。
