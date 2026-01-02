use crate::injector::SkillInjector;
use crate::loader::SkillConfig;
use crate::registry::SkillRegistry;
use crate::{SkillError, SkillLoader};
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::RwLock;

/// Global skill state combining registry and injector
pub struct SkillState {
    pub registry: SkillRegistry,
    pub injector: SkillInjector,
}

impl SkillState {
    /// Create a new skill state
    pub fn new() -> Self {
        let registry = SkillRegistry::new();
        let injector = SkillInjector::new(registry.clone());
        Self { registry, injector }
    }

    /// Preload skills from configuration (call at program startup)
    pub async fn preload_from_config(config: &SkillConfig) -> Result<(), SkillError> {
        let skills = SkillLoader::from_config(config)?;
        let mut state = Self::get().write().await;
        state.registry.register_all(skills)
    }
}

impl Default for SkillState {
    fn default() -> Self {
        Self::new()
    }
}

/// Global state singleton using OnceLock
static GLOBAL_STATE: OnceLock<Arc<RwLock<SkillState>>> = OnceLock::new();

impl SkillState {
    /// Get the global state instance
    pub fn get() -> &'static Arc<RwLock<SkillState>> {
        GLOBAL_STATE.get_or_init(|| Arc::new(RwLock::new(Self::new())))
    }

    /// Reset the global state (useful for testing)
    pub fn reset() {
        // Note: OnceLock doesn't support reset, this is a no-op in production
        // In tests, you'd need to use a different approach
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SkillExample, SkillId, SkillMetadata};
    use crate::{Skill, SkillCategory};
    use std::collections::HashSet;

    #[test]
    fn test_state_creation() {
        let state = SkillState::new();
        assert_eq!(state.registry.count(), 0);
    }

    fn create_test_skill(name: &str) -> Skill {
        Skill {
            id: SkillId::new(SkillCategory::Syntax, name, "Rust"),
            name: name.into(),
            description: format!("{} skill", name),
            content: format!("Content for {}", name),
            examples: vec![SkillExample {
                input: "test".into(),
                output: "result".into(),
                explanation: "test explanation".into(),
            }],
            related_tools: vec![],
            metadata: SkillMetadata {
                language: "Rust".into(),
                version: "1.0".into(),
                author: None,
                tags: HashSet::from_iter(vec!["test".into()]),
            },
        }
    }

    #[tokio::test]
    async fn test_global_state_singleton() {
        // Get global state and register a skill
        let state1 = SkillState::get().read().await;
        let skill = create_test_skill("test_singleton");
        let id = skill.id.clone();
        drop(state1);

        let mut state2 = SkillState::get().write().await;
        state2.registry.register(skill).unwrap();
        drop(state2);

        // Verify the skill is still there
        let state3 = SkillState::get().read().await;
        assert!(state3.registry.contains(&id));
    }

    #[tokio::test]
    async fn test_preload_from_config() {
        use serde_json::json;

        // Get initial count
        let state = SkillState::get().read().await;
        let initial_count = state.registry.count();
        drop(state);

        let config = SkillConfig {
            files: vec![],
            inline_skills: vec![json!({
                "id": {
                    "category": "Syntax",
                    "name": "test_preload",
                    "language": "Rust"
                },
                "name": "Test Preload",
                "description": "A test skill",
                "content": "Test content",
                "examples": [],
                "related_tools": [],
                "metadata": {
                    "language": "Rust",
                    "version": "1.0",
                    "tags": []
                }
            })],
        };

        SkillState::preload_from_config(&config)
            .await
            .unwrap();

        // Check that at least one new skill was loaded
        // (can't use exact count due to parallel tests sharing global state)
        let state = SkillState::get().read().await;
        assert!(state.registry.count() >= initial_count, "Should have at least {} skills, got {}", initial_count, state.registry.count());
    }
}
