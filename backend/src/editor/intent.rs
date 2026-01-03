use uuid::Uuid;

/// 编辑器特定的详细意图。
#[derive(Debug, Clone)]
pub enum EditorIntent {
    /// 打开指定路径的文件。
    OpenFile { path: String },

    /// 切换到指定的 Tab。
    SwitchTab { tab_id: Uuid },

    /// 写入内容到指定文件。
    WriteFile { path: String, content: Vec<u8> },

    /// 删除指定路径的文件。
    DeleteFile { path: String },

    /// 保存当前编辑器状态。
    Save,
}
