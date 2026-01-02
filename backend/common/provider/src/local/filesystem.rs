use crate::traits::{FileMetadata, FileProvider, ProviderError};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// 本地文件系统提供者
///
/// 基于 `tokio::fs` 实现高性能的异步文件系统操作
#[derive(Debug, Clone)]
pub struct LocalFileProvider {
    /// 工作目录（可选，用于相对路径解析）
    work_dir: Option<PathBuf>,
}

impl LocalFileProvider {
    /// 创建新的本地文件提供者
    pub fn new() -> Self {
        Self { work_dir: None }
    }

    /// 创建带工作目录的本地文件提供者
    pub fn with_work_dir(work_dir: PathBuf) -> Self {
        Self {
            work_dir: Some(work_dir),
        }
    }

    /// 解析路径（将相对路径转换为绝对路径）
    fn resolve_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(ref work_dir) = self.work_dir {
            work_dir.join(path)
        } else {
            path.to_path_buf()
        }
    }
}

impl Default for LocalFileProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileProvider for LocalFileProvider {
    async fn read(&self, path: &Path) -> Result<Vec<u8>, ProviderError> {
        let resolved = self.resolve_path(path);
        fs::read(&resolved)
            .await
            .map_err(|_e| ProviderError::NotFound {
                path: resolved.display().to_string(),
            })
    }

    async fn write(&self, path: &Path, data: &[u8]) -> Result<(), ProviderError> {
        let resolved = self.resolve_path(path);
        // 确保父目录存在
        if let Some(parent) = resolved.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                ProviderError::Other(format!("Failed to create parent directory: {}", e))
            })?;
        }
        fs::write(&resolved, data).await?;
        Ok(())
    }

    async fn create_dir(&self, path: &Path) -> Result<(), ProviderError> {
        let resolved = self.resolve_path(path);
        fs::create_dir(&resolved).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::AlreadyExists => ProviderError::AlreadyExists {
                path: resolved.display().to_string(),
            },
            std::io::ErrorKind::PermissionDenied => ProviderError::PermissionDenied,
            _ => ProviderError::Other(format!("Failed to create directory: {}", e)),
        })
    }

    async fn create_dir_all(&self, path: &Path) -> Result<(), ProviderError> {
        let resolved = self.resolve_path(path);
        fs::create_dir_all(&resolved).await?;
        Ok(())
    }

    async fn remove(&self, path: &Path) -> Result<(), ProviderError> {
        let resolved = self.resolve_path(path);
        let metadata = fs::metadata(&resolved).await;

        if let Err(_e) = metadata {
            return Err(ProviderError::NotFound {
                path: resolved.display().to_string(),
            });
        }

        let metadata = metadata.unwrap();
        if metadata.is_dir() {
            fs::remove_dir(&resolved).await?;
        } else {
            fs::remove_file(&resolved).await?;
        }
        Ok(())
    }

    async fn remove_dir_all(&self, path: &Path) -> Result<(), ProviderError> {
        let resolved = self.resolve_path(path);
        fs::remove_dir_all(&resolved)
            .await
            .map_err(|_e| ProviderError::NotFound {
                path: resolved.display().to_string(),
            })
    }

    async fn metadata(&self, path: &Path) -> Result<FileMetadata, ProviderError> {
        let resolved = self.resolve_path(path);
        let metadata = fs::metadata(&resolved)
            .await
            .map_err(|_| ProviderError::NotFound {
                path: resolved.display().to_string(),
            })?;

        #[cfg(unix)]
        let permissions = Some(metadata.permissions().mode());
        #[cfg(not(unix))]
        let permissions = None;

        Ok(FileMetadata {
            size: metadata.len(),
            is_dir: metadata.is_dir(),
            is_file: metadata.is_file(),
            permissions,
            modified: metadata.modified().ok(),
            created: metadata.created().ok(),
            accessed: metadata.accessed().ok(),
        })
    }

    async fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, ProviderError> {
        let resolved = self.resolve_path(path);
        let mut entries = fs::read_dir(&resolved)
            .await
            .map_err(|_| ProviderError::NotFound {
                path: resolved.display().to_string(),
            })?;

        let mut result = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            result.push(entry.path());
        }
        Ok(result)
    }

    async fn exists(&self, path: &Path) -> Result<bool, ProviderError> {
        let resolved = self.resolve_path(path);
        Ok(fs::try_exists(&resolved).await?)
    }

    async fn copy(&self, from: &Path, to: &Path) -> Result<(), ProviderError> {
        let from_resolved = self.resolve_path(from);
        let to_resolved = self.resolve_path(to);

        // 确保目标目录存在
        if let Some(parent) = to_resolved.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                ProviderError::Other(format!("Failed to create parent directory: {}", e))
            })?;
        }

        fs::copy(&from_resolved, &to_resolved)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => ProviderError::NotFound {
                    path: from_resolved.display().to_string(),
                },
                _ => ProviderError::Other(format!("Failed to copy file: {}", e)),
            })?;
        Ok(())
    }

    async fn rename(&self, from: &Path, to: &Path) -> Result<(), ProviderError> {
        let from_resolved = self.resolve_path(from);
        let to_resolved = self.resolve_path(to);

        // 确保目标目录存在
        if let Some(parent) = to_resolved.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                ProviderError::Other(format!("Failed to create parent directory: {}", e))
            })?;
        }

        fs::rename(&from_resolved, &to_resolved)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => ProviderError::NotFound {
                    path: from_resolved.display().to_string(),
                },
                _ => ProviderError::Other(format!("Failed to rename file: {}", e)),
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_read_write() {
        let temp_dir = TempDir::new().unwrap();
        let provider = LocalFileProvider::with_work_dir(temp_dir.path().to_path_buf());

        let test_file = PathBuf::from("test.txt");
        let test_data = b"Hello, World!";

        // 写入
        provider.write(&test_file, test_data).await.unwrap();

        // 读取
        let read_data = provider.read(&test_file).await.unwrap();
        assert_eq!(read_data, test_data);
    }

    #[tokio::test]
    async fn test_create_and_list_dir() {
        let temp_dir = TempDir::new().unwrap();
        let provider = LocalFileProvider::with_work_dir(temp_dir.path().to_path_buf());

        let test_dir = PathBuf::from("test_dir");

        // 创建目录
        provider.create_dir(&test_dir).await.unwrap();

        // 检查存在
        assert!(provider.exists(&test_dir).await.unwrap());

        // 列出目录
        let entries = provider.list_dir(Path::new(".")).await.unwrap();
        assert!(entries.iter().any(|p| p.ends_with("test_dir")));
    }

    #[tokio::test]
    async fn test_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let provider = LocalFileProvider::with_work_dir(temp_dir.path().to_path_buf());

        let test_file = PathBuf::from("test.txt");
        let test_data = b"Hello, World!";

        provider.write(&test_file, test_data).await.unwrap();

        let metadata = provider.metadata(&test_file).await.unwrap();
        assert_eq!(metadata.size, 13);
        assert!(metadata.is_file);
        assert!(!metadata.is_dir);
    }

    #[tokio::test]
    async fn test_copy_rename() {
        let temp_dir = TempDir::new().unwrap();
        let provider = LocalFileProvider::with_work_dir(temp_dir.path().to_path_buf());

        let src_file = PathBuf::from("src.txt");
        let dst_file = PathBuf::from("dst.txt");
        let test_data = b"Test data";

        provider.write(&src_file, test_data).await.unwrap();

        // 复制
        provider.copy(&src_file, &dst_file).await.unwrap();
        assert!(provider.exists(&dst_file).await.unwrap());

        // 重命名
        let renamed_file = PathBuf::from("renamed.txt");
        provider.rename(&dst_file, &renamed_file).await.unwrap();
        assert!(provider.exists(&renamed_file).await.unwrap());
        assert!(!provider.exists(&dst_file).await.unwrap());
    }

    #[tokio::test]
    async fn test_remove() {
        let temp_dir = TempDir::new().unwrap();
        let provider = LocalFileProvider::with_work_dir(temp_dir.path().to_path_buf());

        let test_file = PathBuf::from("test.txt");
        provider.write(&test_file, b"test").await.unwrap();

        assert!(provider.exists(&test_file).await.unwrap());
        provider.remove(&test_file).await.unwrap();
        assert!(!provider.exists(&test_file).await.unwrap());
    }
}
