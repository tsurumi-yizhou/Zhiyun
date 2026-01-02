//! # Skill 模块
//!
//! 提供语言和项目特定的技能知识库及工具调用接口。
//! 技能是从前端传递的结构化知识，并注入到 LLM 提示中，
//! 以增强模型对特定语言、工具和任务的理解。

mod injector;
mod loader;
mod registry;
mod state;
mod tool;
mod types;

// 重新导出公共 API
pub use injector::{InjectionConfig, SkillInjector};
pub use loader::{SkillConfig, SkillLoader};
pub use registry::SkillRegistry;
pub use state::SkillState;
pub use tool::{SkillToolRegistry, Tool, ToolOutput};
pub use types::{Skill, SkillCategory, SkillError, SkillExample, SkillId, SkillMetadata};
