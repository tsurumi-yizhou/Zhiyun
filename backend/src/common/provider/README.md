# Provider 模块 (Infrastructure Abstraction)

`provider` 模块提供了底层基础设施（文件系统、进程管理）的统一抽象，支持本地与远程操作的透明化。

## 核心子模块

- [local/](./local/): 本地基础设施实现。
- [remote/](./remote/): 远程基础设施实现（通过 SSH 或 Agent 代理）。

## 核心组件

- [traits.rs](./traits.rs): 定义了 `FileSystem` 和 `ProcessManager` 的标准接口。

## 关键能力

- **透明访问**: 上层模块无需关心文件是存储在本地磁盘还是远程服务器上。
- **环境隔离**: 为不同的 `Routine` 提供受控的执行环境。
