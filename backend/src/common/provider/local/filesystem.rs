use crate::common::provider::traits::{FileMetadata, StorageProvider};
use async_trait::async_trait;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use tokio::fs;

pub struct LocalFileSystem {
    base_path: PathBuf,
}

impl LocalFileSystem {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    fn full_path(&self, path: &str) -> PathBuf {
        let path = path.trim_start_matches('/').trim_start_matches('\\');
        self.base_path.join(path)
    }
}

#[async_trait]
impl StorageProvider for LocalFileSystem {
    fn id(&self) -> &str {
        "local-fs"
    }

    async fn read_file(&self, path: &str) -> anyhow::Result<Vec<u8>> {
        let full_path = self.full_path(path);
        Ok(fs::read(full_path).await?)
    }

    async fn write_file(&self, path: &str, content: &[u8]) -> anyhow::Result<()> {
        let full_path = self.full_path(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(full_path, content).await?;
        Ok(())
    }

    async fn delete(&self, path: &str, recursive: bool) -> anyhow::Result<()> {
        let full_path = self.full_path(path);
        let meta = fs::metadata(&full_path).await?;
        if meta.is_dir() {
            if recursive {
                fs::remove_dir_all(full_path).await?;
            } else {
                fs::remove_dir(full_path).await?;
            }
        } else {
            fs::remove_file(full_path).await?;
        }
        Ok(())
    }

    async fn list_dir(&self, path: &str) -> anyhow::Result<Vec<FileMetadata>> {
        let full_path = self.full_path(path);
        let mut entries = fs::read_dir(full_path).await?;
        let mut result = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let meta = entry.metadata().await?;
            let path = entry
                .path()
                .strip_prefix(&self.base_path)?
                .to_string_lossy()
                .into_owned();

            result.push(FileMetadata {
                path,
                size: meta.len(),
                is_dir: meta.is_dir(),
                modified_at: meta.modified()?.duration_since(UNIX_EPOCH)?.as_secs(),
                created_at: meta
                    .created()
                    .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
                    .unwrap_or(0),
            });
        }
        Ok(result)
    }

    async fn get_metadata(&self, path: &str) -> anyhow::Result<FileMetadata> {
        let full_path = self.full_path(path);
        let meta = fs::metadata(&full_path).await?;
        Ok(FileMetadata {
            path: path.to_string(),
            size: meta.len(),
            is_dir: meta.is_dir(),
            modified_at: meta.modified()?.duration_since(UNIX_EPOCH)?.as_secs(),
            created_at: meta
                .created()
                .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
                .unwrap_or(0),
        })
    }

    async fn exists(&self, path: &str) -> anyhow::Result<bool> {
        let full_path = self.full_path(path);
        Ok(full_path.exists())
    }

    async fn create_dir(&self, path: &str, recursive: bool) -> anyhow::Result<()> {
        let full_path = self.full_path(path);
        if recursive {
            fs::create_dir_all(full_path).await?;
        } else {
            fs::create_dir(full_path).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_local_fs_operations() {
        let dir = tempdir().unwrap();
        let fs = LocalFileSystem::new(dir.path());

        // 测试写入和读取
        fs.write_file("test.txt", b"hello world").await.unwrap();
        let content = fs.read_file("test.txt").await.unwrap();
        assert_eq!(content, b"hello world");

        // 测试元数据
        let meta = fs.get_metadata("test.txt").await.unwrap();
        assert_eq!(meta.size, 11);
        assert!(!meta.is_dir);

        // 测试列出目录
        let list = fs.list_dir("").await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].path, "test.txt");

        // 测试删除
        fs.delete("test.txt", false).await.unwrap();
        assert!(!fs.exists("test.txt").await.unwrap());
    }
}
