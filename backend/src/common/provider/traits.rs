use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 文件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified_at: u64, // Unix timestamp
    pub created_at: u64,  // Unix timestamp
}

/// 存储提供者接口
#[async_trait]
pub trait StorageProvider: Send + Sync {
    /// 获取提供者标识符
    fn id(&self) -> &str;

    /// 读取文件内容
    async fn read_file(&self, path: &str) -> anyhow::Result<Vec<u8>>;

    /// 写入文件内容
    async fn write_file(&self, path: &str, content: &[u8]) -> anyhow::Result<()>;

    /// 删除文件或目录
    async fn delete(&self, path: &str, recursive: bool) -> anyhow::Result<()>;

    /// 列出目录内容
    async fn list_dir(&self, path: &str) -> anyhow::Result<Vec<FileMetadata>>;

    /// 获取元数据
    async fn get_metadata(&self, path: &str) -> anyhow::Result<FileMetadata>;

    /// 检查路径是否存在
    async fn exists(&self, path: &str) -> anyhow::Result<bool>;

    /// 创建目录
    async fn create_dir(&self, path: &str, recursive: bool) -> anyhow::Result<()>;
}

/// 执行选项
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecuteOptions {
    pub cwd: Option<String>,
    pub env: std::collections::HashMap<String, String>,
    pub timeout_ms: Option<u64>,
}

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// 执行提供者接口
#[async_trait]
pub trait ExecutionProvider: Send + Sync {
    /// 执行命令
    async fn execute(&self, command: &str, options: ExecuteOptions) -> anyhow::Result<ExecuteResult>;

    /// 终止当前运行的任务（如果支持）
    async fn kill(&self, task_id: &str) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockStorage;
    #[async_trait]
    impl StorageProvider for MockStorage {
        fn id(&self) -> &str { "mock" }
        async fn read_file(&self, _path: &str) -> anyhow::Result<Vec<u8>> { Ok(vec![]) }
        async fn write_file(&self, _path: &str, _content: &[u8]) -> anyhow::Result<()> { Ok(()) }
        async fn delete(&self, _path: &str, _recursive: bool) -> anyhow::Result<()> { Ok(()) }
        async fn list_dir(&self, _path: &str) -> anyhow::Result<Vec<FileMetadata>> { Ok(vec![]) }
        async fn get_metadata(&self, path: &str) -> anyhow::Result<FileMetadata> {
            Ok(FileMetadata {
                path: path.to_string(),
                size: 0,
                is_dir: false,
                modified_at: 0,
                created_at: 0,
            })
        }
        async fn exists(&self, _path: &str) -> anyhow::Result<bool> { Ok(true) }
        async fn create_dir(&self, _path: &str, _recursive: bool) -> anyhow::Result<()> { Ok(()) }
    }

    #[tokio::test]
    async fn test_storage_trait_mock() {
        let storage = MockStorage;
        assert_eq!(storage.id(), "mock");
        let meta = storage.get_metadata("test.txt").await.unwrap();
        assert_eq!(meta.path, "test.txt");
    }
}
