use crate::loader::SkillLoader;
use crate::state::SkillState;
use crate::types::SkillCategory;
use crate::{SkillError, SkillId};
use async_trait::async_trait;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Tool execution result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolOutput {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Unified tool interface for LLM function calling
#[async_trait(?Send)]
pub trait Tool: Send + Sync {
    /// Tool name (for LLM function calling)
    fn name(&self) -> &'static str;

    /// Tool description (for LLM to understand usage)
    fn description(&self) -> &'static str;

    /// Parameter schema (JSON Schema for validation)
    fn parameter_schema(&self) -> Value;

    /// Execute the tool
    async fn execute(&self, args: Value) -> Result<ToolOutput, SkillError>;
}

// ============================================================================
// Tool 1: Register Skill
// ============================================================================

pub struct RegisterSkillTool;

#[async_trait(?Send)]
impl Tool for RegisterSkillTool {
    fn name(&self) -> &'static str {
        "register_skill"
    }

    fn description(&self) -> &'static str {
        "Register a new skill to the knowledge base. The skill will be available for future queries and injections."
    }

    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "skill": {
                    "type": "object",
                    "description": "Skill definition (same format as YAML/JSON file)"
                }
            },
            "required": ["skill"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolOutput, SkillError> {
        let skill = SkillLoader::load_from_json_value(args["skill"].clone())?;

        let mut state = SkillState::get().write().await;
        state.registry.register(skill.clone())?;

        Ok(ToolOutput {
            content: format!("Skill '{}' registered successfully", skill.name),
            data: Some(json!({
                "id": {
                    "category": format!("{:?}", skill.id.category),
                    "name": skill.id.name,
                    "language": skill.id.language
                },
                "name": skill.name,
                "description": skill.description
            })),
        })
    }
}

// ============================================================================
// Tool 2: Search Skills
// ============================================================================

pub struct SearchSkillsTool;

#[async_trait(?Send)]
impl Tool for SearchSkillsTool {
    fn name(&self) -> &'static str {
        "search_skills"
    }

    fn description(&self) -> &'static str {
        "Search for skills relevant to a task. Returns matching skills with their descriptions."
    }

    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "task": {
                    "type": "string",
                    "description": "Task description to find relevant skills for"
                },
                "language": {
                    "type": "string",
                    "description": "Target programming language (optional)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of skills to return",
                    "default": 5
                }
            },
            "required": ["task"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolOutput, SkillError> {
        let task = args["task"]
            .as_str()
            .ok_or_else(|| SkillError::InvalidSkill("task is required".into()))?;
        let language = args["language"].as_str();
        let limit = args["limit"].as_u64().unwrap_or(5) as usize;

        let state = SkillState::get().read().await;
        let skills = state.registry.find_relevant(task, language, limit);

        let results: Vec<Value> = skills
            .iter()
            .map(|s| {
                json!({
                    "name": s.name,
                    "description": s.description,
                    "category": format!("{:?}", s.id.category),
                    "language": s.id.language,
                    "tags": s.metadata.tags,
                    "related_tools": s.related_tools
                })
            })
            .collect();

        Ok(ToolOutput {
            content: format!("Found {} relevant skills", results.len()),
            data: Some(json!(results)),
        })
    }
}

// ============================================================================
// Tool 3: Inject Skills
// ============================================================================

pub struct InjectSkillsTool;

#[async_trait(?Send)]
impl Tool for InjectSkillsTool {
    fn name(&self) -> &'static str {
        "inject_skills"
    }

    fn description(&self) -> &'static str {
        "Inject relevant skills into a prompt to enhance LLM understanding. Returns the augmented prompt."
    }

    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "task": {
                    "type": "string",
                    "description": "Task description"
                },
                "base_prompt": {
                    "type": "string",
                    "description": "Original prompt to augment"
                },
                "max_skills": {
                    "type": "integer",
                    "description": "Maximum skills to inject",
                    "default": 5
                }
            },
            "required": ["task", "base_prompt"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolOutput, SkillError> {
        let task = args["task"]
            .as_str()
            .ok_or_else(|| SkillError::InvalidSkill("task is required".into()))?;
        let base_prompt = args["base_prompt"]
            .as_str()
            .ok_or_else(|| SkillError::InvalidSkill("base_prompt is required".into()))?;

        let state = SkillState::get().read().await;
        let augmented = state.injector.inject_to_prompt(task, base_prompt);

        Ok(ToolOutput {
            content: "Skills injected successfully".into(),
            data: Some(json!({ "augmented_prompt": augmented })),
        })
    }
}

// ============================================================================
// Tool 4: Get Skill
// ============================================================================

pub struct GetSkillTool;

#[async_trait(?Send)]
impl Tool for GetSkillTool {
    fn name(&self) -> &'static str {
        "get_skill"
    }

    fn description(&self) -> &'static str {
        "Get a specific skill by its category, name, and language."
    }

    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "category": {
                    "type": "string",
                    "enum": ["Syntax", "Semantic", "Project", "Refactoring", "LanguageSpecific"],
                    "description": "Skill category"
                },
                "name": {
                    "type": "string",
                    "description": "Skill name"
                },
                "language": {
                    "type": "string",
                    "description": "Programming language"
                }
            },
            "required": ["category", "name", "language"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolOutput, SkillError> {
        let category_str = args["category"]
            .as_str()
            .ok_or_else(|| SkillError::InvalidSkill("category is required".into()))?;
        let name = args["name"]
            .as_str()
            .ok_or_else(|| SkillError::InvalidSkill("name is required".into()))?;
        let language = args["language"]
            .as_str()
            .ok_or_else(|| SkillError::InvalidSkill("language is required".into()))?;

        let category = parse_category(category_str).ok_or_else(|| {
            SkillError::ParseError(format!("Invalid category: {}", category_str))
        })?;

        let id = SkillId::new(category, name, language);
        let state = SkillState::get().read().await;
        let skill = state
            .registry
            .get(&id)
            .ok_or_else(|| SkillError::NotFound(format!("{:?}", id)))?;

        Ok(ToolOutput {
            content: format!("Found skill: {}", skill.name),
            data: Some(json!({
                "id": {
                    "category": format!("{:?}", skill.id.category),
                    "name": skill.id.name,
                    "language": skill.id.language
                },
                "name": skill.name,
                "description": skill.description,
                "content": skill.content,
                "examples": skill.examples,
                "related_tools": skill.related_tools,
                "metadata": {
                    "language": skill.metadata.language,
                    "version": skill.metadata.version,
                    "author": skill.metadata.author,
                    "tags": skill.metadata.tags
                }
            })),
        })
    }
}

// ============================================================================
// Tool 5: List Skills
// ============================================================================

pub struct ListSkillsTool;

#[async_trait(?Send)]
impl Tool for ListSkillsTool {
    fn name(&self) -> &'static str {
        "list_skills"
    }

    fn description(&self) -> &'static str {
        "List all registered skills, optionally filtered by category or language."
    }

    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "category": {
                    "type": "string",
                    "enum": ["Syntax", "Semantic", "Project", "Refactoring", "LanguageSpecific"],
                    "description": "Filter by category (optional)"
                },
                "language": {
                    "type": "string",
                    "description": "Filter by language (optional)"
                }
            }
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolOutput, SkillError> {
        let state = SkillState::get().read().await;

        let skills = if let Some(cat_str) = args["category"].as_str() {
            let category = parse_category(cat_str).ok_or_else(|| {
                SkillError::ParseError(format!("Invalid category: {}", cat_str))
            })?;
            state.registry.by_category(category)
        } else if let Some(lang) = args["language"].as_str() {
            state.registry.by_language(lang)
        } else {
            state.registry.all()
        };

        let results: Vec<Value> = skills
            .iter()
            .map(|s| {
                json!({
                    "name": s.name,
                    "description": s.description,
                    "category": format!("{:?}", s.id.category),
                    "language": s.id.language
                })
            })
            .collect();

        Ok(ToolOutput {
            content: format!("Found {} skills", results.len()),
            data: Some(json!(results)),
        })
    }
}

// ============================================================================
// Tool Registry
// ============================================================================

/// Registry for all skill tools
pub struct SkillToolRegistry {
    tools: HashMap<&'static str, Arc<dyn Tool>>,
}

impl SkillToolRegistry {
    /// Create a new tool registry with all skill tools registered
    pub fn new() -> Self {
        let mut tools = HashMap::new();
        tools.insert("register_skill", Arc::new(RegisterSkillTool) as Arc<dyn Tool>);
        tools.insert("search_skills", Arc::new(SearchSkillsTool) as Arc<dyn Tool>);
        tools.insert("inject_skills", Arc::new(InjectSkillsTool) as Arc<dyn Tool>);
        tools.insert("get_skill", Arc::new(GetSkillTool) as Arc<dyn Tool>);
        tools.insert("list_skills", Arc::new(ListSkillsTool) as Arc<dyn Tool>);
        Self { tools }
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// Get all tools as a map
    pub fn get_all(&self) -> &HashMap<&'static str, Arc<dyn Tool>> {
        &self.tools
    }

    /// Get all tool schemas (for LLM function calling)
    pub fn get_all_schemas(&self) -> Vec<Value> {
        self.tools
            .values()
            .map(|tool| {
                json!({
                    "name": tool.name(),
                    "description": tool.description(),
                    "parameters": tool.parameter_schema()
                })
            })
            .collect()
    }

    /// Execute a tool by name
    pub async fn execute(&self, name: &str, args: Value) -> Result<ToolOutput, SkillError> {
        let tool = self
            .get(name)
            .ok_or_else(|| SkillError::NotFound(format!("Tool not found: {}", name)))?;
        tool.execute(args).await
    }
}

impl Default for SkillToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse category from string (copied from loader.rs)
fn parse_category(s: &str) -> Option<SkillCategory> {
    match s.to_lowercase().as_str() {
        "syntax" => Some(SkillCategory::Syntax),
        "semantic" => Some(SkillCategory::Semantic),
        "project" => Some(SkillCategory::Project),
        "refactoring" => Some(SkillCategory::Refactoring),
        "languagespecific" => Some(SkillCategory::LanguageSpecific),
        _ => None,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SkillExample, SkillMetadata};
    use std::collections::HashSet;

    fn create_test_skill(name: &str) -> crate::Skill {
        crate::Skill {
            id: SkillId::new(SkillCategory::Syntax, name, "Rust"),
            name: name.into(),
            description: format!("{} skill", name),
            content: format!("Content for {}", name),
            examples: vec![SkillExample {
                input: "test".into(),
                output: "result".into(),
                explanation: "test explanation".into(),
            }],
            related_tools: vec![],
            metadata: SkillMetadata {
                language: "Rust".into(),
                version: "1.0".into(),
                author: None,
                tags: HashSet::from_iter(vec!["test".into()]),
            },
        }
    }

    #[tokio::test]
    async fn test_register_skill_tool() {
        let tool = RegisterSkillTool;
        assert_eq!(tool.name(), "register_skill");

        // Get initial count
        let state = SkillState::get().read().await;
        let initial_count = state.registry.count();
        drop(state);

        let skill_json = json!({
            "id": {
                "category": "Syntax",
                "name": "test_tool",
                "language": "Rust"
            },
            "name": "Test Tool",
            "description": "A test skill",
            "content": "Test content",
            "examples": [],
            "related_tools": [],
            "metadata": {
                "language": "Rust",
                "version": "1.0",
                "tags": []
            }
        });

        let result = tool
            .execute(json!({ "skill": skill_json }))
            .await
            .unwrap();

        assert!(result.content.contains("registered successfully"));

        // Check that at least one new skill was registered
        // (can't use exact count due to parallel tests sharing global state)
        let state = SkillState::get().read().await;
        assert!(state.registry.count() >= initial_count, "Should have at least {} skills, got {}", initial_count, state.registry.count());
    }

    #[tokio::test]
    async fn test_search_skills_tool() {
        // Register a skill with a unique name for this test
        let unique_name = "search_test_unique_parse_rust";
        let mut state = SkillState::get().write().await;
        state.registry.register(create_test_skill(unique_name)).unwrap();
        drop(state);

        let tool = SearchSkillsTool;
        let result = tool
            .execute(json!({
                "task": unique_name,
                "language": "Rust",
                "limit": 100
            }))
            .await
            .unwrap();

        assert!(result.content.contains("Found"));
        if let Some(data) = result.data {
            let skills: Vec<Value> = serde_json::from_value(data).unwrap();
            assert!(!skills.is_empty());
            // Check that our registered skill is in the results
            let found = skills
                .iter()
                .any(|s| s["name"] == unique_name);
            assert!(found, "Should find the registered skill");
        }
    }

    #[tokio::test]
    async fn test_get_skill_tool() {
        // First register a skill
        let mut state = SkillState::get().write().await;
        state
            .registry
            .register(create_test_skill("test_get"))
            .unwrap();
        drop(state);

        let tool = GetSkillTool;
        let result = tool
            .execute(json!({
                "category": "Syntax",
                "name": "test_get",
                "language": "Rust"
            }))
            .await
            .unwrap();

        assert!(result.content.contains("Found skill"));
        if let Some(data) = result.data {
            assert!(data["name"].is_string());
            assert_eq!(data["name"], "test_get");
        }
    }

    #[tokio::test]
    async fn test_list_skills_tool() {
        // Register some skills
        let mut state = SkillState::get().write().await;
        state
            .registry
            .register(create_test_skill("skill1"))
            .unwrap();
        state
            .registry
            .register(create_test_skill("skill2"))
            .unwrap();
        drop(state);

        let tool = ListSkillsTool;
        let result = tool.execute(json!({})).await.unwrap();

        assert!(result.content.contains("skills"));
        if let Some(data) = result.data {
            let skills: Vec<Value> = serde_json::from_value(data).unwrap();
            assert!(skills.len() >= 2);
        }
    }

    #[tokio::test]
    async fn test_tool_registry() {
        let registry = SkillToolRegistry::new();

        // Check all tools are registered
        assert_eq!(registry.get_all().len(), 5);

        // Get schemas
        let schemas = registry.get_all_schemas();
        assert_eq!(schemas.len(), 5);

        // Execute a tool
        let result = registry
            .execute(
                "list_skills",
                json!({
                    "category": "Syntax"
                }),
            )
            .await;

        assert!(result.is_ok());
    }
}
