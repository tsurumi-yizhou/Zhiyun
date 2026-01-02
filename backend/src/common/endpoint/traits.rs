// Core data types for LLM endpoint
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Provider & Model from models.dev
// ============================================================================

/// Provider information from models.dev
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub api: String,
    pub env: Vec<String>,
    pub doc: String,
    pub models: HashMap<String, ModelInfo>,
}

/// Model information from models.dev
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub family: String,
    pub reasoning: bool,
    pub tool_call: bool,
    pub attachment: bool,
    pub vision: bool,
    pub cost: ModelCost,
    pub limit: ModelLimit,
}

/// Cost information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCost {
    pub input: f64,
    pub output: f64,
    pub cache_read: Option<f64>,
}

/// Model limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLimit {
    pub context: usize,
    pub output: usize,
}

// ============================================================================
// Chat Types
// ============================================================================

/// Chat message for LLM completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: MessageContent,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

/// Message role
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Message content supporting text, images, and files
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Image {
        url: String,
        detail: Option<ImageDetail>,
    },
    File {
        local_id: String,
    },
    MultiModal(Vec<ContentPart>),
}

/// Content part for multimodal messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text {
        text: String,
    },
    ImageUrl {
        url: String,
        detail: Option<ImageDetail>,
    },
    FileRef {
        local_id: String,
    },
}

/// Image detail level
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageDetail {
    Auto,
    Low,
    High,
}

/// Tool call in assistant message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub function: FunctionCall,
}

/// Function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Chat options
#[derive(Debug, Clone, Default)]
pub struct ChatOptions {
    pub max_tokens: Option<u32>,
    pub tools: Option<Vec<ToolDefinition>>,
}

/// Tool definition for function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub r#type: String,
    pub function: FunctionDefinition,
}

/// Function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

// ============================================================================
// Routing Model Types
// ============================================================================

/// Model routing result with priority-ordered model IDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRoutingResult {
    /// Primary model choice (highest priority)
    pub primary: String,
    /// Secondary fallback options (priority order)
    pub fallbacks: Vec<String>,
}

/// Task category for routing decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCategory {
    pub name: String,
    pub description: String,
    pub preferred_models: Vec<String>,
}

// ============================================================================
// Embedding Types
// ============================================================================

/// Embedding response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub data: Vec<Embedding>,
    pub usage: EmbeddingUsage,
}

/// Single embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub index: usize,
    pub embedding: Vec<f32>,
}

/// Embedding usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingUsage {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}

// ============================================================================
// Token Usage & Cost Tracking
// ============================================================================

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub cache_read_tokens: Option<u32>,
}

impl Usage {
    /// Calculate cost based on model pricing
    pub fn cost(&self, model_info: &ModelInfo) -> CostBreakdown {
        let input_cost = (self.prompt_tokens as f64 / 1_000_000.0) * model_info.cost.input;
        let output_cost = (self.completion_tokens as f64 / 1_000_000.0) * model_info.cost.output;
        let cache_cost = self.cache_read_tokens.map_or(0.0, |tokens| {
            (tokens as f64 / 1_000_000.0) * model_info.cost.cache_read.unwrap_or(0.0)
        });

        CostBreakdown {
            input: input_cost,
            output: output_cost,
            cache_read: cache_cost,
            total: input_cost + output_cost + cache_cost,
        }
    }

    /// Calculate context usage percentage
    pub fn context_usage_percent(&self, model_info: &ModelInfo) -> f64 {
        (self.total_tokens as f64 / model_info.limit.context as f64) * 100.0
    }
}

/// Cost breakdown
#[derive(Debug, Clone)]
pub struct CostBreakdown {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub total: f64,
}

// ============================================================================
// File Management Types
// ============================================================================

/// File upload request
#[derive(Debug, Clone)]
pub struct FileUploadRequest {
    pub file: Vec<u8>,
    pub filename: String,
    pub purpose: FilePurpose,
}

/// File purpose
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilePurpose {
    Assistants,
    AssistantsOutput,
    FineTune,
    Batch,
    Vision,
}

/// File object from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileObject {
    pub id: String,
    pub bytes: u64,
    pub created_at: u64,
    pub filename: String,
    pub purpose: String,
}

/// File deletion status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDeletionStatus {
    pub id: String,
    pub deleted: bool,
}

/// File content response
#[derive(Debug, Clone)]
pub struct FileContentResponse {
    pub content: String,
}

/// File state across providers
#[derive(Debug, Clone)]
pub struct FileState {
    pub filename: String,
    pub bytes: u64,
    pub purpose: FilePurpose,
    pub provider_files: HashMap<String, ProviderFileState>,
}

/// Provider file state
#[derive(Debug, Clone)]
pub struct ProviderFileState {
    pub file_id: String,
    pub uploaded_at: u64,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_info_serialization() {
        let info = ModelInfo {
            id: "gpt-4".to_string(),
            name: "GPT-4".to_string(),
            family: "gpt-4".to_string(),
            reasoning: false,
            tool_call: true,
            attachment: true,
            vision: true,
            cost: ModelCost {
                input: 0.03,
                output: 0.06,
                cache_read: Some(0.01),
            },
            limit: ModelLimit {
                context: 8192,
                output: 4096,
            },
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"gpt-4\""));
        assert!(json.contains("\"tool_call\":true"));
        assert!(json.contains("\"attachment\":true"));
    }

    #[test]
    fn test_chat_message_text() {
        let msg = ChatMessage {
            role: MessageRole::User,
            content: MessageContent::Text("Hello".to_string()),
            tool_calls: None,
            tool_call_id: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Hello"));
        assert!(json.contains("\"role\":\"user\""));
    }

    #[test]
    fn test_chat_message_file() {
        let msg = ChatMessage {
            role: MessageRole::User,
            content: MessageContent::File {
                local_id: "file-123".to_string(),
            },
            tool_calls: None,
            tool_call_id: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("file-123"));
    }

    #[test]
    fn test_usage_cost() {
        let usage = Usage {
            prompt_tokens: 1000,
            completion_tokens: 500,
            total_tokens: 1500,
            cache_read_tokens: Some(200),
        };

        let info = ModelInfo {
            id: "test".to_string(),
            name: "Test".to_string(),
            family: "test".to_string(),
            reasoning: false,
            tool_call: false,
            attachment: false,
            vision: false,
            cost: ModelCost {
                input: 10.0,
                output: 20.0,
                cache_read: Some(1.0),
            },
            limit: ModelLimit {
                context: 100000,
                output: 10000,
            },
        };

        let cost = usage.cost(&info);
        assert!((cost.input - 0.01).abs() < 0.0001); // 1000/1M * 10
        assert!((cost.output - 0.01).abs() < 0.0001); // 500/1M * 20
        assert!((cost.cache_read - 0.0002).abs() < 0.0001); // 200/1M * 1
        assert!((cost.total - 0.0202).abs() < 0.0001);
    }

    #[test]
    fn test_context_usage_percent() {
        let usage = Usage {
            prompt_tokens: 50000,
            completion_tokens: 0,
            total_tokens: 50000,
            cache_read_tokens: None,
        };

        let info = ModelInfo {
            id: "test".to_string(),
            name: "Test".to_string(),
            family: "test".to_string(),
            reasoning: false,
            tool_call: false,
            attachment: false,
            vision: false,
            cost: ModelCost {
                input: 1.0,
                output: 1.0,
                cache_read: None,
            },
            limit: ModelLimit {
                context: 100000,
                output: 10000,
            },
        };

        let percent = usage.context_usage_percent(&info);
        assert!((percent - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_file_state() {
        let mut state = FileState {
            filename: "test.txt".to_string(),
            bytes: 1024,
            purpose: FilePurpose::Assistants,
            provider_files: HashMap::new(),
        };

        state.provider_files.insert(
            "openai".to_string(),
            ProviderFileState {
                file_id: "file-abc".to_string(),
                uploaded_at: 123456,
            },
        );

        assert_eq!(state.filename, "test.txt");
        assert_eq!(state.provider_files.len(), 1);
        assert_eq!(
            state.provider_files.get("openai").unwrap().file_id,
            "file-abc"
        );
    }

    #[test]
    fn test_chat_options_default() {
        let opts = ChatOptions::default();
        assert!(opts.max_tokens.is_none());
        assert!(opts.tools.is_none());
    }

    #[test]
    fn test_image_detail() {
        let detail = ImageDetail::High;
        let json = serde_json::to_string(&detail).unwrap();
        assert_eq!(json, "\"high\"");
    }
}

