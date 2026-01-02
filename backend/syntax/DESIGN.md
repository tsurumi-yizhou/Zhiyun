# backend/syntax

## 概述

提供语法分析工具集，基于TreeSitter实现代码解析、符号提取、结构查询。前端传入.scm查询文件和语言配置，后端执行解析并返回结果。

## 核心职责

1. **语言配置管理** - 管理前端传入的语言配置（.wasm语法和.scm查询）
2. **代码解析** - 使用TreeSitter解析代码生成语法树
3. **符号提取** - 使用.scm查询提取符号定义和引用
4. **结构查询** - 执行任意TreeSitter查询获取语法结构

## 文件结构

```text
src/
├── lib.rs              # 模块导出
├── config.rs           # 语言配置定义
├── parser.rs           # TreeSitter解析器
├── queries.rs          # .scm查询管理
└── symbols.rs          # 符号提取
```

---

## lib.rs

**职责**: 模块导出，不实现功能

导出LanguageConfig、SyntaxService、ParsedFile、Symbol等核心类型。

---

## config.rs

**职责**: 语言配置定义和加载

**核心概念**:

- 前端传入语言配置，包含编译后的TreeSitter语法和.scm查询文件
- 配置是动态的，不硬编码任何语言
- 支持多种查询类型：符号定义、引用、作用域

**核心类型**:

- `LanguageConfig`: 语言配置，包含name、file_extensions、tree_sitter_language、queries
- `LanguageQueries`: 查询集合，包含definitions、references、scopes
- `QueryType`: 查询类型枚举

**主要方法**:

- `from_frontend()`: 从前端传入的数据创建配置
- `supports_extension()`: 检查是否支持文件扩展名
- `get_query()`: 获取特定类型的查询

**测试要点**:

- 配置加载正确
- 扩展名匹配准确
- 查询获取成功

---

## parser.rs

**职责**: TreeSitter解析器

**核心概念**:

- 使用TreeSitter解析源代码生成语法树
- 支持增量解析（大文件优化）
- 解析错误不阻断整个树（容错性）

**核心类型**:

- `SyntaxService`: 服务，包含语言配置和解析器池
- `ParsedFile`: 解析结果，包含tree、language、errors

**主要方法**:

- `parse()`: 解析源代码，返回ParsedFile
- `parse_with_errors()`: 解析并返回错误信息
- `detect_language()`: 根据文件扩展名检测语言

**解析流程**:

1. 根据文件扩展名查找语言配置
2. 创建或复用TreeSitter Parser
3. 解析源代码生成Tree
4. 收集解析错误（如果有）
5. 返回ParsedFile

**测试要点**:

- 正确的代码解析成功
- 错误的代码也能部分解析（容错性）
- 语言检测准确

---

## queries.rs

**职责**: TreeSitter查询管理

**核心概念**:

- 前端通过.scm文件定义查询
- 查询使用TreeSitter的查询语法
- 支持命名捕获（@name）和谓词

**核心类型**:

- `QueryManager`: 管理所有查询
- `QueryResult`: 查询结果，包含matches和captures
- `QueryMatch`: 单个匹配，包含pattern和captures

**主要方法**:

- `execute_query()`: 执行查询返回结果
- `extract_captures()`: 提取命名的捕获节点
- `load_query_from_scm()`: 从.scm字符串加载查询

**常用查询类型**:

- 符号定义查询: 提取函数、类、变量定义
- 符号引用查询: 提取函数调用、类型引用
- 作用域查询: 提取块、函数体作用域

**测试要点**:

- 查询执行正确
- 捕获提取准确
- .scm解析成功

---

## symbols.rs

**职责**: 符号提取

**核心概念**:

- 使用查询提取符号定义和引用
- 符号包含位置、类型、名称等信息
- 支持跨文件符号引用

**核心类型**:

- `Symbol`: 符号，包含id、name、kind、span、file
- `SymbolKind`: 符号类型（Function、Struct、Variable等）
- `Reference`: 引用，包含symbol_id、span、file
- `SymbolTable`: 符号表，包含definitions和references

**主要方法**:

- `extract_definitions()`: 提取符号定义
- `extract_references()`: 提取符号引用
- `build_symbol_table()`: 构建符号表
- `find_definition()`: 查找符号定义

**提取流程**:

1. 执行定义查询，提取所有符号定义
2. 执行引用查询，提取所有符号引用
3. 为每个符号分配唯一ID
4. 构建符号表

**测试要点**:

- 定义提取准确
- 引用定位正确
- 符号表完整

---

## 依赖

```toml
tree-sitter = "0.22"
serde = { workspace = true }
thiserror = { workspace = true }
```

---

## 使用流程

1. 前端传入LanguageConfig（包含.wasm和.scm）
2. SyntaxService加载配置
3. 调用parse()解析代码
4. 调用extract_symbols()提取符号
5. 返回ParsedFile和SymbolTable给调用者

---

## .scm查询示例

```scm
;; 函数定义查询
(function_definition
  name: (identifier) @def.name
  parameters: (parameters
    (parameter
      name: (identifier) @param.name)*)
  body: (block) @def.body) @def.scope

;; 函数调用查询
(call_expression
  function: (identifier) @ref.name) @ref.call
```
