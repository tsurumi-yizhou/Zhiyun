use crate::common::meta::ast::MetaNode;
use async_trait::async_trait;
use lazy_static::lazy_static;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

lazy_static! {
    /// 全局服务管理器
    pub static ref GLOBAL_SERVICE_MANAGER: ServiceManager = ServiceManager::new();
}

/// 服务接口，所有后台服务都需要实现此接口
#[async_trait]
pub trait Service: Send + Sync + Any {
    /// 获取服务名称
    fn name(&self) -> &str;

    /// 执行服务调用
    async fn call(&self, input: MetaNode) -> anyhow::Result<MetaNode>;

    /// 将服务转换为 Any，用于向下转型
    fn as_any(&self) -> &dyn Any;
}

/// 服务管理器，负责服务的注册和发现
pub struct ServiceManager {
    services: Arc<RwLock<HashMap<String, Arc<dyn Service>>>>,
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceManager {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Mock 注册服务
    pub fn register(&self, service: Arc<dyn Service>) {
        let mut services = self.services.write().unwrap();
        services.insert(service.name().to_string(), service);
    }

    /// Mock 获取服务
    pub fn get(&self, name: &str) -> Option<Arc<dyn Service>> {
        let services = self.services.read().unwrap();
        services.get(name).cloned()
    }

    /// Mock 调用服务
    pub async fn call(&self, name: &str, input: MetaNode) -> anyhow::Result<MetaNode> {
        let service = self
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Service not found: {}", name))?;
        service.call(input).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::meta::ast::MetaNode;

    struct MockService;

    #[async_trait]
    impl Service for MockService {
        fn name(&self) -> &str {
            "mock-service"
        }

        async fn call(&self, input: MetaNode) -> anyhow::Result<MetaNode> {
            // 简单的 Mock 逻辑：返回输入本身
            Ok(input)
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[tokio::test]
    async fn test_service_mock() {
        let manager = ServiceManager::new();
        let service = Arc::new(MockService);

        manager.register(service);

        let input = MetaNode::identifier("test");
        let result = manager.call("mock-service", input.clone()).await.unwrap();

        assert_eq!(result, input);
    }
}
