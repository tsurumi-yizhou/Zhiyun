# Editor 模块 (Editor Runtime)

`editor` 模块将 UI 抽象为一个特殊的 Agent，负责管理用户的交互会话、Tab 状态以及与主线 Thread 的实时同步。

## 核心功能

- **会话与 Tab 管理**: 支持 Tab 的创建、分割、切换和关闭。
- **状态同步**: 每一个 Tab 绑定到一个具体的 Thread，实时反映 Thread 中的内容变更。
- **文本编辑桥接**: 与受限的 AI Agent 不同，Editor Agent 拥有调用底层文本编辑接口的权限，支持字符级的插入与删除。

## 核心组件

- [session.rs](./session.rs): `SessionManager` 管理编辑器会话与活动项目。
- [tab.rs](./tab.rs): `TabControl` 实现 Tab 的生命周期管理与元调用。
- [reconciler.rs](./reconciler.rs): `Reconciler` 协调本地 UI 状态与 CRDT Thread 状态的一致性。

## 设计原则

- **UI 即 Agent**: 遵循统一的 Agent 交互模式，但拥有更高的操作权限。
- **主线优先**: Editor 所在的 Thread 被视为系统的 Master Thread。

## 架构设计细节

### 1. UI 作为特殊 Agent
- **权限模型**: 不同于普通的 AI Agent，Editor Agent 拥有操作 Master Thread 的核心权限，支持实时的字符级文本编辑。
- **元调用支持**: 编辑器的交互动作（如分屏、Tab 切换）被抽象为统一的元调用命令。

### 2. 状态调和机制
- **Thread 绑定**: 每个 Tab 实时绑定到一个 CRDT Thread。
- **Reconciler**: 负责高效调和本地编辑器 UI 状态与底层的无冲突变更流，确保用户感知的零延迟同步。

### 3. 会话管理
- **Master Thread 优先**: 编辑器所在的 Thread 被系统视为唯一真理来源的主线，所有的合并操作最终都以主线一致性为目标。

---
更多技术细节请参阅源代码。
