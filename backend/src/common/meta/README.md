# Meta 模块 (Metaprogramming & Registry)

`meta` 模块负责系统的元编程抽象和插件注册机制，确保系统的语言无关性和高扩展性。

## 核心组件

- [ast.rs](./ast.rs): 定义了 **Meta AST (`MetaNode`)**，这是一种语言无关的统一语法树表示。
- [registry.rs](./registry.rs): 全局服务注册表，用于模块间的解耦发现。
- [plugin.rs](./plugin.rs): 定义插件加载与生命周期管理接口。
- [service.rs](./service.rs): 核心服务的抽象接口定义。

## 核心设计

- **统一表示**: 所有编程语言最终都转换为 `MetaNode`，使得后续的语义分析和重构逻辑可以复用。
- **解耦调用**: 模块间通过 `Registry` 获取彼此的能力，而不产生直接的编译时依赖。
