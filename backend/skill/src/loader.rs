use crate::types::{Skill, SkillCategory, SkillError, SkillExample, SkillId, SkillMetadata};
use serde::Deserialize;
use std::path::Path;

/// Configuration for loading skills
#[derive(Debug, Clone, Deserialize)]
pub struct SkillConfig {
    /// List of file paths to load skills from
    #[serde(default)]
    pub files: Vec<String>,

    /// Inline skill definitions as JSON strings
    #[serde(default)]
    pub inline_skills: Vec<serde_json::Value>,
}

/// Skill loader for parsing skills from various sources
pub struct SkillLoader;

impl SkillLoader {
    /// Load skills from a configuration
    pub fn from_config(config: &SkillConfig) -> Result<Vec<Skill>, SkillError> {
        let mut skills = Vec::new();

        // Load from files
        for file_path in &config.files {
            let file_skills = Self::load_from_file(Path::new(file_path))?;
            skills.extend(file_skills);
        }

        // Load inline skills
        for inline in &config.inline_skills {
            let skill = Self::load_from_json(inline.to_string())?;
            skills.push(skill);
        }

        Ok(skills)
    }

    /// Load a single skill from a file (YAML or JSON)
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

    /// Load skills from a YAML string
    /// Supports both single skill and array of skills
    pub fn load_from_yaml(content: &str) -> Result<Vec<Skill>, SkillError> {
        // Try parsing as a single skill first
        if let Ok(raw) = serde_yaml::from_str::<RawSkill>(content) {
            let skill = raw.into_skill()?;
            return Ok(vec![skill]);
        }

        // Try parsing as an array of skills
        if let Ok(raw_skills) = serde_yaml::from_str::<Vec<RawSkill>>(content) {
            let mut skills = Vec::new();
            for raw in raw_skills {
                skills.push(raw.into_skill()?);
            }
            return Ok(skills);
        }

        Err(SkillError::ParseError(
            "Invalid YAML format for skill".into(),
        ))
    }

    /// Load a single skill from a JSON string
    pub fn load_from_json(content: String) -> Result<Skill, SkillError> {
        let raw: RawSkill = serde_json::from_str(&content)
            .map_err(|e| SkillError::ParseError(format!("Invalid JSON: {}", e)))?;
        raw.into_skill()
    }

    /// Load a single skill from a JSON value
    pub fn load_from_json_value(value: serde_json::Value) -> Result<Skill, SkillError> {
        let raw: RawSkill = serde_json::from_value(value)
            .map_err(|e| SkillError::ParseError(format!("Invalid JSON value: {}", e)))?;
        raw.into_skill()
    }
}

/// Raw skill format for deserialization
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
        // Now accepts any string as a category
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
        // With dynamic categories, any category name is valid
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
