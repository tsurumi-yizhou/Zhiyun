use crate::common::provider::traits::{StorageProvider, FileMetadata};
use async_trait::async_trait;

pub struct RemoteFileSystem;

#[async_trait]
impl StorageProvider for RemoteFileSystem {
    fn id(&self) -> &str {
        "remote-fs"
    }

    async fn read_file(&self, _path: &str) -> anyhow::Result<Vec<u8>> {
        // Mock: 远程读取逻辑（如通过 SSH/HTTP）
        Ok(vec![])
    }

    async fn write_file(&self, _path: &str, _content: &[u8]) -> anyhow::Result<()> {
        // Mock: 远程写入逻辑
        Ok(())
    }

    async fn delete(&self, _path: &str, _recursive: bool) -> anyhow::Result<()> {
        Ok(())
    }

    async fn list_dir(&self, _path: &str) -> anyhow::Result<Vec<FileMetadata>> {
        Ok(vec![])
    }

    async fn get_metadata(&self, path: &str) -> anyhow::Result<FileMetadata> {
        Ok(FileMetadata {
            path: path.to_string(),
            size: 0,
            is_dir: false,
            modified_at: 0,
            created_at: 0,
        })
    }

    async fn exists(&self, _path: &str) -> anyhow::Result<bool> {
        Ok(true)
    }

    async fn create_dir(&self, _path: &str, _recursive: bool) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_remote_fs_mock() {
        let fs = RemoteFileSystem;
        assert_eq!(fs.id(), "remote-fs");
    }
}
