use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 消息角色
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// 聊天消息内容部分
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text {
        text: String,
    },
    ImageUrl {
        url: String,
        detail: Option<ImageDetail>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImageDetail {
    Auto,
    Low,
    High,
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: MessageContent,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

/// 工具调用
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// 聊天选项
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatOptions {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
    pub stop: Option<Vec<String>>,
    pub presence_penalty: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub user: Option<String>,
}

/// 模型使用统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// 模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub context_window: u32,
    pub supports_vision: bool,
    pub supports_tools: bool,
}

/// 提供者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub base_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_serialization() {
        let msg = ChatMessage {
            role: MessageRole::User,
            content: MessageContent::Text("hello".to_string()),
            tool_calls: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"hello\""));
    }
}

// 剩余占位符，保持接口完整性
pub type CostBreakdown = HashMap<String, f64>;
pub type Embedding = Vec<f32>;
pub struct EmbeddingResponse {
    pub data: Vec<Embedding>,
    pub usage: Usage,
}
pub type EmbeddingUsage = Usage;
pub type FileContentResponse = Vec<u8>;
pub struct FileDeletionStatus {
    pub id: String,
    pub deleted: bool,
}
pub struct FileObject {
    pub id: String,
    pub bytes: u32,
    pub filename: String,
    pub purpose: String,
}
pub type FilePurpose = String;
pub type FileState = String;
pub struct FileUploadRequest {
    pub filename: String,
    pub purpose: String,
    pub content: Vec<u8>,
}
pub struct FunctionDefinition {
    pub name: String,
    pub description: Option<String>,
    pub parameters: serde_json::Value,
}
pub type ModelCost = f64;
pub type ModelLimit = u32;
pub struct ModelRoutingResult {
    pub model_id: String,
    pub priority: u32,
}
pub type ProviderFileState = String;
pub type TaskCategory = String;
pub struct ToolDefinition {
    pub r#type: String,
    pub function: FunctionDefinition,
}
