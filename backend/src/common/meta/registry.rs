use crate::common::meta::plugin::Plugin;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

lazy_static! {
    /// 全局插件注册表
    pub static ref GLOBAL_REGISTRY: PluginRegistry = PluginRegistry::new();
}

/// 插件注册表，用于管理所有已加载的插件
pub struct PluginRegistry {
    plugins: Arc<RwLock<HashMap<String, Arc<dyn Plugin>>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Mock 注册插件
    pub fn register(&self, plugin: Arc<dyn Plugin>) {
        let mut plugins = self.plugins.write().unwrap();
        plugins.insert(plugin.name().to_string(), plugin);
    }

    /// Mock 获取插件
    pub fn get(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(name).cloned()
    }

    /// Mock 获取所有插件名称
    pub fn list_plugin_names(&self) -> Vec<String> {
        let plugins = self.plugins.read().unwrap();
        plugins.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::meta::plugin::Plugin;

    struct MockPlugin {
        name: String,
    }

    impl Plugin for MockPlugin {
        fn name(&self) -> &str {
            &self.name
        }
        fn version(&self) -> &str {
            "1.0.0"
        }
    }

    #[test]
    fn test_registry_mock() {
        let registry = PluginRegistry::new();
        let plugin = Arc::new(MockPlugin {
            name: "test-plugin".to_string(),
        });

        registry.register(plugin.clone());

        let found = registry.get("test-plugin");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "test-plugin");

        let names = registry.list_plugin_names();
        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "test-plugin");
    }
}
