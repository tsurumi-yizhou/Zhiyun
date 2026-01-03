# Remote Provider

`remote` 模块实现了针对远程环境的基础设施访问。

## 核心组件

- [filesystem.rs](./filesystem.rs): 通过网络协议（如 SFTP 或自定义代理）实现远程文件操作。
- [process.rs](./process.rs): 实现远程进程的执行与流式日志回传。
