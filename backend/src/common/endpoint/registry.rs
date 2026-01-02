// Model Registry - manages providers, models, and API calls
use futures::stream::Stream;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::common::endpoint::error::EndpointError;
use crate::common::endpoint::stream::*;
use crate::common::endpoint::traits::*;

// ============================================================================
// FileManager - Manages file state across providers
// ============================================================================

/// File manager (internal use)
pub struct FileManager {
    files: RwLock<HashMap<String, FileState>>,
}

impl FileManager {
    /// Create a new file manager
    pub fn new() -> Self {
        Self {
            files: RwLock::new(HashMap::new()),
        }
    }

    /// Add file record
    pub async fn add_file(&self, local_id: String, state: FileState) {
        let mut files = self.files.write().await;
        files.insert(local_id, state);
    }

    /// Get file ID for a specific provider
    /// Returns None if file needs to be uploaded
    pub async fn get_provider_file_id(&self, local_id: &str, provider_id: &str) -> Option<String> {
        let files = self.files.read().await;
        files
            .get(local_id)
            .and_then(|state| state.provider_files.get(provider_id))
            .map(|state| state.file_id.clone())
    }

    /// Mark file as uploaded to a provider
    pub async fn mark_uploaded(&self, local_id: &str, provider_id: &str, file_id: String) {
        let mut files = self.files.write().await;
        if let Some(state) = files.get_mut(local_id) {
            state.provider_files.insert(
                provider_id.to_string(),
                ProviderFileState {
                    file_id,
                    uploaded_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                },
            );
        }
    }

    /// List all files
    pub async fn list_files(&self) -> Vec<FileState> {
        let files = self.files.read().await;
        files.values().cloned().collect()
    }
}

impl Default for FileManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Model Registry
// ============================================================================

/// Model registry - manages providers and API calls
pub struct ModelRegistry {
    providers: RwLock<HashMap<String, ProviderInfo>>,
    provider_configs: RwLock<HashMap<String, ProviderConfig>>,
    file_manager: Arc<FileManager>,
    routing_model: RwLock<Option<Endpoint>>,
}

impl ModelRegistry {
    /// Create a new model registry
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
            provider_configs: RwLock::new(HashMap::new()),
            file_manager: Arc::new(FileManager::new()),
            routing_model: RwLock::new(None),
        }
    }

    /// Create a new model registry with a routing model
    pub fn with_routing(provider_id: &str, model_id: &str) -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
            provider_configs: RwLock::new(HashMap::new()),
            file_manager: Arc::new(FileManager::new()),
            routing_model: RwLock::new(Some(Endpoint::new(provider_id, model_id))),
        }
    }

    /// Set or update the routing model
    pub async fn set_routing_model(&self, provider_id: &str, model_id: &str) {
        let mut routing = self.routing_model.write().await;
        *routing = Some(Endpoint::new(provider_id, model_id));
    }

    /// Add provider configuration
    pub async fn add_provider(&self, config: ProviderConfig) {
        let mut configs = self.provider_configs.write().await;
        configs.insert(config.provider_id.clone(), config);
    }

    /// Get model info
    pub async fn get_model(&self, provider_id: &str, model_id: &str) -> Option<ModelInfo> {
        let providers = self.providers.read().await;
        providers
            .get(provider_id)
            .and_then(|p| p.models.get(model_id))
            .cloned()
    }

    /// List all files (cross-provider state)
    pub async fn list_files(&self) -> Vec<FileState> {
        self.file_manager.list_files().await
    }

    /// Get file manager (internal)
    pub fn file_manager(&self) -> &Arc<FileManager> {
        &self.file_manager
    }

    /// Load provider info from models.dev API
    pub async fn load_providers(&self) -> Result<(), EndpointError> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://models.dev/api.json")
            .send()
            .await
            .map_err(|e| EndpointError::FetchError(format!("Failed to fetch models.dev: {}", e)))?;

        if !response.status().is_success() {
            return Err(EndpointError::FetchError(format!(
                "models.dev returned status: {}",
                response.status()
            )));
        }

        let providers_data: Vec<ProviderInfo> = response
            .json()
            .await
            .map_err(|e| EndpointError::ParseError(format!("Failed to parse providers: {}", e)))?;

        let mut providers = self.providers.write().await;
        for provider in providers_data {
            providers.insert(provider.id.clone(), provider);
        }

        Ok(())
    }

    /// Get or create provider config
    pub async fn get_provider_config(
        &self,
        provider_id: &str,
    ) -> Result<ProviderConfig, EndpointError> {
        let configs = self.provider_configs.read().await;
        configs
            .get(provider_id)
            .cloned()
            .ok_or_else(|| EndpointError::ProviderNotFound(provider_id.to_string()))
    }

    /// Chat completion (non-streaming)
    pub async fn chat_completion(
        &self,
        endpoint: &Endpoint,
        messages: Vec<ChatMessage>,
        options: Option<ChatOptions>,
    ) -> Result<ChatResponse, EndpointError> {
        // Get provider config
        let config = self.get_provider_config(&endpoint.provider_id).await?;

        // Create async-openai client
        let client = if let Some(base_url) = &config.base_url {
            async_openai::Client::with_config(
                async_openai::config::OpenAIConfig::new()
                    .with_api_key(&config.api_key)
                    .with_api_base(base_url),
            )
        } else {
            async_openai::Client::with_config(
                async_openai::config::OpenAIConfig::new().with_api_key(&config.api_key),
            )
        };

        // Convert our ChatMessage types to async-openai types
        let openai_messages: Vec<async_openai::types::ChatCompletionRequestMessage> = messages
            .into_iter()
            .map(|msg| {
                let role = match msg.role {
                    MessageRole::System => async_openai::types::Role::System,
                    MessageRole::User => async_openai::types::Role::User,
                    MessageRole::Assistant => async_openai::types::Role::Assistant,
                    MessageRole::Tool => async_openai::types::Role::Tool,
                };

                // For async-openai 0.26, create appropriate message based on role
                match role {
                    async_openai::types::Role::System => {
                        let content = match msg.content {
                            MessageContent::Text(text) => {
                                async_openai::types::ChatCompletionRequestSystemMessageContent::Text(
                                    text,
                                )
                            }
                            _ => async_openai::types::ChatCompletionRequestSystemMessageContent::Text(
                                String::new(),
                            ),
                        };
                        async_openai::types::ChatCompletionRequestMessage::System(
                            async_openai::types::ChatCompletionRequestSystemMessage {
                                content,
                                ..Default::default()
                            },
                        )
                    }
                    async_openai::types::Role::User => {
                        let content = match msg.content {
                            MessageContent::Text(text) => {
                                async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                                    text,
                                )
                            }
                            _ => async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                                String::new(),
                            ),
                        };
                        async_openai::types::ChatCompletionRequestMessage::User(
                            async_openai::types::ChatCompletionRequestUserMessage {
                                content,
                                ..Default::default()
                            },
                        )
                    }
                    async_openai::types::Role::Assistant => {
                        async_openai::types::ChatCompletionRequestMessage::Assistant(
                            async_openai::types::ChatCompletionRequestAssistantMessage {
                                content: Some(match msg.content {
                                    MessageContent::Text(text) => {
                                        async_openai::types::
                                            ChatCompletionRequestAssistantMessageContent::Text(
                                                text,
                                            )
                                    }
                                    _ => {
                                        async_openai::types::
                                            ChatCompletionRequestAssistantMessageContent::Text(
                                                String::new(),
                                            )
                                    }
                                }),
                                tool_calls: msg.tool_calls.map(|calls| {
                                    calls
                                        .into_iter()
                                        .map(|call| {
                                            async_openai::types::ChatCompletionMessageToolCall {
                                                id: call.id,
                                                r#type: async_openai::types::ChatCompletionToolType::Function,
                                                function: async_openai::types::FunctionCall {
                                                    name: call.function.name,
                                                    arguments: call.function.arguments,
                                                },
                                            }
                                        })
                                        .collect()
                                }),
                                ..Default::default()
                            },
                        )
                    }
                    async_openai::types::Role::Tool => {
                        async_openai::types::ChatCompletionRequestMessage::Tool(
                            async_openai::types::ChatCompletionRequestToolMessage {
                                content: match msg.content {
                                    MessageContent::Text(text) => {
                                        async_openai::types::ChatCompletionRequestToolMessageContent::Text(text)
                                    }
                                    _ => async_openai::types::ChatCompletionRequestToolMessageContent::Text(
                                        String::new(),
                                    ),
                                },
                                ..Default::default()
                            },
                        )
                    }
                    async_openai::types::Role::Function => {
                        // Function role is not typically used in request messages
                        // Convert to Assistant with text content
                        async_openai::types::ChatCompletionRequestMessage::Assistant(
                            async_openai::types::ChatCompletionRequestAssistantMessage {
                                content: Some(match msg.content {
                                    MessageContent::Text(text) => {
                                        async_openai::types::
                                            ChatCompletionRequestAssistantMessageContent::Text(
                                                text,
                                            )
                                    }
                                    _ => {
                                        async_openai::types::
                                            ChatCompletionRequestAssistantMessageContent::Text(
                                                String::new(),
                                            )
                                    }
                                }),
                                ..Default::default()
                            },
                        )
                    }
                }
            })
            .collect();

        // Build request
        let mut builder = async_openai::types::CreateChatCompletionRequestArgs::default();
        let _ = builder.model(&endpoint.model_id).messages(openai_messages);

        // Add options
        if let Some(opts) = options {
            if let Some(max_tokens) = opts.max_tokens {
                let _ = builder.max_tokens(max_tokens);
            }
            if let Some(tools) = opts.tools {
                let tools_vec: Vec<async_openai::types::ChatCompletionTool> = tools
                    .into_iter()
                    .map(|tool| async_openai::types::ChatCompletionTool {
                        r#type: async_openai::types::ChatCompletionToolType::Function,
                        function: async_openai::types::FunctionObject {
                            name: tool.function.name,
                            description: Some(tool.function.description),
                            parameters: Some(tool.function.parameters),
                            strict: None,
                        },
                    })
                    .collect();
                let _ = builder.tools(tools_vec);
            }
        }

        let request = builder
            .build()
            .map_err(|e| EndpointError::InvalidRequest(format!("Invalid request: {}", e)))?;

        // Call API
        let response = client
            .chat()
            .create(request)
            .await
            .map_err(|e| EndpointError::ApiError(format!("API call failed: {}", e)))?;

        // Convert response back to our types
        let choices: Vec<Choice> = response
            .choices
            .into_iter()
            .map(|choice| Choice {
                index: choice.index as usize,
                message: ChatMessage {
                    role: match choice.message.role {
                        async_openai::types::Role::System => MessageRole::System,
                        async_openai::types::Role::User => MessageRole::User,
                        async_openai::types::Role::Assistant => MessageRole::Assistant,
                        async_openai::types::Role::Tool => MessageRole::Tool,
                        async_openai::types::Role::Function => MessageRole::Tool, // Map Function to Tool
                    },
                    content: MessageContent::Text(choice.message.content.unwrap_or_default()),
                    tool_calls: choice.message.tool_calls.map(|calls| {
                        calls
                            .into_iter()
                            .map(|call| ToolCall {
                                id: call.id,
                                function: FunctionCall {
                                    name: call.function.name,
                                    arguments: call.function.arguments,
                                },
                            })
                            .collect()
                    }),
                    tool_call_id: None,
                },
                finish_reason: choice.finish_reason.map(|r| format!("{:?}", r)),
            })
            .collect();

        let usage = response.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
            cache_read_tokens: None,
        });

        Ok(ChatResponse {
            id: response.id,
            model: response.model,
            choices,
            usage,
        })
    }

    /// Route a prompt to the best models using the routing model
    /// Returns a ModelRoutingResult with a primary model and fallback options
    pub async fn route_models(&self, prompt: &str) -> Result<ModelRoutingResult, EndpointError> {
        let routing_endpoint = self.routing_model.read().await;
        let endpoint = routing_endpoint.as_ref().ok_or_else(|| {
            EndpointError::InvalidRequest("Routing model not configured".to_string())
        })?;

        // Build routing prompt
        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: MessageContent::Text(
                    "You are a model routing expert. Analyze the task and recommend the best models. \
                     Always respond with valid JSON containing 'primary' and 'fallbacks' fields."
                        .to_string(),
                ),
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: MessageContent::Text(format!(
                    "Task: {}\n\nRecommend the top 3 models for this task. \
                     Return as JSON: {{\"primary\": \"model-id\", \"fallbacks\": [\"model-id-1\", \"model-id-2\"]}}",
                    prompt
                )),
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let response = self.chat_completion(endpoint, messages, None).await?;

        // Parse response to extract model IDs
        if let Some(choice) = response.choices.first()
            && let MessageContent::Text(content) = &choice.message.content
        {
            // Try to extract JSON from the response
            let json_str = content
                .trim()
                .strip_prefix("```json")
                .or_else(|| content.trim().strip_prefix("```"))
                .unwrap_or(content.trim());
            let json_str = json_str.strip_suffix("```").unwrap_or(json_str);

            let result: serde_json::Value = serde_json::from_str(json_str).map_err(|_| {
                EndpointError::ParseError(format!(
                    "Invalid routing response: {}",
                    content.chars().take(100).collect::<String>()
                ))
            })?;

            let primary = result["primary"]
                .as_str()
                .ok_or_else(|| {
                    EndpointError::ParseError(
                        "Missing 'primary' field in routing response".to_string(),
                    )
                })?
                .to_string();

            let fallbacks = result["fallbacks"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            return Ok(ModelRoutingResult { primary, fallbacks });
        }

        Err(EndpointError::ApiError(
            "Empty routing response".to_string(),
        ))
    }

    /// Chat completion (streaming)
    pub async fn chat_completion_stream(
        &self,
        endpoint: &Endpoint,
        _messages: Vec<ChatMessage>,
        _options: Option<ChatOptions>,
    ) -> Result<Box<dyn Stream<Item = ChatStreamEvent> + Send + Unpin>, EndpointError> {
        // Get provider config
        let _config = self.get_provider_config(&endpoint.provider_id).await?;

        // TODO: Implement actual streaming API call
        // This is a placeholder - need to convert to async-openai types
        Err(EndpointError::ApiError("Not yet implemented".to_string()))
    }

    /// Upload file to provider
    pub async fn upload_file(
        &self,
        endpoint: &Endpoint,
        request: FileUploadRequest,
    ) -> Result<FileObject, EndpointError> {
        // Check if already uploaded to this provider
        let local_id = format!("{}:{}", request.filename, request.purpose as u32);
        if let Some(file_id) = self
            .file_manager
            .get_provider_file_id(&local_id, &endpoint.provider_id)
            .await
        {
            return Ok(FileObject {
                id: file_id,
                bytes: request.file.len() as u64,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                filename: request.filename,
                purpose: format!("{:?}", request.purpose).to_lowercase(),
            });
        }

        // Get provider config
        let _config = self.get_provider_config(&endpoint.provider_id).await?;

        // TODO: Implement actual file upload
        // This is a placeholder - need to convert to async-openai types
        Err(EndpointError::ApiError("Not yet implemented".to_string()))
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_manager_new() {
        let fm = FileManager::new();
        let files = fm.files.read().await;
        assert_eq!(files.len(), 0);
    }

    #[tokio::test]
    async fn test_file_manager_add_and_get() {
        let fm = FileManager::new();

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

        fm.add_file("local-1".to_string(), state).await;

        let file_id = fm.get_provider_file_id("local-1", "openai").await;
        assert_eq!(file_id, Some("file-abc".to_string()));
    }

    #[tokio::test]
    async fn test_file_manager_get_nonexistent() {
        let fm = FileManager::new();
        let file_id = fm.get_provider_file_id("local-1", "openai").await;
        assert_eq!(file_id, None);
    }

    #[tokio::test]
    async fn test_file_manager_mark_uploaded() {
        let fm = FileManager::new();

        let state = FileState {
            filename: "test.txt".to_string(),
            bytes: 1024,
            purpose: FilePurpose::Assistants,
            provider_files: HashMap::new(),
        };

        fm.add_file("local-1".to_string(), state).await;
        fm.mark_uploaded("local-1", "anthropic", "file-xyz".to_string())
            .await;

        let file_id = fm.get_provider_file_id("local-1", "anthropic").await;
        assert_eq!(file_id, Some("file-xyz".to_string()));
    }

    #[tokio::test]
    async fn test_file_manager_list() {
        let fm = FileManager::new();

        let state1 = FileState {
            filename: "test1.txt".to_string(),
            bytes: 100,
            purpose: FilePurpose::Assistants,
            provider_files: HashMap::new(),
        };

        let state2 = FileState {
            filename: "test2.txt".to_string(),
            bytes: 200,
            purpose: FilePurpose::Batch,
            provider_files: HashMap::new(),
        };

        fm.add_file("local-1".to_string(), state1).await;
        fm.add_file("local-2".to_string(), state2).await;

        let files = fm.list_files().await;
        assert_eq!(files.len(), 2);
    }

    #[tokio::test]
    async fn test_model_registry_new() {
        let registry = ModelRegistry::new();

        let providers = registry.providers.read().await;
        assert_eq!(providers.len(), 0);

        let configs = registry.provider_configs.read().await;
        assert_eq!(configs.len(), 0);
    }

    #[tokio::test]
    async fn test_model_registry_add_provider() {
        let registry = ModelRegistry::new();

        let config = ProviderConfig {
            provider_id: "openai".to_string(),
            api_key: "sk-test".to_string(),
            base_url: None,
        };

        registry.add_provider(config).await;

        let configs = registry.provider_configs.read().await;
        assert_eq!(configs.len(), 1);
        assert_eq!(configs.get("openai").unwrap().api_key, "sk-test");
    }

    #[tokio::test]
    async fn test_model_registry_get_model_empty() {
        let registry = ModelRegistry::new();

        let model = registry.get_model("openai", "gpt-4").await;
        assert!(model.is_none());
    }

    #[tokio::test]
    async fn test_model_registry_get_model() {
        let registry = ModelRegistry::new();

        {
            let mut providers = registry.providers.write().await;

            let mut models = HashMap::new();
            models.insert(
                "gpt-4".to_string(),
                ModelInfo {
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
                        cache_read: None,
                    },
                    limit: ModelLimit {
                        context: 8192,
                        output: 4096,
                    },
                },
            );

            providers.insert(
                "openai".to_string(),
                ProviderInfo {
                    id: "openai".to_string(),
                    name: "OpenAI".to_string(),
                    api: "https://api.openai.com/v1".to_string(),
                    env: vec!["OPENAI_API_KEY".to_string()],
                    doc: "https://docs.openai.com".to_string(),
                    models,
                },
            );
        } // Write lock is released here

        let model = registry.get_model("openai", "gpt-4").await;
        assert!(model.is_some());
        assert_eq!(model.unwrap().name, "GPT-4");
    }

    #[tokio::test]
    async fn test_model_registry_list_files_empty() {
        let registry = ModelRegistry::new();
        let files = registry.list_files().await;
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_endpoint_new() {
        let endpoint = Endpoint::new("openai", "gpt-4");
        assert_eq!(endpoint.provider_id, "openai");
        assert_eq!(endpoint.model_id, "gpt-4");
    }

    #[test]
    fn test_provider_config_with_base_url() {
        let config = ProviderConfig {
            provider_id: "custom".to_string(),
            api_key: "key".to_string(),
            base_url: Some("https://custom.api/v1".to_string()),
        };

        assert_eq!(config.base_url, Some("https://custom.api/v1".to_string()));
    }

    #[test]
    fn test_provider_config_default() {
        let config = ProviderConfig {
            provider_id: "openai".to_string(),
            api_key: "sk-xxx".to_string(),
            base_url: None,
        };

        assert!(config.base_url.is_none());
    }

    #[tokio::test]
    async fn test_file_manager_default() {
        let fm = FileManager::default();
        let files = fm.files.read().await;
        assert_eq!(files.len(), 0);
    }

    #[tokio::test]
    async fn test_model_registry_default() {
        let registry = ModelRegistry::default();
        let providers = registry.providers.read().await;
        assert_eq!(providers.len(), 0);
    }

    #[tokio::test]
    async fn test_file_state_cross_provider() {
        let fm = FileManager::new();

        let mut state = FileState {
            filename: "shared.txt".to_string(),
            bytes: 2048,
            purpose: FilePurpose::Vision,
            provider_files: HashMap::new(),
        };

        state.provider_files.insert(
            "openai".to_string(),
            ProviderFileState {
                file_id: "file-openai".to_string(),
                uploaded_at: 1000,
            },
        );

        state.provider_files.insert(
            "anthropic".to_string(),
            ProviderFileState {
                file_id: "file-anthropic".to_string(),
                uploaded_at: 2000,
            },
        );

        fm.add_file("shared".to_string(), state).await;

        assert_eq!(
            fm.get_provider_file_id("shared", "openai").await,
            Some("file-openai".to_string())
        );
        assert_eq!(
            fm.get_provider_file_id("shared", "anthropic").await,
            Some("file-anthropic".to_string())
        );
        assert_eq!(fm.get_provider_file_id("shared", "unknown").await, None);
    }

    #[tokio::test]
    async fn test_multiple_registries_independent() {
        let registry1 = ModelRegistry::new();
        let registry2 = ModelRegistry::new();

        registry1
            .add_provider(ProviderConfig {
                provider_id: "openai".to_string(),
                api_key: "key1".to_string(),
                base_url: None,
            })
            .await;

        registry2
            .add_provider(ProviderConfig {
                provider_id: "anthropic".to_string(),
                api_key: "key2".to_string(),
                base_url: None,
            })
            .await;

        let configs1 = registry1.provider_configs.read().await;
        let configs2 = registry2.provider_configs.read().await;

        assert_eq!(configs1.len(), 1);
        assert_eq!(configs2.len(), 1);
        assert!(configs2.get("openai").is_none());
        assert!(configs1.get("anthropic").is_none());
    }
}

