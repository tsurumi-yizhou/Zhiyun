use std::collections::HashMap;
use uuid::Uuid;

/// 实现 Tab 的生命周期管理与元调用
pub struct TabControl {
    tabs: HashMap<Uuid, TabState>,
}

pub struct TabState {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub file_path: String,
}

impl Default for TabControl {
    fn default() -> Self {
        Self::new()
    }
}

impl TabControl {
    pub fn new() -> Self {
        Self {
            tabs: HashMap::new(),
        }
    }

    /// 打开新 Tab
    pub fn open_tab(&mut self, thread_id: Uuid, file_path: &str) -> Uuid {
        let id = Uuid::new_v4();
        self.tabs.insert(
            id,
            TabState {
                id,
                thread_id,
                file_path: file_path.to_string(),
            },
        );
        id
    }

    /// 获取 Tab 状态
    pub fn get_tab(&self, id: &Uuid) -> Option<&TabState> {
        self.tabs.get(id)
    }

    /// 关闭 Tab
    pub fn close_tab(&mut self, id: &Uuid) {
        self.tabs.remove(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_control() {
        let mut control = TabControl::new();
        let thread_id = Uuid::new_v4();
        let id = control.open_tab(thread_id, "src/lib.rs");
        let tab = control.get_tab(&id).unwrap();
        assert_eq!(tab.thread_id, thread_id);
        assert_eq!(tab.file_path, "src/lib.rs");
    }
}
