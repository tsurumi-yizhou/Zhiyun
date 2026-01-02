# Syntax 模块设计 (Syntax Module Design)

`syntax` 模块负责将源代码解析为系统可理解的结构化数据，它是实现语言无关性的第一站。

## 1. 核心功能

### 1.1 插件化解析引擎
- **Tree-sitter 驱动**: 默认使用 Tree-sitter 作为底层解析工具。
- **SCM 赋予能力**: 通过加载 `.scm` (Queries) 文件，动态定义如何从原生 AST 中提取符号和结构信息。

### 1.2 元 AST 转换
- **标准化**: 将 Tree-sitter 的输出转换为 `common/meta/ast` 中定义的 `MetaNode` 树。
- **解耦**: 上层模块（如 `semantic`）只处理 `MetaNode`，不感知底层具体的解析技术。

## 2. 核心组件
- `ParserExecutor`: 负责调度注册的解析器插件。
- `GrammarLoader`: 动态加载不同语言的语法文件和 SCM 查询。
- `IncrementalCache`: 管理增量解析的缓存，提升实时编辑性能。

## 3. 设计原则
- **不假设语言**: 所有的解析规则都存在于外部插件或配置中。
- **零硬编码**: 严禁在代码中出现 `if lang == "rust"` 这种逻辑。
