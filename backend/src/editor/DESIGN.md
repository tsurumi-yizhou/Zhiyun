# Editor 模块设计 (Editor Design)

`editor` 模块将 UI 抽象为一个特殊的 Agent，负责管理用户的交互会话、Tab 状态以及与主线 Thread 的实时同步。

## 1. 核心功能

### 1.1 会话与 Tab 管理
- **元调用**: 支持 Tab 的创建、分割、切换和关闭。
- **状态同步**: 每一个 Tab 绑定到一个具体的 Thread，实时反映 Thread 中的内容变更。

### 1.2 文本编辑桥接
- **直接操作**: 与受限的 AI Agent 不同，Editor Agent 拥有调用底层文本编辑接口的权限，支持字符级的插入与删除。
- **冲突解决**: 处理用户编辑与 Agent 自动修改之间的实时冲突。

## 2. 核心组件
- `SessionManager`: 管理编辑器会话与活动项目。
- `TabControl`: 实现 Tab 的生命周期管理与元调用。
- `Reconciler`: 协调本地 UI 状态与 CRDT Thread 状态的一致性。

## 3. 设计原则
- **UI 即 Agent**: 遵循统一的 Agent 交互模式，但拥有更高的操作权限。
- **主线优先**: Editor 所在的 Thread 被视为系统的 Master Thread。
