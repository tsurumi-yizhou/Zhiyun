# Provider 模块设计文档

## 概述

Provider 模块为不同平台提供统一的文件系统管理和进程管理接口，支持本地平台（Linux/Unix/Windows）和远程平台（SSH）。

## 核心组件

### 1. FileProvider

文件提供者，负责管理指定平台的文件系统操作。

**主要功能：**

- 文件/目录的创建、读取、写入、删除
- 文件元数据查询（权限、大小、修改时间等）
- 目录遍历
- 文件监控（可选）

### 2. ProcessManager

进程管理器，负责在指定平台上执行命令。

**执行模式：**

- **流式返回（Streaming）**: 实时获取命令输出，适用于长时间运行的命令
- **一次性返回（Buffered）**: 等待命令执行完成后一次性返回全部输出，适用于短时间命令

**主要功能：**

- 执行命令并获取输出（stdout/stderr）
- 进程生命周期管理（启动、停止、状态查询）
- 环境变量和工作目录配置
- 退出码获取

## 模块结构

```text
provider/
├── Cargo.toml          # 依赖配置
├── DESIGN.md           # 设计文档
└── src/
    ├── lib.rs          # 模块入口，导出公共接口
    ├── traits.rs       # 公共 trait 定义和类型
    ├── local/          # 本地平台实现 (tokio)
    │   ├── mod.rs      # 模块入口
    │   ├── filesystem.rs     # FileProvider 实现
    │   └── process.rs  # ProcessManager 实现
    └── remote/         # SSH 远程实现（russh）
        ├── mod.rs      # 模块入口
        ├── filesystem.rs     # FileProvider 实现（基于 SFTP）
        └── process.rs  # ProcessManager 实现（基于 SSH channel）
```

## 平台依赖

### 本地

- **异步 I/O**: `tokio::fs`
- **条件编译**: `tokio::process`

### SSH (远程)

- **SSH 库**: `russh` / `russh-sftp`
- **适用场景**: 远程服务器操作

## traits.rs 公共类型定义

定义跨平台共享的类型和 trait：

### 核心类型

- **FileMetadata**: 文件元数据（大小、权限、时间戳等）
- **ProcessOutput**: 进程一次性执行结果（stdout/stderr/exit_code）
- **StreamOutput**: 进程流式输出枚举（Stdout/Stderr/Exit）
- **ProviderError**: 统一错误类型

### 核心 Trait

- **FileProvider**: 文件系统操作接口（read/write/create_dir/remove/metadata/list_dir）
- **ProcessManager**: 进程管理接口（execute/execute_stream/with_cwd/with_env）

## 实现策略

### 平台特定实现

每个平台在独立的目录下实现 `FileProvider` 和 `ProcessManager`：

#### Local (`src/local/`)

- **异步 I/O**: 基于 `tokio` 实现
- **filesystem.rs**: 使用 `tokio::fs` 实现高性能文件操作
- **process.rs**: 使用 `tokio::process` 实现异步进程管理

#### SSH (`src/remote/`)

- **SSH 库**: `russh` 和 `russh-sftp`
- **filesystem.rs**: 通过 SFTP 协议实现文件操作
- **process.rs**: 通过 SSH channel 执行远程命令
- **适用场景**: 跨平台远程操作

## lib.rs 结构

作为模块入口，根据编译目标平台条件导出对应实现：

- 导出公共 traits 和类型

## 性能考虑

1. **零拷贝**: 尽可能使用零拷贝技术减少数据复制
2. **批量操作**: 支持批量文件操作以减少系统调用
3. **异步优先**: 所有 I/O 操作都是异步的