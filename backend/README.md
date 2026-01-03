# Zhiyun Backend

Zhiyun 是一款基于 **“变更核心 (Change-Centric)”** 哲学设计的下一代语义化智能 IDE 后端。

## 项目愿景

Zhiyun 旨在通过无冲突复制数据类型 (CRDT) 和多层次语义图谱，为人类开发者与 AI Agent 提供一个统一、一致且高度智能的代码协作环境。

## 核心特性

- **CRDT 驱动的并发协作**: 允许多个 Agent 与用户在不同线程 (Thread) 上并行工作并无冲突合并。
- **元 AST 抽象**: 语言无关的设计，通过插件轻松支持任何编程语言。
- **智能 Routine 模型**: 类进程的任务执行模型，支持任务的递归分解与执行。
- **RAG 增强的知识层**: 结合向量检索与知识图谱，为 Agent 提供精准的代码库知识。

## 架构设计

### 1. 核心哲学：变更核心 (Change-Centric)

Zhiyun 的底层存储与同步机制建立在 **CRDT (Conflict-free Replicated Data Types)** 之上。

- **无冲突文件系统**: 文件系统被抽象为一个受版本控制的更改流。
- **Thread (线程)**: Thread 是版本化变更的主线。系统维护一条“主线 (Master Thread)”，而 Agent 或用户实验可以存在于旁路分支。
- **并发编辑**: CRDT 确保了多个智能 Agent 与人类用户在不同 Thread 上并行工作后，更改最终能达成一致并合并。
- **原子变更操作**: 禁止直接文本操作，系统内部传递具有语义意图的原子操作（如 `InsertNode`, `RenameSymbol`）。

### 2. 语言无关与元 AST (Language-Agnostic & Meta AST)

Zhiyun 坚持毫无硬编码的设计原则，不对特定编程语言作任何假设。

- **元 AST (Meta AST)**: 系统内部使用一种语言无关的统一 AST 表示。
- **插件化适配**: 通过传入 `.scm` 文件赋予 `treesitter` 解析能力，将特定语言的语法树转换为元 AST。
- **能力赋予**: 通过注册 `Skill` 赋予 Agent 能力，通过注册 `Parser` 赋予语法支持。

### 3. 运行模型：Routine 与 Thread (Routine & Thread Model)

Zhiyun 采用类进程-线程的运行模型。

- **Routine (进程)**: Agent 的 Routine 代表一系列对话和任务流。
- **元调用**: Routine 具有元调用能力，可以分支 (fork) 出新的子 Routine 来处理子任务。
- **Thread (线程)**: Thread 是版本化的更改主线。所有的实验性修改都在独立的分支 Thread 上进行，通过合并 (Merge) 而非沙箱隔离来确保主线安全。

## 项目结构

- [src/](./src/): 核心源代码目录。
  - [common/](./src/common/): 基础设施：CRDT, 元 AST 定义, 插件注册表, LLM 通信
  - [syntax/](./src/syntax/): 语法层：基于元 AST 的插件化解析引擎
  - [semantic/](./src/semantic/): 语义层：基于元 AST 的 Scope Graph 构建与符号解析
  - [compiler/](./src/compiler/): 编译器层：插件化的权威验证集成
  - [project/](./src/project/): 项目层：构建系统与依赖管理抽象
  - [knowledge/](./src/knowledge/): 知识层：RAG 流程、向量库与图数据库
  - [skill/](./src/skill/): 技能层：Agent 可调用的插件化工具集
  - [agent/](./src/agent/): Agent 运行时：Routine 生命周期、任务调度与元调用
  - [editor/](./src/editor/): Editor 运行时：Tab 管理、主线 Thread 同步

## 快速开始

本项目使用 Rust 编写，通过 Cargo 进行管理。

```bash
cargo build
cargo test
```

---
更多技术细节请参阅各子目录下的 `README.md`。
