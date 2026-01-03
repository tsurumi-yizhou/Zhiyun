# Local Provider

`local` 模块实现了基于本地操作系统的基础设施访问。

## 核心组件

- [filesystem.rs](./filesystem.rs): 封装了 `std::fs` 操作，提供符合 `FileSystem` Trait 的实现。
- [process.rs](./process.rs): 封装了本地进程的启动、监控和信号管理。
