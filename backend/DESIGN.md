# Zhiyun 后端架构设计 (Zhiyun Backend Architecture)

Zhiyun 是一款基于 **“变更核心 (Change-Centric)”** 哲学设计的下一代语义化智能 IDE。它彻底抛弃了传统的“以文本为中心”的编辑模式，转而通过无冲突复制数据类型 (CRDT) 和多层次语义图谱，为人类开发者与 AI Agent 提供一个统一、一致且高度智能的代码协作环境。

---

## 1. 核心哲学：变更核心 (Change-Centric)

Zhiyun 的底层存储与同步机制建立在 **CRDT (Conflict-free Replicated Data Types)** 之上。

### 1.1 无冲突文件系统
- **统一抽象**: 文件系统被抽象为一个受版本控制的更改流。
- **Thread (线程)**: Thread 是版本化变更的主线。系统维护一条“主线 (Master Thread)”，而 Agent 或用户实验可以存在于旁路分支。
- **并发编辑**: CRDT 确保了多个智能 Agent 与人类用户在不同 Thread 上并行工作后，更改最终能达成一致并合并。

### 1.2 原子变更操作
- **禁止直接文本操作**: 系统内部传递具有语义意图的原子操作（如 `InsertNode`, `RenameSymbol`），而非脆弱的行/列字符偏移。

---

## 2. 语言无关与元 AST (Language-Agnostic & Meta AST)

Zhiyun 坚持毫无硬编码的设计原则，不对特定编程语言作任何假设。

### 2.1 元 AST (Meta AST)
- **统一表示**: 系统内部使用一种语言无关的统一 AST 表示。
- **插件化适配**: 通过传入 `.scm` 文件赋予 `treesitter` 解析能力，将特定语言的语法树转换为元 AST。

### 2.2 插件化架构
- **能力赋予**: 通过注册 `Skill` 赋予 Agent 能力，通过注册 `Parser` 赋予语法支持。
- **无硬编码**: 后端核心不包含任何语言特定的硬逻辑。

---

## 3. 运行模型：Routine 与 Thread (Routine & Thread Model)

Zhiyun 采用类进程-线程的运行模型。

### 3.1 Routine (进程)
- **智能进程**: Agent 的 Routine 代表一系列对话和任务流。
- **元调用**: Routine 具有元调用能力，可以分支 (fork) 出新的子 Routine 来处理子任务。
- **多对多关系**: 一个 Routine 可以操作多个 Thread，一个 Thread 也可以被多个 Routine 访问。

### 3.2 Thread (线程)
- **变更流**: Thread 是版本化的更改主线。
- **无需沙箱**: 所有的实验性修改都在独立的分支 Thread 上进行，通过合并 (Merge) 而非沙箱隔离来确保主线安全。

---

## 4. 模块化设计原则 (Modularity Principles)

- **高内聚低耦合**: 模块间除了共同依赖 `common` 基础设施外，彼此不产生直接依赖。
- **服务注册**: 模块间通过 `common/meta/registry` 发现和调用彼此的能力。
- **UI 作为 Editor Agent**: 用户界面被视为一个特殊的 Agent (Editor)，拥有操作 Tab 和主线 Thread 的特殊元调用权限。

---

## 5. 模块结构 (Module Structure)

```
backend/src/
├── common/          # 基础设施：CRDT, 元 AST 定义, 插件注册表, LLM 通信
├── syntax/          # 语法层：基于元 AST 的插件化解析引擎
├── semantic/        # 语义层：基于元 AST 的 Scope Graph 构建与符号解析
├── compiler/        # 编译器层：插件化的权威验证集成
├── project/         # 项目层：构建系统与依赖管理抽象
├── knowledge/       # 知识层：RAG 流程、向量库与图数据库
├── skill/           # 技能层：Agent 可调用的插件化工具集
├── agent/           # Agent 运行时：Routine 生命周期、任务调度与元调用
└── editor/          # Editor 运行时：Tab 管理、主线 Thread 同步
```
