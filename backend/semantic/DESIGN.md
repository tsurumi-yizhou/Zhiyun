# backend/semantic

## 概述

提供语义分析工具集，包括通用IR（中间表示）定义、TreeSitter到IR转换、类型推断、编译器AST集成。实现三层符号信息融合（TreeSitter、LLM、编译器）。

## 核心职责

1. **通用IR定义** - 跨语言的AST中间表示
2. **AST到IR转换** - TreeSitter树转换为IRGraph
3. **类型推断** - LLM辅助的类型信息推断
4. **编译器集成** - 调用编译器获取权威AST
5. **符号融合** - 合并三层符号信息

## 文件结构

```text
src/
├── lib.rs              # 模块导出
├── ir.rs               # 通用IR定义
├── meta_ast.rs         # MetaAST定义（TreeSitter→IR映射）
├── converter.rs        # AST到IR转换器
├── inference.rs        # LLM类型推断
├── compiler.rs         # 编译器桥接
└── symbols.rs          # 符号表和融合
```

---

## lib.rs

**职责**: 模块导出，不实现功能

导出IRNode、IRGraph、IRToIRConverter、SymbolTable、InferenceStrategy等核心类型。

---

## ir.rs

**职责**: 通用中间表示（IR）定义

**核心概念**:

- IR是语言无关的AST抽象
- 提供LanguageSpecific节点作为扩展点
- 支持完整的类型系统

**核心类型**:

- `IRNode`: Module、Function、Struct、Expression、Statement、Type、LanguageSpecific
- `IRGraph`: 图结构，包含nodes HashMap、root、parent_map
- `IRType`: Primitive、Named、Function、Reference、Array、TypeParameter、Unknown、Tuple
- `IRExpression`: Literal、Variable、Call、BinaryOp、Block、If、Loop、Lambda等
- `IRStatement`: Let、Assignment、Expression、Return、ControlFlow
- `NodeId`: 节点唯一标识符

**主要方法**:

- `IRGraph::new()`: 创建空的IR图
- `IRGraph::get_node()`: 获取节点
- `IRGraph::add_node()`: 添加节点
- `IRNode::id()`: 获取节点ID
- `IRNode::children()`: 获取子节点列表

**测试要点**:

- IR序列化/反序列化正确
- 图结构维护一致性（父子关系）
- LanguageSpecific节点可以作为扩展点

---

## meta_ast.rs

**职责**: MetaAST定义，描述TreeSitter到IR的映射

**核心概念**:

- 前端传入MetaAST配置，定义如何将TreeSitter节点转换为IR节点
- 支持命名子节点、索引子节点、查询子节点
- 可以提取名称、类型、元数据

**核心类型**:

- `MetaAST`: 元AST，包含language、node_mappings、type_mappings
- `NodeMapping`: 节点映射规则（ts_node_type、ir_node_type、提取规则）
- `ChildExtractionRule`: Child、Named、Query、Custom
- `TypeExtractionRule`: 类型提取规则

**主要方法**:

- `MetaAST::rust_definition()`: 返回Rust语言的MetaAST
- `MetaAST::typescript_definition()`: 返回TypeScript的MetaAST
- `NodeMapping::extract_name()`: 提取节点名称
- `NodeMapping::extract_type()`: 提取类型信息

**测试要点**:

- MetaAST加载成功
- 映射规则正确提取信息

---

## converter.rs

**职责**: TreeSitter AST到IR转换器

**核心概念**:

- 使用MetaAST定义的映射规则转换
- 递归转换所有子节点
- 保持图的父子关系

**核心类型**:

- `ASTToIRConverter`: 转换器，包含meta_ast和language
- `ConversionContext`: 转换上下文，跟踪节点映射

**主要方法**:

- `convert()`: 转换TreeSitter树为IRGraph
- `convert_node()`: 递归转换单个节点
- `resolve_node_path()`: 解析NodePath找到TS节点

**转换流程**:

1. 查找TS节点类型的映射规则
2. 根据映射规则创建IR节点
3. 递归转换子节点
4. 维护父子关系
5. 返回IRGraph

**测试要点**:

- 简单函数转换正确
- 嵌套结构转换正确
- 语言特定节点正确处理

---

## inference.rs

**职责**: LLM类型推断

**核心概念**:

- LLM推断TreeSitter无法提供的类型信息
- 中等置信度，位于TreeSitter和编译器AST之间
- 按需触发（节省成本）

**核心类型**:

- `LLMInferenceService`: 推断服务
- `InferredSymbol`: 推断的符号
- `TypeInferencePrompt`: 类型推断的prompt模板

**主要方法**:

- `extract_symbols()`: 推断符号类型
- `infer_variable_types()`: 推断变量类型
- `infer_function_return_types()`: 推断函数返回类型
- `build_type_inference_prompt()`: 构造推断prompt

**推断流程**:

1. 传入TreeSitter符号列表
2. 构造prompt，要求LLM推断类型
3. 调用LLM
4. 解析返回的类型信息
5. 返回推断结果

**测试要点**:

- 简单类型推断准确
- 复杂类型（泛型、闭包）推断合理
- 解析LLM返回正确

---

## compiler.rs

**职责**: 编译器桥接

**核心概念**:

- 前端传入如何调用编译器的配置
- 编译器AST是最终权威来源
- 支持多种编译器集成方式

**核心类型**:

- `CompilerBridge`: 桥接服务
- `CompilerConfig`: 编译器配置（Shell、API、IPC）
- `CompilerOutputFormat`: 输出格式（JSON、Custom）
- `CompilerAST`: 编译器返回的AST

**主要方法**:

- `from_config()`: 从配置创建桥接
- `extract_symbols()`: 提取编译器符号
- `run_compiler()`: 运行编译器
- `parse_output()`: 解析编译器输出

**支持的编译器**:

- Rust: rustc --pretty=typed
- TypeScript: tsc --emitDeclarationOnly
- Python: pyright --outputjson

**测试要点**:

- 编译器调用成功
- 输出解析正确
- 符号提取完整

---

## symbols.rs

**职责**: 符号表和三层融合

**核心概念**:

- TreeSitter（低）、LLM（中）、编译器（高）三层符号信息
- 用户可选择融合策略
- 符号表维护多个来源的信息

**核心类型**:

- `SymbolTable`: 符号表，包含definitions、references、sources
- `Symbol`: 符号，包含id、name、kind、span、confidence、source
- `Confidence`: Low、Medium、High
- `SymbolSource`: TreeSitter、LLMInference、CompilerAST
- `InferenceStrategy`: TreeSitterOnly、OnDemandLLM、AutoLLM、Full

**主要方法**:

- `merge_sources()`: 合并三层符号信息
- `get_symbol_with_sources()`: 获取符号及其所有来源
- `should_update_with_llm()`: 判断是否应该用LLM覆盖TreeSitter
- `update_symbols()`: 根据策略更新符号表

**融合策略**:

- TreeSitterOnly: 仅使用TreeSitter符号
- OnDemandLLM: TreeSitter + 按需LLM
- AutoLLM: TreeSitter + 自动LLM
- Full: TreeSitter + LLM + 编译器AST

**测试要点**:

- 合并逻辑正确
- 高置信度覆盖低置信度
- 冲突处理合理

---

## 依赖

```toml
tree-sitter = "0.22"
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
async-trait = { workspace = true }
tokio = { workspace = true }
```

---

## 使用流程

1. TreeSitter解析代码，生成符号（低置信度）
2. 根据策略决定是否触发LLM推断
3. 如果需要，调用LLM推断类型（中置信度）
4. 如果需要，调用编译器获取AST（高置信度）
5. 合并三层信息到SymbolTable
6. 返回给调用者使用

---

## MetaAST示例（Rust函数）

```text
TS节点: function_item
  ├─ name: identifier @name
  ├─ parameters: parameters @params
  └─ body: block @body

↓ 映射到

IR节点: IRFunction
  ├─ name: 从@name提取
  ├─ signature: 从@params提取
  └─ body: 从@body转换为语句列表
```
