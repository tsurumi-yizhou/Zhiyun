use crate::common::endpoint::ChatMessage;

/// 负责对话上下文的智能压缩与窗口管理
pub struct ContextManager {
    messages: Vec<ChatMessage>,
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    /// 添加消息到上下文
    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
    }

    /// 压缩上下文（Mock 逻辑：保留最后 N 条）
    pub fn compress(&mut self, limit: usize) {
        if self.messages.len() > limit {
            let start = self.messages.len() - limit;
            self.messages = self.messages.drain(start..).collect();
        }
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::endpoint::{MessageContent, MessageRole};

    #[test]
    fn test_context_manager() {
        let mut manager = ContextManager::new();
        let msg = ChatMessage {
            role: MessageRole::User,
            content: MessageContent::Text("hello".to_string()),
            tool_calls: None,
        };

        manager.add_message(msg.clone());
        manager.add_message(msg.clone());
        assert_eq!(manager.message_count(), 2);

        manager.compress(1);
        assert_eq!(manager.message_count(), 1);
    }
}
