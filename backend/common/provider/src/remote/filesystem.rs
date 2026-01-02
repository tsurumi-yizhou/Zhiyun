use crate::traits::{FileMetadata, FileProvider, ProviderError, SshConfig};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 远程文件系统提供者
///
/// 基于 SFTP 协议实现远程文件系统操作
///
/// 注意：这是一个框架实现，实际使用需要正确的 SSH 客户端配置
#[derive(Debug)]
pub struct RemoteFileProvider {
    /// SSH 配置
    ssh_config: SshConfig,
    /// SFTP 会话占位符（实际使用需要 SSH 客户端）
    _sftp_session: Arc<tokio::sync::Mutex<Option<bool>>>,
}

impl Clone for RemoteFileProvider {
    fn clone(&self) -> Self {
        Self {
            ssh_config: self.ssh_config.clone(),
            _sftp_session: Arc::clone(&self._sftp_session),
        }
    }
}

impl RemoteFileProvider {
    /// 创建新的远程文件提供者
    pub fn new(ssh_config: SshConfig) -> Self {
        Self {
            ssh_config,
            _sftp_session: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    /// 解析路径为字符串
    fn resolve_path_str(&self, path: &Path) -> String {
        if path.is_absolute() {
            path.to_string_lossy().to_string()
        } else {
            self.ssh_config
                .work_dir
                .join(path)
                .to_string_lossy()
                .to_string()
        }
    }

    /// 解析路径
    #[allow(dead_code)]
    fn resolve_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.ssh_config.work_dir.join(path)
        }
    }
}

#[async_trait]
impl FileProvider for RemoteFileProvider {
    async fn read(&self, path: &Path) -> Result<Vec<u8>, ProviderError> {
        Err(ProviderError::Other(format!(
            "Remote read requires SSH client setup. Path: {}",
            self.resolve_path_str(path)
        )))
    }

    async fn write(&self, path: &Path, _data: &[u8]) -> Result<(), ProviderError> {
        Err(ProviderError::Other(format!(
            "Remote write requires SSH client setup. Path: {}",
            self.resolve_path_str(path)
        )))
    }

    async fn create_dir(&self, path: &Path) -> Result<(), ProviderError> {
        Err(ProviderError::Other(format!(
            "Remote create_dir requires SSH client setup. Path: {}",
            self.resolve_path_str(path)
        )))
    }

    async fn create_dir_all(&self, path: &Path) -> Result<(), ProviderError> {
        Err(ProviderError::Other(format!(
            "Remote create_dir_all requires SSH client setup. Path: {}",
            self.resolve_path_str(path)
        )))
    }

    async fn remove(&self, path: &Path) -> Result<(), ProviderError> {
        Err(ProviderError::Other(format!(
            "Remote remove requires SSH client setup. Path: {}",
            self.resolve_path_str(path)
        )))
    }

    async fn remove_dir_all(&self, path: &Path) -> Result<(), ProviderError> {
        Err(ProviderError::Other(format!(
            "Remote remove_dir_all requires SSH client setup. Path: {}",
            self.resolve_path_str(path)
        )))
    }

    async fn metadata(&self, path: &Path) -> Result<FileMetadata, ProviderError> {
        Err(ProviderError::Other(format!(
            "Remote metadata requires SSH client setup. Path: {}",
            self.resolve_path_str(path)
        )))
    }

    async fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, ProviderError> {
        Err(ProviderError::Other(format!(
            "Remote list_dir requires SSH client setup. Path: {}",
            self.resolve_path_str(path)
        )))
    }

    async fn exists(&self, path: &Path) -> Result<bool, ProviderError> {
        Err(ProviderError::Other(format!(
            "Remote exists requires SSH client setup. Path: {}",
            self.resolve_path_str(path)
        )))
    }

    async fn copy(&self, from: &Path, to: &Path) -> Result<(), ProviderError> {
        Err(ProviderError::Other(format!(
            "Remote copy requires SSH client setup. From: {}, To: {}",
            self.resolve_path_str(from),
            self.resolve_path_str(to)
        )))
    }

    async fn rename(&self, from: &Path, to: &Path) -> Result<(), ProviderError> {
        Err(ProviderError::Other(format!(
            "Remote rename requires SSH client setup. From: {}, To: {}",
            self.resolve_path_str(from),
            self.resolve_path_str(to)
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_remote_file_provider() {
        let ssh_config = SshConfig::with_password("localhost", "testuser", "testpass", "/tmp");
        let provider = RemoteFileProvider::new(ssh_config);
        assert_eq!(provider.ssh_config.hostname, "localhost");
    }

    #[test]
    fn test_resolve_path_str_absolute() {
        let ssh_config = SshConfig::with_password("localhost", "testuser", "testpass", "/tmp");
        let provider = RemoteFileProvider::new(ssh_config);

        let path = Path::new("/absolute/path");
        let resolved = provider.resolve_path_str(path);
        assert_eq!(resolved, "/absolute/path");
    }

    #[test]
    fn test_resolve_path_str_relative() {
        let ssh_config = SshConfig::with_password("localhost", "testuser", "testpass", "/tmp");
        let provider = RemoteFileProvider::new(ssh_config);

        let path = Path::new("relative/path");
        let resolved = provider.resolve_path_str(path);
        assert_eq!(resolved, "/tmp/relative/path");
    }

    #[test]
    fn test_resolve_path_absolute() {
        let ssh_config = SshConfig::with_password("localhost", "testuser", "testpass", "/tmp");
        let provider = RemoteFileProvider::new(ssh_config);

        let path = Path::new("/absolute/path");
        let resolved = provider.resolve_path(path);
        assert_eq!(resolved, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn test_resolve_path_relative() {
        let ssh_config = SshConfig::with_password("localhost", "testuser", "testpass", "/tmp");
        let provider = RemoteFileProvider::new(ssh_config);

        let path = Path::new("relative/path");
        let resolved = provider.resolve_path(path);
        assert_eq!(resolved, PathBuf::from("/tmp/relative/path"));
    }

    #[test]
    fn test_clone() {
        let ssh_config = SshConfig::with_password("localhost", "testuser", "testpass", "/tmp");
        let provider = RemoteFileProvider::new(ssh_config);
        let cloned = provider.clone();

        assert_eq!(provider.ssh_config.hostname, cloned.ssh_config.hostname);
    }

    #[tokio::test]
    async fn test_read_returns_error() {
        let ssh_config = SshConfig::with_password("localhost", "testuser", "testpass", "/tmp");
        let provider = RemoteFileProvider::new(ssh_config);

        let result = provider.read(Path::new("test.txt")).await;
        assert!(result.is_err());
        if let Err(ProviderError::Other(msg)) = result {
            assert!(msg.contains("SSH client setup"));
        } else {
            panic!("Expected ProviderError::Other");
        }
    }

    #[tokio::test]
    async fn test_write_returns_error() {
        let ssh_config = SshConfig::with_password("localhost", "testuser", "testpass", "/tmp");
        let provider = RemoteFileProvider::new(ssh_config);

        let result = provider.write(Path::new("test.txt"), b"data").await;
        assert!(result.is_err());
        if let Err(ProviderError::Other(msg)) = result {
            assert!(msg.contains("SSH client setup"));
        } else {
            panic!("Expected ProviderError::Other");
        }
    }

    #[tokio::test]
    async fn test_create_dir_returns_error() {
        let ssh_config = SshConfig::with_password("localhost", "testuser", "testpass", "/tmp");
        let provider = RemoteFileProvider::new(ssh_config);

        let result = provider.create_dir(Path::new("test_dir")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_metadata_returns_error() {
        let ssh_config = SshConfig::with_password("localhost", "testuser", "testpass", "/tmp");
        let provider = RemoteFileProvider::new(ssh_config);

        let result = provider.metadata(Path::new("test.txt")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_dir_returns_error() {
        let ssh_config = SshConfig::with_password("localhost", "testuser", "testpass", "/tmp");
        let provider = RemoteFileProvider::new(ssh_config);

        let result = provider.list_dir(Path::new(".")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_exists_returns_error() {
        let ssh_config = SshConfig::with_password("localhost", "testuser", "testpass", "/tmp");
        let provider = RemoteFileProvider::new(ssh_config);

        let result = provider.exists(Path::new("test.txt")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_copy_returns_error() {
        let ssh_config = SshConfig::with_password("localhost", "testuser", "testpass", "/tmp");
        let provider = RemoteFileProvider::new(ssh_config);

        let result = provider
            .copy(Path::new("from.txt"), Path::new("to.txt"))
            .await;
        assert!(result.is_err());
        if let Err(ProviderError::Other(msg)) = result {
            assert!(msg.contains("From:"));
            assert!(msg.contains("To:"));
        } else {
            panic!("Expected ProviderError::Other with from/to info");
        }
    }

    #[tokio::test]
    async fn test_rename_returns_error() {
        let ssh_config = SshConfig::with_password("localhost", "testuser", "testpass", "/tmp");
        let provider = RemoteFileProvider::new(ssh_config);

        let result = provider
            .rename(Path::new("old.txt"), Path::new("new.txt"))
            .await;
        assert!(result.is_err());
        if let Err(ProviderError::Other(msg)) = result {
            assert!(msg.contains("From:"));
            assert!(msg.contains("To:"));
        } else {
            panic!("Expected ProviderError::Other with from/to info");
        }
    }
}
