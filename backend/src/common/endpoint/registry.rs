use crate::common::endpoint::traits::ModelInfo;
use std::collections::HashMap;

pub struct ModelRegistry {
    models: HashMap<String, ModelInfo>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    pub fn register(&mut self, model: ModelInfo) {
        self.models.insert(model.id.clone(), model);
    }

    pub fn list(&self) -> Vec<&ModelInfo> {
        self.models.values().collect()
    }

    pub fn list_by_provider(&self, provider: &str) -> Vec<&ModelInfo> {
        self.models.values().filter(|m| m.provider == provider).collect()
    }
}

/// 提供者注册表
pub struct ProviderRegistry {
    providers: HashMap<String, crate::common::endpoint::traits::ProviderInfo>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self { providers: HashMap::new() }
    }

    pub fn register(&mut self, provider: crate::common::endpoint::traits::ProviderInfo) {
        self.providers.insert(provider.id.clone(), provider);
    }

    pub fn get(&self, id: &str) -> Option<&crate::common::endpoint::traits::ProviderInfo> {
        self.providers.get(id)
    }
}

pub struct FileManager;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::endpoint::traits::ModelInfo;

    #[test]
    fn test_registry_mock() {
        let mut registry = ModelRegistry::new();
        registry.register(ModelInfo {
            id: "gpt-4".to_string(),
            name: "GPT-4".to_string(),
            provider: "openai".to_string(),
            context_window: 128000,
            supports_vision: true,
            supports_tools: true,
        });

        assert!(registry.list_by_provider("openai").len() == 1);
    }
}
