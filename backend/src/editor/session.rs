use crate::common::change::Change;
use crate::common::change::operation::Operation;
use crate::common::change::thread::{ThreadId, ThreadManager};
use crate::common::change::version::VectorClock;
use crate::common::intent::{EditorIntent, IntentHandler, SystemIntent};
use crate::common::provider::traits::StorageProvider;
use crate::editor::reconciler::Reconciler;
use crate::editor::tab::TabControl;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 编辑器会话的内部状态
pub struct EditorSessionState {
    pub id: Uuid,
    pub project_path: String,
    pub active_thread: ThreadId,
    pub storage: Arc<dyn StorageProvider>,
    pub thread_manager: Arc<ThreadManager>,
    pub reconciler: Reconciler,
    pub tabs: TabControl,
    pub active_tab: Option<Uuid>,
    /// 暂存的变更（尚未提交到 Thread）
    pub pending_operations: Vec<Operation>,
    /// 当前 Thread 的最新 Change ID
    pub head_change_id: Option<Uuid>,
}

/// 单个编辑器会话（通过 Arc<RwLock> 实现线程安全）
pub struct EditorSession {
    pub id: Uuid,
    pub state: Arc<RwLock<EditorSessionState>>,
}

impl EditorSession {
    pub fn new(
        project_path: String,
        thread_id: ThreadId,
        storage: Arc<dyn StorageProvider>,
        thread_manager: Arc<ThreadManager>,
    ) -> Self {
        let reconciler = Reconciler::new(storage.clone());
        let head_change_id = thread_manager
            .get_thread(thread_id)
            .and_then(|t| t.head_change_id);

        let id = Uuid::new_v4();
        let state = EditorSessionState {
            id,
            project_path,
            active_thread: thread_id,
            storage,
            thread_manager,
            reconciler,
            tabs: TabControl::new(),
            active_tab: None,
            pending_operations: Vec::new(),
            head_change_id,
        };

        Self {
            id,
            state: Arc::new(RwLock::new(state)),
        }
    }
}

#[async_trait]
impl IntentHandler for EditorSession {
    async fn handle(&self, intent: SystemIntent) -> Result<()> {
        match intent {
            SystemIntent::Editor(editor_intent) => {
                let mut state = self.state.write().await;
                match editor_intent {
                    EditorIntent::OpenFile { path } => {
                        let _content = state.storage.read_file(&path).await?;
                        let thread_id = state.active_thread;
                        let tab_id = state.tabs.open_tab(thread_id, &path);
                        state.active_tab = Some(tab_id);
                        Ok(())
                    }
                    EditorIntent::SwitchTab { tab_id } => {
                        if state.tabs.get_tab(&tab_id).is_some() {
                            state.active_tab = Some(tab_id);
                        }
                        Ok(())
                    }
                    EditorIntent::WriteFile { path, content } => {
                        let op = Operation::file_write(path, content);
                        state.pending_operations.push(op);
                        Ok(())
                    }
                    EditorIntent::DeleteFile { path } => {
                        let op = Operation::file_delete(path);
                        state.pending_operations.push(op);
                        Ok(())
                    }
                    EditorIntent::Save => {
                        if !state.pending_operations.is_empty() {
                            let operations = std::mem::take(&mut state.pending_operations);
                            let parents =
                                state.head_change_id.map(|id| vec![id]).unwrap_or_default();

                            let change = Change::new(
                                Uuid::new_v4(), // 模拟用户 ID
                                operations,
                                VectorClock::new(),
                                parents,
                            );

                            // 1. 应用到物理文件系统 (Provider)
                            state.reconciler.apply_to_storage(&change).await?;

                            // 2. 提交到 ThreadManager
                            state
                                .thread_manager
                                .commit_change(state.active_thread, change.clone())?;

                            // 3. 更新本地 Head
                            state.head_change_id = Some(change.id);
                        }
                        Ok(())
                    }
                }
            }
            _ => Err(anyhow::anyhow!(
                "EditorSession cannot handle non-editor intents"
            )),
        }
    }
}

/// 管理编辑器会话与活动项目
pub struct SessionManager {
    thread_manager: Arc<ThreadManager>,
    sessions: HashMap<Uuid, Arc<EditorSession>>,
}

impl SessionManager {
    pub fn new(thread_manager: Arc<ThreadManager>) -> Self {
        Self {
            thread_manager,
            sessions: HashMap::new(),
        }
    }

    /// 创建会话
    pub async fn create_session(
        &mut self,
        project_path: String,
        thread_id: ThreadId,
        storage: Arc<dyn StorageProvider>,
    ) -> Uuid {
        let session = EditorSession::new(
            project_path,
            thread_id,
            storage,
            self.thread_manager.clone(),
        );
        let id = session.id;
        let session_arc = Arc::new(session);
        self.sessions.insert(id, session_arc);
        id
    }

    /// 获取会话
    pub fn get_session(&self, id: &Uuid) -> Option<Arc<EditorSession>> {
        self.sessions.get(id).cloned()
    }

    /// 关闭会话
    pub fn close_session(&mut self, id: &Uuid) {
        self.sessions.remove(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::intent::{IntentCategory, IntentDispatcher};
    use crate::common::provider::traits::FileMetadata;

    struct MockStorage;
    #[async_trait]
    impl StorageProvider for MockStorage {
        fn id(&self) -> &str {
            "mock"
        }
        async fn read_file(&self, _path: &str) -> Result<Vec<u8>> {
            Ok(b"hello".to_vec())
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
    async fn test_editor_session_flow_via_dispatcher() {
        let storage = Arc::new(MockStorage);
        let thread_manager = Arc::new(ThreadManager::new());
        let mut manager = SessionManager::new(thread_manager.clone());
        let dispatcher = IntentDispatcher::new();

        let thread_id = thread_manager
            .create_branch(
                thread_manager
                    .get_thread(thread_manager.get_thread_id_by_name("main").unwrap())
                    .unwrap()
                    .id,
                "test",
            )
            .unwrap();

        let session_id = manager
            .create_session("/project".to_string(), thread_id, storage)
            .await;
        let session = manager.get_session(&session_id).unwrap();

        // 注册处理器到分发器
        dispatcher
            .register(IntentCategory::Editor, session.clone())
            .await;

        // 1. 发送 Intent: 打开文件
        dispatcher
            .dispatch(SystemIntent::Editor(EditorIntent::OpenFile {
                path: "test.txt".to_string(),
            }))
            .await
            .unwrap();

        {
            let state = session.state.read().await;
            assert!(state.active_tab.is_some());
        }

        // 2. 发送 Intent: 写入文件
        dispatcher
            .dispatch(SystemIntent::Editor(EditorIntent::WriteFile {
                path: "test.txt".to_string(),
                content: b"world".to_vec(),
            }))
            .await
            .unwrap();

        {
            let state = session.state.read().await;
            assert_eq!(state.pending_operations.len(), 1);
        }

        // 3. 发送 Intent: 保存
        dispatcher
            .dispatch(SystemIntent::Editor(EditorIntent::Save))
            .await
            .unwrap();

        {
            let state = session.state.read().await;
            assert!(state.pending_operations.is_empty());
            assert!(state.head_change_id.is_some());
        }
    }
}
