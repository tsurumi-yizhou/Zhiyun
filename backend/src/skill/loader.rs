use crate::skill::types::{Skill, SkillCategory, SkillError, SkillExample, SkillId, SkillMetadata};
use serde::Deserialize;
use std::path::Path;

/// 加载技能的配置
#[derive(Debug, Clone, Deserialize)]
pub struct SkillConfig {
    /// 用于加载技能的文件路径列表
    #[serde(default)]
    pub files: Vec<String>,

    /// 内联技能定义，为 JSON 字符串格式
    #[serde(default)]
    pub inline_skills: Vec<serde_json::Value>,
}

/// 用于从各种来源解析技能的技能加载器
pub struct SkillLoader;

impl SkillLoader {
    /// 从配置加载技能
    pub fn from_config(config: &SkillConfig) -> Result<Vec<Skill>, SkillError> {
        let mut skills = Vec::new();

        // 从文件加载
        for file_path in &config.files {
            let file_skills = Self::load_from_file(Path::new(file_path))?;
            skills.extend(file_skills);
        }

        // 加载内联技能
        for inline in &config.inline_skills {
            let skill = Self::load_from_json(inline.to_string())?;
            skills.push(skill);
        }

        Ok(skills)
    }

    /// 从文件加载单个技能（YAML 或 JSON）
    pub fn load_from_file(path: &Path) -> Result<Vec<Skill>, SkillError> {
        let content = std::fs::read_to_string(path)?;

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| SkillError::ParseError("No file extension".into()))?;

        match extension.to_lowercase().as_str() {
            "yaml" | "yml" => Self::load_from_yaml(&content),
            "json" => {
                let skill = Self::load_from_json(content)?;
                Ok(vec![skill])
            }
            _ => Err(SkillError::ParseError(format!(
                "Unsupported file extension: {}",
                extension
            ))),
        }
    }

    /// 从 YAML 字符串加载技能
    /// 支持单个技能和技能数组
    pub fn load_from_yaml(content: &str) -> Result<Vec<Skill>, SkillError> {
        // 首先尝试解析为单个技能
        if let Ok(raw) = serde_yaml::from_str::<RawSkill>(content) {
            let skill = raw.into_skill()?;
            return Ok(vec![skill]);
        }

        // 尝试解析为技能数组
        if let Ok(raw_skills) = serde_yaml::from_str::<Vec<RawSkill>>(content) {
            let mut skills = Vec::new();
            for raw in raw_skills {
                skills.push(raw.into_skill()?);
            }
            return Ok(skills);
        }

        Err(SkillError::ParseError(
            "技能的 YAML 格式无效".into(),
        ))
    }

    /// 从 JSON 字符串加载单个技能
    pub fn load_from_json(content: String) -> Result<Skill, SkillError> {
        let raw: RawSkill = serde_json::from_str(&content)
            .map_err(|e| SkillError::ParseError(format!("无效的 JSON: {}", e)))?;
        raw.into_skill()
    }

    /// 从 JSON 值加载单个技能
    pub fn load_from_json_value(value: serde_json::Value) -> Result<Skill, SkillError> {
        let raw: RawSkill = serde_json::from_value(value)
            .map_err(|e| SkillError::ParseError(format!("无效的 JSON 值: {}", e)))?;
        raw.into_skill()
    }
}

/// 用于反序列化的原始技能格式
#[derive(Debug, Deserialize)]
struct RawSkill {
    id: RawSkillId,
    name: String,
    description: String,
    content: String,
    #[serde(default)]
    examples: Vec<RawExample>,
    #[serde(default)]
    related_tools: Vec<String>,
    metadata: RawMetadata,
}

#[derive(Debug, Deserialize)]
struct RawSkillId {
    category: String,
    name: String,
    language: String,
}

#[derive(Debug, Deserialize)]
struct RawExample {
    input: String,
    output: String,
    explanation: String,
}

#[derive(Debug, Deserialize)]
struct RawMetadata {
    language: String,
    version: String,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

impl RawSkill {
    fn into_skill(self) -> Result<Skill, SkillError> {
        // 现在接受任何字符串作为类别
        let category = SkillCategory::new(&self.id.category);

        let id = SkillId::new(category, &self.id.name, &self.id.language);

        Ok(Skill {
            id: id.clone(),
            name: self.name,
            description: self.description,
            content: self.content,
            examples: self
                .examples
                .into_iter()
                .map(|e| SkillExample {
                    input: e.input,
                    output: e.output,
                    explanation: e.explanation,
                })
                .collect(),
            related_tools: self.related_tools,
            metadata: SkillMetadata {
                language: self.metadata.language,
                version: self.metadata.version,
                author: self.metadata.author,
                tags: self.metadata.tags.into_iter().collect(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const YAML_SKILL: &str = r#"
id:
  category: Syntax
  name: rust_macro_rules
  language: Rust
name: "Rust macro_rules! Syntax"
description: "How to parse Rust's macro_rules! macro"
content: |
  Rust's macro_rules! macro has the following characteristics...
examples:
  - input: "Parse macro definition"
    output: "Use TreeSitter query..."
    explanation: "Match macro_definition node"
related_tools:
  - syntax::parse
metadata:
  language: Rust
  version: "1.0"
  tags: ["macro", "syntax"]
"#;

    const JSON_SKILL: &str = r#"
{
  "id": {
    "category": "Syntax",
    "name": "rust_macro_rules",
    "language": "Rust"
  },
  "name": "Rust macro_rules! Syntax",
  "description": "How to parse Rust's macro_rules! macro",
  "content": "Rust's macro_rules! macro has the following characteristics...",
  "examples": [
    {
      "input": "Parse macro definition",
      "output": "Use TreeSitter query...",
      "explanation": "Match macro_definition node"
    }
  ],
  "related_tools": ["syntax::parse"],
  "metadata": {
    "language": "Rust",
    "version": "1.0",
    "tags": ["macro", "syntax"]
  }
}
"#;

    #[test]
    fn test_load_from_yaml() {
        let skills = SkillLoader::load_from_yaml(YAML_SKILL).unwrap();
        assert_eq!(skills.len(), 1);

        let skill = &skills[0];
        assert_eq!(skill.name, "Rust macro_rules! Syntax");
        assert_eq!(skill.id.category.as_str(), "Syntax");
        assert_eq!(skill.id.language, "Rust");
        assert_eq!(skill.examples.len(), 1);
        assert!(skill.metadata.tags.contains("macro"));
    }

    #[test]
    fn test_load_from_json() {
        let skill = SkillLoader::load_from_json(JSON_SKILL.to_string()).unwrap();

        assert_eq!(skill.name, "Rust macro_rules! Syntax");
        assert_eq!(skill.id.category.as_str(), "Syntax");
        assert_eq!(skill.id.language, "Rust");
        assert_eq!(skill.related_tools.len(), 1);
        assert_eq!(skill.related_tools[0], "syntax::parse");
    }

    #[test]
    fn test_from_config() {
        let config = SkillConfig {
            files: vec![],
            inline_skills: vec![serde_json::from_str(JSON_SKILL).unwrap()],
        };

        let skills = SkillLoader::from_config(&config).unwrap();
        assert_eq!(skills.len(), 1);
    }

    #[test]
    fn test_custom_category() {
        // 使用动态类别，任何类别名称都是有效的
        let custom_yaml = r#"
id:
  category: CustomCategory
  name: test
  language: Rust
name: "Test"
description: "Test"
content: "Test"
metadata:
  language: Rust
  version: "1.0"
  tags: []
"#;

        let result = SkillLoader::load_from_yaml(custom_yaml);
        assert!(result.is_ok());
        let skills = result.unwrap();
        assert_eq!(skills[0].id.category.as_str(), "CustomCategory");
    }
}
