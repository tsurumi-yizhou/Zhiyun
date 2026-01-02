# Skill 模块设计 (Skill Module Design)

`skill` 模块是系统能力的封装层，它将底层的复杂操作（语法分析、项目构建、知识检索）抽象为 Agent 可理解和调用的结构化工具。

## 1. 核心功能

### 1.1 技能注册与发现
- **统一接口**: 所有的技能都必须实现 `Skill` Trait。
- **元数据定义**: 技能必须声明其输入参数的 Schema 和功能描述，以便 LLM 理解。

### 1.2 结构化调用
- **受控操作**: Skill 是 Agent 修改系统的唯一合法途径。
- **变更绑定**: Skill 的执行结果如果涉及代码修改，必须产生 CRDT Change 并绑定到调用者的 Thread。

## 2. 核心组件
- `SkillRegistry`: 技能的全局仓库，支持动态加载。
- `ToolCallValidator`: 验证 Agent 传入参数的合法性。
- `BuiltinSkills`: 内置的基础技能（如读写文件、语义搜索）。

## 3. 设计原则
- **高内聚**: 每个 Skill 应该只负责一个原子的逻辑功能。
- **解耦**: Skill 模块通过 `common/meta/registry` 获取其他模块的能力，不产生直接依赖。
