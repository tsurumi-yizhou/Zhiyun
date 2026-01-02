# backend/skill

## 概述

提供语言和项目特定的技能知识库。Skill是前端传入的结构化知识，注入到LLM prompt中以增强模型对特定语言、工具、任务的理解。

## 核心职责

1. **Skill注册和管理** - 注册、索引、检索技能
2. **Skill加载** - 从文件或内联配置加载Skill
3. **Skill注入** - 将相关Skill注入到LLM prompt
4. **Skill分类** - 按类别（语法、语义、项目、重构）组织

## 文件结构

```text
src/
├── lib.rs              # 模块导出
├── registry.rs         # Skill注册表
├── loader.rs           # Skill加载器
├── injector.rs         # Skill注入到prompt
└── types.rs            # Skill类型定义
```

---

## lib.rs

**职责**: 模块导出，不实现功能

导出Skill、SkillRegistry、SkillLoader、SkillInjector等核心类型。

---

## types.rs

**职责**: Skill相关的类型定义

**核心类型**:

- `SkillId`: Skill标识符，包含category、name、language
- `SkillCategory`: Syntax、Semantic、Project、Refactoring、LanguageSpecific
- `Skill`: Skill定义，包含id、name、description、content、examples、metadata
- `SkillExample`: 示例，包含input、output、explanation
- `SkillMetadata`: 元数据，包含language、version、author、tags

**主要方法**:

- `SkillId::new()`: 创建新的Skill ID
- `Skill::validate()`: 验证Skill内容的有效性

**测试要点**:

- Skill ID的唯一性
- Skill验证（必填字段、格式正确性）

---

## registry.rs

**职责**: Skill注册表，管理和索引所有Skill

**核心概念**:

- 注册表维护所有可用的Skill
- 支持多种索引：按category、language、tags
- 向量索引用于语义搜索相关Skill

**核心类型**:

- `SkillRegistry`: 注册表，包含skills HashMap和多个索引
- 索引类型: by_category (HashMap)、by_language (HashMap)、vector_index (VectorDB)

**主要方法**:

- `register()`: 注册新Skill，更新所有索引
- `get()`: 查找Skill
- `find_relevant()`: 根据任务查找相关Skill
- `by_category()`: 按类别获取Skill列表
- `by_language()`: 按语言获取Skill列表

**测试要点**:

- 注册后Skill可以被检索
- 多个索引保持一致性
- 查找相关Skill返回正确结果

---

## loader.rs

**职责**: 从前端传入的配置加载Skill

**核心概念**:

- 前端通过YAML文件或内联JSON传入Skill
- 支持批量加载多个Skill
- 加载时验证Skill格式

**核心类型**:

- `SkillLoader`: 加载器，提供静态方法
- `SkillConfig`: 配置，包含files路径列表和inline_skills列表

**主要方法**:

- `from_config()`: 从配置加载Skill列表
- `load_from_file()`: 从文件加载单个Skill
- `load_from_yaml()`: 从YAML字符串解析Skill
- `load_from_json()`: 从JSON字符串解析Skill

**测试要点**:

- 从文件加载Skill成功
- YAML格式解析正确
- 无效格式返回错误

---

## injector.rs

**职责**: 将Skill注入到LLM prompt

**核心概念**:

- 根据任务描述查找相关Skill
- 将Skill内容和示例插入到系统提示
- 限制Skill数量避免超出token限制

**核心类型**:

- `SkillInjector`: 注入器，包含registry引用
- `InjectionConfig`: 配置，包含max_skills、max_examples_per_skill

**主要方法**:

- `inject_to_prompt()`: 注入相关Skill到prompt
- `find_relevant_skills()`: 查找相关Skill（按类别和语义）
- `format_skill()`: 格式化Skill为Markdown
- `infer_category()`: 从任务描述推断类别

**注入策略**:

1. 解析任务描述，推断类别
2. 按类别查找候选Skill
3. 使用向量数据库排序
4. 选择Top N个Skill
5. 格式化为Markdown插入到prompt

**测试要点**:

- 注入后的prompt包含Skill内容
- 相关度高的Skill排在前面
- 不会超出token限制

---

## 依赖

```toml
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
```

---

## Skill YAML格式示例

```yaml
id:
  category: Syntax
  name: rust_macro_rules
  language: Rust
name: "Rust macro_rules! Syntax"
description: "如何解析Rust的macro_rules!宏"
content: |
  Rust的macro_rules!宏有以下特点...
examples:
  - input: "解析宏定义"
    output: "使用TreeSitter查询..."
    explanation: "匹配macro_definition节点"
related_tools:
  - syntax::parse
metadata:
  language: Rust
  version: "1.0"
  tags: ["macro", "syntax"]
```

---

## 使用流程

1. 前端传入Skill配置（YAML文件或内联JSON）
2. SkillLoader加载所有Skill
3. 注册到SkillRegistry
4. Agent执行任务时，SkillInjector查找相关Skill
5. 注入到LLM prompt
6. LLM利用Skill知识生成更准确的响应
