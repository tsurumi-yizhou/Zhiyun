# Syntax Engine

`engine` 模块定义了语法解析引擎的核心抽象接口。

## 核心组件

- [interface.rs](./interface.rs): 定义了 `ParserEngine` Trait，所有的语法解析实现都必须遵循此接口。
- [adapter.rs](./adapter.rs): `Adapter` 核心抽象层，将 Tree-sitter 的节点与元 AST 映射。
- [traversal.rs](./traversal.rs): `Traversal` 实现高效的树遍历与查询。
- [query.rs](./query.rs): `QueryEngine` 执行 SCM 查询并返回匹配结果。
