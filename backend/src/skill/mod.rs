//! # Skill Module
//!
//! Provides language and project-specific skill knowledge base with tool call interface.
//! Skills are structured knowledge passed from the frontend and injected
//! into LLM prompts to enhance model understanding of specific languages,
//! tools, and tasks.

mod injector;
mod loader;
mod registry;
mod state;
mod tool;
mod types;

// Re-export public API
pub use injector::{InjectionConfig, SkillInjector};
pub use loader::{SkillConfig, SkillLoader};
pub use registry::SkillRegistry;
pub use state::SkillState;
pub use tool::{SkillToolRegistry, Tool, ToolOutput};
pub use types::{Skill, SkillCategory, SkillError, SkillExample, SkillId, SkillMetadata};
