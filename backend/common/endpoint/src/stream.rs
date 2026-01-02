// Streaming and response types for LLM endpoint
use serde::{Deserialize, Serialize};
use crate::traits::*;

// ============================================================================
// Chat Response Types
// ============================================================================

/// Chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

/// Choice in response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: usize,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

/// Streaming chunk event
#[derive(Debug, Clone)]
pub enum ChatStreamEvent {
    Delta {
        index: usize,
        delta: ChatDelta,
    },
    Choice {
        index: usize,
        message: ChatMessage,
        finish_reason: String,
    },
    Usage(Usage),
    Done,
}

/// Delta update for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

// ============================================================================
// Provider Configuration
// ============================================================================

/// Provider configuration
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub provider_id: String,
    pub api_key: String,
    pub base_url: Option<String>,
}

// ============================================================================
// Endpoint - Stateless model invocation point
// ============================================================================

/// Endpoint - a stateless model invocation point
#[derive(Debug, Clone)]
pub struct Endpoint {
    pub model_id: String,
    pub provider_id: String,
}

impl Endpoint {
    /// Create a new endpoint
    pub fn new(provider_id: &str, model_id: &str) -> Self {
        Self {
            provider_id: provider_id.to_string(),
            model_id: model_id.to_string(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_response_serialization() {
        let response = ChatResponse {
            id: "chat-123".to_string(),
            model: "gpt-4".to_string(),
            choices: vec![Choice {
                index: 0,
                message: ChatMessage {
                    role: MessageRole::Assistant,
                    content: MessageContent::Text("Hello!".to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Some(Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
                cache_read_tokens: None,
            }),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("chat-123"));
        assert!(json.contains("Hello!"));
        assert!(json.contains("stop"));
    }

    #[test]
    fn test_choice_serialization() {
        let choice = Choice {
            index: 0,
            message: ChatMessage {
                role: MessageRole::Assistant,
                content: MessageContent::Text("Test".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
            finish_reason: Some("stop".to_string()),
        };

        let json = serde_json::to_string(&choice).unwrap();
        assert!(json.contains("\"index\":0"));
        assert!(json.contains("Test"));
    }

    #[test]
    fn test_chat_delta() {
        let delta = ChatDelta {
            role: Some("assistant".to_string()),
            content: Some("Hello".to_string()),
            tool_calls: None,
        };

        let json = serde_json::to_string(&delta).unwrap();
        assert!(json.contains("Hello"));
        assert!(json.contains("assistant"));
    }

    #[test]
    fn test_chat_delta_with_tool_calls() {
        let tool_call = ToolCall {
            id: "call-1".to_string(),
            function: FunctionCall {
                name: "test_func".to_string(),
                arguments: "{}".to_string(),
            },
        };

        let delta = ChatDelta {
            role: None,
            content: None,
            tool_calls: Some(vec![tool_call]),
        };

        let json = serde_json::to_string(&delta).unwrap();
        assert!(json.contains("call-1"));
        assert!(json.contains("test_func"));
    }

    #[test]
    fn test_chat_stream_event_delta() {
        let event = ChatStreamEvent::Delta {
            index: 0,
            delta: ChatDelta {
                role: None,
                content: Some("Test".to_string()),
                tool_calls: None,
            },
        };

        match event {
            ChatStreamEvent::Delta { index, delta } => {
                assert_eq!(index, 0);
                assert_eq!(delta.content, Some("Test".to_string()));
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_chat_stream_event_choice() {
        let event = ChatStreamEvent::Choice {
            index: 0,
            message: ChatMessage {
                role: MessageRole::Assistant,
                content: MessageContent::Text("Done".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
            finish_reason: "stop".to_string(),
        };

        match event {
            ChatStreamEvent::Choice { index, message: _, finish_reason } => {
                assert_eq!(index, 0);
                assert_eq!(finish_reason, "stop");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_chat_stream_event_usage() {
        let usage = Usage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
            cache_read_tokens: None,
        };

        let event = ChatStreamEvent::Usage(usage);

        match event {
            ChatStreamEvent::Usage(u) => {
                assert_eq!(u.total_tokens, 150);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_chat_stream_event_done() {
        let event = ChatStreamEvent::Done;

        match event {
            ChatStreamEvent::Done => {}
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_provider_config() {
        let config = ProviderConfig {
            provider_id: "openai".to_string(),
            api_key: "sk-test".to_string(),
            base_url: Some("https://api.openai.com/v1".to_string()),
        };

        assert_eq!(config.provider_id, "openai");
        assert_eq!(config.api_key, "sk-test");
        assert_eq!(config.base_url, Some("https://api.openai.com/v1".to_string()));
    }

    #[test]
    fn test_provider_config_no_base_url() {
        let config = ProviderConfig {
            provider_id: "openai".to_string(),
            api_key: "sk-test".to_string(),
            base_url: None,
        };

        assert_eq!(config.provider_id, "openai");
        assert!(config.base_url.is_none());
    }

    #[test]
    fn test_endpoint_new() {
        let endpoint = Endpoint::new("openai", "gpt-4");

        assert_eq!(endpoint.provider_id, "openai");
        assert_eq!(endpoint.model_id, "gpt-4");
    }

    #[test]
    fn test_endpoint_with_slash() {
        let endpoint = Endpoint::new("anthropic", "claude-3-5-sonnet-20241022");

        assert_eq!(endpoint.provider_id, "anthropic");
        assert_eq!(endpoint.model_id, "claude-3-5-sonnet-20241022");
    }

    #[test]
    fn test_finish_reason_some() {
        let choice = Choice {
            index: 0,
            message: ChatMessage {
                role: MessageRole::Assistant,
                content: MessageContent::Text("Test".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
            finish_reason: Some("stop".to_string()),
        };

        assert_eq!(choice.finish_reason, Some("stop".to_string()));
    }

    #[test]
    fn test_finish_reason_none() {
        let choice = Choice {
            index: 0,
            message: ChatMessage {
                role: MessageRole::Assistant,
                content: MessageContent::Text("Test".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
            finish_reason: None,
        };

        assert!(choice.finish_reason.is_none());
    }
}
