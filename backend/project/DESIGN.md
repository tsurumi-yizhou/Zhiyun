# backend/project

## 概述

提供项目管理工具集，包括构建系统、测试运行、依赖管理、项目类型检测。前端传入构建命令配置，后端执行并解析结果。

## 核心职责

1. **项目类型检测** - 自动检测项目类型（Rust、Node.js、Python等）
2. **构建工具** - 调用构建系统并解析输出
3. **测试运行** - 执行测试并收集结果
4. **依赖管理** - 分析和管理项目依赖

## 文件结构

```text
src/
├── lib.rs              # 模块导出
├── detect.rs           # 项目类型检测
├── build.rs            # 构建工具
├── test.rs             # 测试运行
├── deps.rs             # 依赖管理
└── config.rs           # 构建配置定义
```

---

## lib.rs

**职责**: 模块导出，不实现功能

导出ProjectDetector、BuildTool、TestRunner、DepAnalyzer等核心类型。

---

## detect.rs

**职责**: 项目类型检测

**核心概念**:

- 根据文件系统特征检测项目类型
- 支持多种语言和框架
- 返回项目配置和元信息

**核心类型**:

- `ProjectDetector`: 检测器
- `ProjectType`: 项目类型枚举（Rust、NodeJS、Python、Go等）
- `ProjectInfo`: 项目信息，包含type、root、config_files

**主要方法**:

- `detect()`: 检测项目类型
- `find_project_root()`: 查找项目根目录
- `get_config_files()`: 获取配置文件列表

**检测规则**:

- Rust: 存在Cargo.toml
- Node.js: 存在package.json
- Python: 存在requirements.txt或pyproject.toml
- Go: 存在go.mod

**测试要点**:

- 检测准确
- 多语言项目正确识别
- 嵌套项目找到最近的根

---

## build.rs

**职责**: 构建工具

**核心概念**:

- 前端传入构建命令配置
- 执行构建并解析输出
- 支持增量构建和并行构建

**核心类型**:

- `BuildTool`: 构建工具
- `BuildConfig`: 构建配置（命令、参数、环境）
- `BuildResult`: 构建结果，包含success、errors、warnings、artifacts
- `BuildCommand`: 构建命令（Shell、Custom）

**主要方法**:

- `build()`: 执行构建
- `build_with_target()`: 构建特定目标
- `parse_output()`: 解析构建输出
- `get_errors()`: 提取错误信息
- `get_warnings()`: 提取警告信息

**支持的构建系统**:

- Cargo (Rust)
- NPM/Yarn/PNPM (Node.js)
- Maven/Gradle (Java)
- Pip/Poetry (Python)
- Go build (Go)

**测试要点**:

- 构建执行成功
- 错误提取准确
- 并行构建不冲突

---

## test.rs

**职责**: 测试运行器

**核心概念**:

- 运行项目测试套件
- 收集测试结果（通过、失败、跳过）
- 支持特定测试过滤

**核心类型**:

- `TestRunner`: 测试运行器
- `TestConfig`: 测试配置
- `TestResult`: 测试结果，包含passed、failed、skipped、coverage
- `TestCase`: 单个测试用例

**主要方法**:

- `run_all()`: 运行所有测试
- `run_test()`: 运行特定测试
- `parse_test_output()`: 解析测试输出
- `get_coverage()`: 获取代码覆盖率

**测试框架支持**:

- Cargo test (Rust)
- Jest/Vitest (TypeScript)
- Pytest (Python)
- Go test (Go)

**测试要点**:

- 测试运行成功
- 结果解析准确
- 覆盖率计算正确

---

## deps.rs

**职责**: 依赖分析器

**核心概念**:

- 分析项目依赖
- 检测循环依赖
- 生成依赖图

**核心类型**:

- `DepAnalyzer`: 依赖分析器
- `Dependency`: 依赖，包含name、version、source
- `DepGraph`: 依赖图

**主要方法**:

- `analyze_deps()`: 分析所有依赖
- `find_unused()`: 查找未使用的依赖
- `detect_outdated()`: 检测过时的依赖
- `check_cycles()`: 检查循环依赖
- `get_dep_tree()`: 获取依赖树

**测试要点**:

- 依赖提取完整
- 循环依赖检测正确
- 未使用依赖识别准确

---

## config.rs

**职责**: 构建配置定义

**核心概念**:

- 前端传入如何构建/测试的配置
- 支持自定义命令
- 可扩展的输出解析器

**核心类型**:

- `ProjectConfig`: 项目配置
- `ToolConfig`: 工具配置（构建、测试）
- `OutputParser`: 输出解析器

**主要方法**:

- `from_file()`: 从配置文件加载
- `get_build_command()`: 获取构建命令
- `get_test_command()`: 获取测试命令

**测试要点**:

- 配置加载正确
- 命令生成准确

---

## 依赖

```toml
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
async-trait = { workspace = true }
```

---

## 使用场景

1. **增量开发**: Agent修改代码后运行测试验证
   - TestRunner.run_test()

2. **错误诊断**: 构建失败时提取错误信息
   - BuildTool.build() + get_errors()

3. **依赖优化**: 查找未使用的依赖
   - DepAnalyzer.find_unused()

4. **跨语言项目**: 检测项目类型并选择正确的构建工具
   - ProjectDetector.detect()

---

## 使用流程

1. 检测项目类型
2. 加载构建配置
3. 执行构建
4. 解析输出，提取错误/警告
5. 如果需要，运行测试
6. 返回结果给Agent
