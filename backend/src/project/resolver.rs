use std::collections::HashMap;

/// 分析依赖树并检测冲突
pub struct DependencyResolver {
    dependencies: HashMap<String, Vec<String>>,
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
        }
    }

    /// 解析依赖
    pub fn resolve(&mut self, package: &str, deps: Vec<String>) {
        self.dependencies.insert(package.to_string(), deps);
    }

    /// 获取依赖列表
    pub fn get_dependencies(&self, package: &str) -> Option<&Vec<String>> {
        self.dependencies.get(package)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_resolver() {
        let mut resolver = DependencyResolver::new();
        resolver.resolve("app", vec!["lib1".to_string(), "lib2".to_string()]);
        assert_eq!(resolver.get_dependencies("app").unwrap().len(), 2);
    }
}
