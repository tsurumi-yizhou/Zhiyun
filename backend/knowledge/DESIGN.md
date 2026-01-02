# backend/knowledge

## 概述

提供知识库工具集，包括符号引用图（知识图谱）和向量数据库（RAG记忆）。用于索引代码关系、检索相关上下文、长期记忆存储。

## 核心职责

1. **符号引用图** - 构建和维护符号之间的引用关系
2. **向量数据库** - 存储和检索代码的向量表示
3. **RAG检索** - 结合图和向量进行语义搜索
4. **长期记忆** - 存储Agent的历史交互和知识

## 文件结构

```text
src/
├── lib.rs              # 模块导出
├── graph.rs            # 符号引用图
├── vector.rs           # 向量数据库接口
├── rag.rs              # RAG检索
└── memory.rs           # 长期记忆
```

---

## lib.rs

**职责**: 模块导出，不实现功能

导出SymbolGraph、VectorDB、RAGService、MemoryStore等核心类型。

---

## graph.rs

**职责**: 符号引用图（知识图谱）

**核心概念**:

- 节点是符号定义（函数、类、变量等）
- 边是引用关系（调用、继承、类型引用等）
- 有向图，支持多种边类型

**核心类型**:

- `SymbolGraph`: 图结构，使用petgraph::DiGraph
- `Symbol`: 符号定义，包含id、name、kind、file、span、embedding
- `ReferenceEdge`: 引用边，包含kind（Calls、InheritsFrom、TypeOf等）、count
- `RefKind`: 引用类型枚举

**主要方法**:

- `from_symbols()`: 从符号列表构建图
- `add_symbol()`: 添加符号节点
- `add_reference()`: 添加引用边
- `find_dependencies()`: 查找符号的依赖
- `find_dependents()`: 查找依赖该符号的其他符号
- `get_call_chain()`: 获取调用链
- `get_inheritance_tree()`: 获取继承树

**查询操作**:

- 查找函数的所有调用者
- 查找类的所有子类
- 查找符号的所有引用位置
- 分析代码影响范围

**测试要点**:

- 图构建正确
- 引用关系准确
- 查询结果完整

---

## vector.rs

**职责**: 向量数据库接口

**核心概念**:

- 存储代码、文档、历史的向量表示
- 支持语义搜索（相似度检索）
- 可插拔的向量数据库后端

**核心类型**:

- `VectorDB`: 向量数据库trait
- `Embedding`: 向量表示
- `SearchResult`: 搜索结果，包含id、score、metadata
- `VectorStoreConfig`: 存储配置

**主要方法**:

- `insert()`: 插入向量
- `search()`: 相似度搜索
- `delete()`: 删除向量
- `update()`: 更新向量

**支持的后端**:

- 内存向量存储（简单实现）
- Qdrant（生产推荐）
- DiskANN（大规模）
- Pgvector（PostgreSQL）

**测试要点**:

- 向量插入成功
- 搜索返回相关结果
- 相似度排序正确

---

## rag.rs

**职责**: RAG（检索增强生成）服务

**核心概念**:

- 结合符号图和向量数据库
- 为Agent提供相关上下文
- 支持多种检索策略

**核心类型**:

- `RAGService`: RAG服务
- `RetrievalStrategy`: 检索策略（GraphOnly、VectorOnly、Hybrid）
- `ContextFragment`: 上下文片段，包含content、source、relevance_score

**主要方法**:

- `retrieve_context()`: 检索相关上下文
- `search_by_query()`: 按查询搜索
- `search_by_symbols()`: 按符号搜索
- `hybrid_search()`: 混合搜索（图+向量）

**检索策略**:

1. GraphOnly: 使用符号图查找相关符号
2. VectorOnly: 使用向量数据库语义搜索
3. Hybrid: 结合图和向量（推荐）

**混合检索流程**:

1. 向量搜索找到相关的代码片段
2. 符号图扩展查找相关符号
3. 合并结果，去重，排序
4. 返回Top K个上下文片段

**测试要点**:

- 检索结果相关
- 混合策略优于单一策略
- 性能可接受

---

## memory.rs

**职责**: 长期记忆存储

**核心概念**:

- 存储Agent的历史交互
- 存储用户的反馈和偏好
- 支持记忆的检索和更新

**核心类型**:

- `MemoryStore`: 记忆存储
- `MemoryFragment`: 记忆片段，包含id、content、timestamp、tags
- `MemoryType`: 记忆类型（Conversation、Feedback、Knowledge）

**主要方法**:

- `store()`: 存储记忆
- `retrieve()`: 检索相关记忆
- `update()`: 更新记忆
- `forget()`: 删除记忆
- `search_by_tags()`: 按标签搜索

**记忆类型**:

- Conversation: 对话历史
- Feedback: 用户反馈
- Knowledge: 知识点
- Pattern: 代码模式

**测试要点**:

- 记忆存储成功
- 检索准确
- 标签过滤有效

---

## 依赖

```toml
petgraph = "0.6"
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
async-trait = { workspace = true }
```

可选依赖（向量数据库）:

```toml
qdrant-client = "1.0"  # 可选
```

---

## 使用场景

1. **代码理解**: 用户问"这个函数在哪里被调用？"
   - 查询符号图的引用关系

2. **上下文检索**: Agent需要相关代码作为参考
   - RAG检索相关代码片段

3. **影响分析**: 修改这个函数会影响什么？
   - 查询符号图的依赖关系

4. **长期记忆**: Agent记住用户的偏好
   - 存储到记忆库

---

## 使用流程

1. 从符号表构建符号图
2. 为代码片段生成向量并存储
3. Agent需要上下文时，调用RAG检索
4. 返回相关的代码片段和符号信息
5. Agent使用这些信息生成响应
