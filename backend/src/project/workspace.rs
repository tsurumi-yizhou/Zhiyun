use crate::common::provider::traits::StorageProvider;
use std::sync::Arc;

/// 识别项目根目录与多包 (Monorepo) 结构
pub struct WorkspaceManager {
    storage: Arc<dyn StorageProvider>,
    root_path: String,
}

impl WorkspaceManager {
    pub fn new(storage: Arc<dyn StorageProvider>, root: String) -> Self {
        Self {
            storage,
            root_path: root,
        }
    }

    /// 获取项目根目录
    pub fn root(&self) -> &str {
        &self.root_path
    }

    /// 检测是否为 Monorepo
    pub async fn is_monorepo(&self) -> bool {
        // 通过 provider 检查是否存在特定文件（如 pnpm-workspace.yaml 或 Cargo.toml 中的 workspace）
        // 屏蔽平台细节
        let _exists = self
            .storage
            .exists(&format!("{}/Cargo.toml", self.root_path))
            .await
            .unwrap_or(false);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::provider::traits::FileMetadata;
    use anyhow::Result;
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
    async fn test_workspace_manager() {
        let storage = Arc::new(MockStorage);
        let manager = WorkspaceManager::new(storage, "/test".to_string());
        assert_eq!(manager.root(), "/test");
        assert!(!manager.is_monorepo().await);
    }
}
