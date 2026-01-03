use std::collections::HashMap;

/// 管理已加载的编译器插件
pub struct CompilerRegistry {
    compilers: HashMap<String, String>,
}

impl Default for CompilerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilerRegistry {
    pub fn new() -> Self {
        Self {
            compilers: HashMap::new(),
        }
    }

    /// 注册编译器
    pub fn register(&mut self, language: &str, path: &str) {
        self.compilers
            .insert(language.to_string(), path.to_string());
    }

    /// 获取编译器路径
    pub fn get_compiler(&self, language: &str) -> Option<&String> {
        self.compilers.get(language)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_registry() {
        let mut registry = CompilerRegistry::new();
        registry.register("rust", "rustc");
        assert_eq!(registry.get_compiler("rust").unwrap(), "rustc");
    }
}
