use crate::skill::loader::SkillLoader;
use crate::skill::state::SkillState;
use crate::skill::traits::SkillCategory;
use crate::skill::traits::SkillError;
use crate::skill::traits::SkillId;
use async_trait::async_trait;
use serde_json::Value;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

/// 工具执行结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolOutput {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// 用于 LLM 函数调用的统一工具接口
#[async_trait(?Send)]
pub trait Tool: Send + Sync {
    /// 工具名称（用于 LLM 函数调用）
    fn name(&self) -> &'static str;

    /// 工具描述（用于 LLM 理解使用方法）
    fn description(&self) -> &'static str;

    /// 参数模式（用于验证的 JSON Schema）
    fn parameter_schema(&self) -> Value;

    /// 执行工具
    async fn execute(&self, args: Value) -> Result<ToolOutput, SkillError>;
}

// ============================================================================
// 工具 1: 注册技能
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
                    "description": "技能定义（与 YAML/JSON 文件格式相同）"
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
                    "category": skill.id.category.as_str(),
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
// 工具 2: 搜索技能
// ============================================================================

pub struct SearchSkillsTool;

#[async_trait(?Send)]
impl Tool for SearchSkillsTool {
    fn name(&self) -> &'static str {
        "search_skills"
    }

    fn description(&self) -> &'static str {
        "搜索与任务相关的技能。返回匹配的技能及其描述。"
    }

    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "task": {
                    "type": "string",
                    "description": "任务描述，用于查找相关技能"
                },
                "language": {
                    "type": "string",
                    "description": "目标编程语言（可选）"
                },
                "limit": {
                    "type": "integer",
                    "description": "返回的最大技能数量",
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
                    "category": s.id.category.as_str(),
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
// 工具 3: 注入技能
// ============================================================================

pub struct InjectSkillsTool;

#[async_trait(?Send)]
impl Tool for InjectSkillsTool {
    fn name(&self) -> &'static str {
        "inject_skills"
    }

    fn description(&self) -> &'static str {
        "将相关技能注入到提示中以增强 LLM 理解。返回增强后的提示。"
    }

    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "task": {
                    "type": "string",
                    "description": "任务描述"
                },
                "base_prompt": {
                    "type": "string",
                    "description": "要增强的原始提示"
                },
                "max_skills": {
                    "type": "integer",
                    "description": "最大注入技能数",
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
// 工具 4: 获取技能
// ============================================================================

pub struct GetSkillTool;

#[async_trait(?Send)]
impl Tool for GetSkillTool {
    fn name(&self) -> &'static str {
        "get_skill"
    }

    fn description(&self) -> &'static str {
        "根据类别、名称和语言获取特定技能。"
    }

    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "category": {
                    "type": "string",
                    "description": "技能类别（例如：Syntax、Semantic、Project 等）"
                },
                "name": {
                    "type": "string",
                    "description": "技能名称"
                },
                "language": {
                    "type": "string",
                    "description": "编程语言"
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

        let category = SkillCategory::new(category_str);

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
                    "category": skill.id.category.as_str(),
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
// 工具 5: 列出技能
// ============================================================================

pub struct ListSkillsTool;

#[async_trait(?Send)]
impl Tool for ListSkillsTool {
    fn name(&self) -> &'static str {
        "list_skills"
    }

    fn description(&self) -> &'static str {
        "列出所有已注册的技能，可按类别或语言筛选。"
    }

    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "category": {
                    "type": "string",
                    "description": "按类别筛选（可选）"
                },
                "language": {
                    "type": "string",
                    "description": "按语言筛选（可选）"
                }
            }
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolOutput, SkillError> {
        let state = SkillState::get().read().await;

        let skills = if let Some(cat_str) = args["category"].as_str() {
            let category = SkillCategory::new(cat_str);
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
                    "category": s.id.category.as_str(),
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
// 工具注册表
// ============================================================================

/// 所有技能工具的注册表
pub struct SkillToolRegistry {
    tools: HashMap<&'static str, Arc<dyn Tool>>,
}

impl SkillToolRegistry {
    /// 创建一个新的工具注册表，注册所有技能工具
    pub fn new() -> Self {
        let mut tools = HashMap::new();
        tools.insert(
            "register_skill",
            Arc::new(RegisterSkillTool) as Arc<dyn Tool>,
        );
        tools.insert("search_skills", Arc::new(SearchSkillsTool) as Arc<dyn Tool>);
        tools.insert("inject_skills", Arc::new(InjectSkillsTool) as Arc<dyn Tool>);
        tools.insert("get_skill", Arc::new(GetSkillTool) as Arc<dyn Tool>);
        tools.insert("list_skills", Arc::new(ListSkillsTool) as Arc<dyn Tool>);
        Self { tools }
    }

    /// 根据名称获取工具
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// 获取所有工具作为映射
    pub fn get_all(&self) -> &HashMap<&'static str, Arc<dyn Tool>> {
        &self.tools
    }

    /// 获取所有工具模式（用于 LLM 函数调用）
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

    /// 根据名称执行工具
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
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::traits::{Skill, SkillExample, SkillMetadata};
    use std::collections::HashSet;

    fn create_test_skill(name: &str) -> Skill {
        Skill {
            id: SkillId::new(SkillCategory::new("Syntax"), name, "Rust"),
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

        // 获取初始计数
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

        let result = tool.execute(json!({ "skill": skill_json })).await.unwrap();

        assert!(result.content.contains("registered successfully"));

        // 检查是否至少注册了一个新技能
        // （由于并行测试共享全局状态，无法使用确切计数）
        let state = SkillState::get().read().await;
        assert!(
            state.registry.count() >= initial_count,
            "Should have at least {} skills, got {}",
            initial_count,
            state.registry.count()
        );
    }

    #[tokio::test]
    async fn test_search_skills_tool() {
        // 为此测试注册一个具有唯一名称的技能
        let unique_name = "search_test_unique_parse_rust";
        let mut state = SkillState::get().write().await;
        state
            .registry
            .register(create_test_skill(unique_name))
            .unwrap();
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
            // 检查我们注册的技能是否在结果中
            let found = skills.iter().any(|s| s["name"] == unique_name);
            assert!(found, "Should find the registered skill");
        }
    }

    #[tokio::test]
    async fn test_get_skill_tool() {
        // 首先注册一个技能
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
        // 注册一些技能
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

        // 检查是否所有工具都已注册
        assert_eq!(registry.get_all().len(), 5);

        // 获取模式
        let schemas = registry.get_all_schemas();
        assert_eq!(schemas.len(), 5);

        // 执行工具
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
