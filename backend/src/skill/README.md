# Skill 模块 (Agent Skills)

`skill` 模块是系统能力的封装层，它将底层的复杂操作（语法分析、项目构建、知识检索）抽象为 Agent 可理解和调用的结构化工具。

## 核心功能

- **技能注册与发现**: 所有的技能都必须实现 `Skill` Trait，并声明其输入参数的 Schema 和功能描述。
- **受控操作**: Skill 是 Agent 修改系统的唯一合法途径。
- **变更绑定**: Skill 的执行结果如果涉及代码修改，必须产生 CRDT Change 并绑定到调用者的 Thread。

## 核心组件

- [registry.rs](./registry.rs): `SkillRegistry` 技能的全局仓库，支持动态加载。
- [loader.rs](./loader.rs): 负责技能的动态发现与加载。
- [injector.rs](./injector.rs): 技能依赖注入机制。
- [tool.rs](./tool.rs): 技能与 LLM Tool Call 的转换适配。
- [types.rs](./types.rs): 技能相关的基础类型定义。
- [state.rs](./state.rs): 技能执行的状态管理。

## 设计原则

- **高内聚**: 每个 Skill 应该只负责一个原子的逻辑功能。
- **解耦**: Skill 模块通过 `common/meta/registry` 获取其他模块的能力，不产生直接依赖。

## 架构设计细节

### 1. 技能契约模型
- **自描述接口**: 每个技能通过 Schema 声明其意图、输入参数和预期输出，这是 LLM 进行 Function Calling 的基础。
- **原子性设计**: 强调每个技能的高内聚，确保 Agent 可以精准、灵活地组合不同技能完成复杂任务。

### 2. 变更追溯机制
- **CRDT 绑定**: 任何由技能触发的代码变更都会被自动封装为 CRDT 操作，并带上调用者的 Routine ID 标签，实现变更的可追溯性。

### 3. 动态扩展性
- **依赖注入**: 支持在运行时为技能注入必要的基础设施（如特定的 LLM 端点或 FS Provider）。
- **解耦调用**: 技能模块本身不感知业务模块，而是通过全局注册表动态获取所需能力。

---
更多技术细节请参阅源代码。
