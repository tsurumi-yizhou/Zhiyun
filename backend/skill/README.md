# Skill Module

提供语言和项目特定的技能知识库，支持通过工具调用接口进行管理。

## 概述

Skill 是结构化的知识片段，可以注入到 LLM prompt 中以增强模型对特定语言、工具、任务的理解。

- **注册管理**: 全局状态存储，进程生命周期
- **分类索引**: 按类别、语言、标签组织
- **智能检索**: 基于任务描述查找相关技能
- **工具接口**: 5 个工具可供 LLM 调用

## 文件结构

```
src/
├── lib.rs              # 模块导出
├── types.rs            # 类型定义
├── registry.rs         # 技能注册表
├── loader.rs           # 技能加载器
├── injector.rs         # Prompt 注入器
├── state.rs            # 全局状态管理
└── tool.rs             # 工具调用接口
```

## 核心 API

### 1. 类型定义

```rust
use skill::{Skill, SkillCategory, SkillId};

// 技能 ID
let id = SkillId::new(SkillCategory::Syntax, "macro_rules", "Rust");

// 技能分类
pub enum SkillCategory {
    Syntax,          // 语法相关
    Semantic,        // 语义理解
    Project,         // 项目特定知识
    Refactoring,     // 重构模式
    LanguageSpecific, // 语言特定惯用法
}
```

### 2. 全局状态管理

```rust
use skill::SkillState;

// 获取全局状态（进程级单例）
let state = SkillState::get().read().await;

// 预加载技能（程序启动时）
use skill::{SkillConfig, SkillLoader};
use serde_json::json;

let config = SkillConfig {
    files: vec!["skills/rust.yaml".to_string()],
    inline_skills: vec![json!({...})],
};

SkillState::preload_from_config(&config).await?;
```

### 3. 工具调用接口

```rust
use skill::SkillToolRegistry;
use serde_json::json;

// 创建工具注册表
let tools = SkillToolRegistry::new();

// 获取所有工具的 schema（用于 LLM function calling）
let schemas = tools.get_all_schemas();
// 返回: [{"name": "register_skill", "description": "...", "parameters": {...}}, ...]

// 执行工具
let result = tools.execute(
    "register_skill",
    json!({
        "skill": {
            "id": {"category": "Syntax", "name": "test", "language": "Rust"},
            "name": "Test Skill",
            "description": "A test skill",
            "content": "Test content",
            "examples": [],
            "related_tools": [],
            "metadata": {"language": "Rust", "version": "1.0", "tags": []}
        }
    })
).await?;

println!("{}", result.content);
if let Some(data) = result.data {
    println!("{}", serde_json::to_string_pretty(&data)?);
}
```

## 可用工具

| 工具名 | 描述 | 参数 |
|--------|------|------|
| `register_skill` | 注册新技能 | `skill` (object) |
| `search_skills` | 搜索相关技能 | `task` (string), `language?`, `limit?` |
| `inject_skills` | 注入技能到提示词 | `task`, `base_prompt`, `max_skills?` |
| `get_skill` | 按 ID 获取技能 | `category`, `name`, `language` |
| `list_skills` | 列出所有技能 | `category?`, `language?` |

### 工具参数 Schema

```json
{
  "name": "register_skill",
  "description": "Register a new skill to the knowledge base",
  "parameters": {
    "type": "object",
    "properties": {
      "skill": {
        "type": "object",
        "description": "Skill definition (same format as YAML/JSON file)"
      }
    },
    "required": ["skill"]
  }
}
```

## 技能配置格式

### YAML 格式

```yaml
id:
  category: Syntax
  name: rust_macro_rules
  language: Rust
name: "Rust macro_rules! Syntax"
description: "如何解析 Rust 的 macro_rules! 宏"
content: |
  Rust 的 macro_rules! 宏有以下特点...
examples:
  - input: "解析宏定义"
    output: "使用 TreeSitter 查询..."
    explanation: "匹配 macro_definition 节点"
related_tools:
  - syntax::parse
metadata:
  language: Rust
  version: "1.0"
  tags: ["macro", "syntax"]
```

### JSON 格式

```json
{
  "id": {
    "category": "Syntax",
    "name": "rust_macro_rules",
    "language": "Rust"
  },
  "name": "Rust macro_rules! Syntax",
  "description": "如何解析 Rust 的 macro_rules! 宏",
  "content": "Rust 的 macro_rules! 宏有以下特点...",
  "examples": [
    {
      "input": "解析宏定义",
      "output": "使用 TreeSitter 查询...",
      "explanation": "匹配 macro_definition 节点"
    }
  ],
  "related_tools": ["syntax::parse"],
  "metadata": {
    "language": "Rust",
    "version": "1.0",
    "tags": ["macro", "syntax"]
  }
}
```

## 使用流程

```rust
use skill::{SkillState, SkillToolRegistry, SkillConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 预加载技能（可选）
    let config = SkillConfig {
        files: vec!["skills/rust.yaml".to_string()],
        inline_skills: vec![],
    };
    SkillState::preload_from_config(&config).await?;

    // 2. 获取工具注册表
    let tools = SkillToolRegistry::new();

    // 3. LLM 可以调用工具
    let result = tools.execute(
        "search_skills",
        json!({
            "task": "parse Rust macro",
            "language": "Rust",
            "limit": 3
        })
    ).await?;

    // 4. 使用返回的技能
    if let Some(data) = result.data {
        let skills: Vec<SkillInfo> = serde_json::from_value(data)?;
        for skill in skills {
            println!("{}: {}", skill.name, skill.description);
        }
    }

    Ok(())
}
```

## 设计特点

| 特性 | 说明 |
|------|------|
| 全局状态 | 进程生命周期，使用 `OnceLock` + `RwLock` 实现 |
| 并发安全 | 所有操作都是异步的，使用读写锁保护 |
| 预加载支持 | 程序启动时加载，运行时动态注册 |
| 无持久化 | 仅进程内状态，重启后清空 |
| 多用户共享 | 所有 agent 共享技能库 |

## 依赖

```toml
[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
serde_yaml = { workspace = true }
thiserror = { workspace = true }
tokio = { workspace = true }
async-trait = { workspace = true }
```

## 测试

```bash
# 运行所有测试
cargo test -p skill

# 运行特定测试
cargo test -p skill test_register_skill_tool

# 带输出的测试
cargo test -p skill -- --nocapture
```
