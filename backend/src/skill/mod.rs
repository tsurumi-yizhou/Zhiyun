//! # Skill 模块
//!
//! 提供语言和项目特定的技能知识库及工具调用接口。
//! 技能是从前端传递的结构化知识，并注入到 LLM 提示中，
//! 以增强模型对特定语言、工具和任务的理解。

pub mod injector;
pub mod loader;
pub mod registry;
pub mod state;
pub mod tool;
pub mod traits;
