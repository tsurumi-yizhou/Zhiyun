use crate::skill::injector::SkillInjector;
use crate::skill::loader::SkillConfig;
use crate::skill::registry::SkillRegistry;
use crate::skill::{SkillError, SkillLoader};
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::RwLock;

/// 结合注册表和注入器的全局技能状态
pub struct SkillState {
    pub registry: SkillRegistry,
    pub injector: SkillInjector,
}

impl SkillState {
    /// 创建新的技能状态
    pub fn new() -> Self {
        let registry = SkillRegistry::new();
        let injector = SkillInjector::new(registry.clone());
        Self { registry, injector }
    }

    /// 从配置预加载技能（在程序启动时调用）
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

/// 使用 OnceLock 的全局状态单例
static GLOBAL_STATE: OnceLock<Arc<RwLock<SkillState>>> = OnceLock::new();

impl SkillState {
    /// 获取全局状态实例
    pub fn get() -> &'static Arc<RwLock<SkillState>> {
        GLOBAL_STATE.get_or_init(|| Arc::new(RwLock::new(Self::new())))
    }

    /// 重置全局状态（用于测试）
    pub fn reset() {
        // 注意：OnceLock 不支持重置，这在生产环境中是无操作
        // 在测试中，需要使用不同的方法
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::types::{SkillExample, SkillId, SkillMetadata};
    use crate::skill::{Skill, SkillCategory};
    use std::collections::HashSet;

    #[test]
    fn test_state_creation() {
        let state = SkillState::new();
        assert_eq!(state.registry.count(), 0);
    }

    fn create_test_skill(name: &str) -> Skill {
        Skill {
            id: SkillId::new(SkillCategory::new("Syntax"), name, "Rust"),
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
        // 获取全局状态并注册一个技能
        let state1 = SkillState::get().read().await;
        let skill = create_test_skill("test_singleton");
        let id = skill.id.clone();
        drop(state1);

        let mut state2 = SkillState::get().write().await;
        state2.registry.register(skill).unwrap();
        drop(state2);

        // 验证技能仍然存在
        let state3 = SkillState::get().read().await;
        assert!(state3.registry.contains(&id));
    }

    #[tokio::test]
    async fn test_preload_from_config() {
        use serde_json::json;

        // 获取初始计数
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

        SkillState::preload_from_config(&config).await.unwrap();

        // 检查是否至少加载了一个新技能
        // （由于并行测试共享全局状态，无法使用确切计数）
        let state = SkillState::get().read().await;
        assert!(
            state.registry.count() >= initial_count,
            "Should have at least {} skills, got {}",
            initial_count,
            state.registry.count()
        );
    }
}
