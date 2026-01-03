use crate::common::change::Change;
use crate::common::change::operation::Operation;
use crate::common::provider::traits::StorageProvider;
use anyhow::Result;
use std::sync::Arc;

/// 协调本地 UI 状态与 CRDT Thread 状态的一致性，并将变更应用到存储提供者
pub struct Reconciler {
    storage: Arc<dyn StorageProvider>,
}

impl Reconciler {
    pub fn new(storage: Arc<dyn StorageProvider>) -> Self {
        Self { storage }
    }

    /// 将 Change 应用到底层存储提供者
    pub async fn apply_to_storage(&self, change: &Change) -> Result<()> {
        for op in &change.operations {
            match op {
                Operation::FileWrite { path, content } => {
                    self.storage.write_file(path, content).await?;
                }
                Operation::FileDelete { path } => {
                    self.storage.delete(path, false).await?;
                }
                // 目前 MVP 仅处理文件级操作
                _ => {}
            }
        }
        Ok(())
    }

    /// 应用变更到本地 UI 状态（Mock）
    pub fn apply_to_ui(&self, _changes: Vec<Change>) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::change::version::VectorClock;
    use crate::common::provider::traits::FileMetadata;
    use async_trait::async_trait;
    use std::sync::Mutex;
    use uuid::Uuid;

    struct SpyStorage {
        written_files: Mutex<Vec<(String, Vec<u8>)>>,
    }

    #[async_trait]
    impl StorageProvider for SpyStorage {
        fn id(&self) -> &str {
            "spy"
        }
        async fn read_file(&self, _path: &str) -> Result<Vec<u8>> {
            Ok(vec![])
        }
        async fn write_file(&self, path: &str, content: &[u8]) -> Result<()> {
            self.written_files
                .lock()
                .unwrap()
                .push((path.to_string(), content.to_vec()));
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
                path: "".to_string(),
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
    async fn test_reconciler_apply_to_storage() {
        let storage = Arc::new(SpyStorage {
            written_files: Mutex::new(Vec::new()),
        });
        let reconciler = Reconciler::new(storage.clone());

        let op = Operation::file_write("test.rs".to_string(), b"fn main() {}".to_vec());
        let change = Change::new(Uuid::new_v4(), vec![op], VectorClock::new(), Vec::new());

        reconciler.apply_to_storage(&change).await.unwrap();

        let written = storage.written_files.lock().unwrap();
        assert_eq!(written.len(), 1);
        assert_eq!(written[0].0, "test.rs");
        assert_eq!(written[0].1, b"fn main() {}");
    }
}
