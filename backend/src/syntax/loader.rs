use crate::common::provider::traits::StorageProvider;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

/// 动态加载不同语言的语法文件和 SCM 查询
pub struct GrammarLoader {
    storage: Arc<dyn StorageProvider>,
    // 模拟语法文件存储路径映射
    grammars: HashMap<String, String>,
}

impl GrammarLoader {
    pub fn new(storage: Arc<dyn StorageProvider>) -> Self {
        Self {
            storage,
            grammars: HashMap::new(),
        }
    }

    /// 加载语法定义
    pub async fn load_grammar(&mut self, language: &str, path: &str) -> Result<()> {
        // 检查语法文件是否存在，屏蔽平台细节
        if self.storage.exists(path).await? {
            self.grammars.insert(language.to_string(), path.to_string());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Grammar file not found: {}", path))
        }
    }

    /// 获取语法定义路径
    pub fn get_grammar_path(&self, language: &str) -> Option<&String> {
        self.grammars.get(language)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::provider::traits::FileMetadata;
    use async_trait::async_trait;

    struct MockStorage;
    #[async_trait]
    impl StorageProvider for MockStorage {
        fn id(&self) -> &str {
            "mock"
        }
        async fn read_file(&self, _path: &str) -> Result<Vec<u8>> {
            Ok(vec![])
        }
        async fn write_file(&self, _path: &str, _content: &[u8]) -> Result<()> {
            Ok(())
        }
        async fn delete(&self, _path: &str, _recursive: bool) -> Result<()> {
            Ok(())
        }
        async fn list_dir(&self, _path: &str) -> Result<Vec<FileMetadata>> {
            Ok(vec![])
        }
        async fn get_metadata(&self, _path: &str) -> Result<FileMetadata> {
            Ok(FileMetadata {
                path: _path.to_string(),
                size: 0,
                is_dir: false,
                modified_at: 0,
                created_at: 0,
            })
        }
        async fn exists(&self, _path: &str) -> Result<bool> {
            Ok(true)
        }
        async fn create_dir(&self, _path: &str, _recursive: bool) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_grammar_loader() {
        let storage = Arc::new(MockStorage);
        let mut loader = GrammarLoader::new(storage);
        loader
            .load_grammar("rust", "/path/to/rust.so")
            .await
            .unwrap();
        assert_eq!(loader.get_grammar_path("rust").unwrap(), "/path/to/rust.so");
    }
}
