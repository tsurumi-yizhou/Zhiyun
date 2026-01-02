use serde::{Deserialize, Serialize};
use crate::common::endpoint::traits::{ChatMessage, Usage, ToolCall};

/// 聊天流增量内容
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// 聊天流事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ChatStreamEvent {
    Start,
    Delta(ChatDelta),
    Usage(Usage),
    Error(String),
    Done,
}

/// 完整聊天响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

/// 端点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub organization: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_event_serialization() {
        let event = ChatStreamEvent::Delta(ChatDelta {
            content: Some("hello".to_string()),
            ..Default::default()
        });
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"delta\""));
        assert!(json.contains("\"content\":\"hello\""));
    }
}

// 占位符
pub struct Endpoint;
