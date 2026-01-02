use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

/// 技能标识符，唯一标识一个技能
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkillId {
    pub category: SkillCategory,
    pub name: String,
    pub language: String,
}

impl SkillId {
    pub fn new(
        category: SkillCategory,
        name: impl Into<String>,
        language: impl Into<String>,
    ) -> Self {
        Self {
            category,
            name: name.into(),
            language: language.into(),
        }
    }
}

/// 技能类别（基于动态字符串）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SkillCategory(#[serde(deserialize_with = "deserialize_category")] pub String);

impl SkillCategory {
    /// 创建新类别
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// 获取类别名称
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 常见的预定义类别（用于向后兼容）
    pub const SYNTAX: &'static str = "Syntax";
    pub const SEMANTIC: &'static str = "Semantic";
    pub const PROJECT: &'static str = "Project";
    pub const REFACTORING: &'static str = "Refactoring";
    pub const LANGUAGE_SPECIFIC: &'static str = "LanguageSpecific";
}

impl fmt::Display for SkillCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for SkillCategory {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for SkillCategory {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<str> for SkillCategory {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// 自定义反序列化器以规范化类别名称
fn deserialize_category<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    // 规范化：首字母大写，处理 snake_case
    Ok(s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect())
}

/// 具有结构化知识的技能定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: SkillId,
    pub name: String,
    pub description: String,
    pub content: String,
    pub examples: Vec<SkillExample>,
    pub related_tools: Vec<String>,
    pub metadata: SkillMetadata,
}

impl Skill {
    pub fn validate(&self) -> Result<(), SkillError> {
        if self.name.is_empty() {
            return Err(SkillError::InvalidSkill("name cannot be empty".into()));
        }
        if self.description.is_empty() {
            return Err(SkillError::InvalidSkill(
                "description cannot be empty".into(),
            ));
        }
        if self.content.is_empty() {
            return Err(SkillError::InvalidSkill("content cannot be empty".into()));
        }
        Ok(())
    }
}

/// 演示技能使用的示例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExample {
    pub input: String,
    pub output: String,
    pub explanation: String,
}

/// 与技能关联的元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub language: String,
    pub version: String,
    pub author: Option<String>,
    pub tags: HashSet<String>,
}

/// 与技能相关的错误
#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    #[error("invalid skill: {0}")]
    InvalidSkill(String),

    #[error("skill not found: {0}")]
    NotFound(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_id_unique() {
        let id1 = SkillId::new(SkillCategory::new("Syntax"), "macro_rules", "Rust");
        let id2 = SkillId::new(SkillCategory::new("Syntax"), "macro_rules", "Rust");
        let id3 = SkillId::new(SkillCategory::new("Semantic"), "macro_rules", "Rust");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_skill_category_newtype() {
        let cat1 = SkillCategory::new("CustomCategory");
        let cat2 = SkillCategory::from("another_category");
        let cat3: SkillCategory = "ThirdCategory".into();

        assert_eq!(cat1.as_str(), "CustomCategory");
        // 注意：规范化仅在反序列化时发生，
        // 而不是通过 From/Into trait 实现
        assert_eq!(cat2.as_str(), "another_category");
        assert_eq!(cat3.as_str(), "ThirdCategory");
    }

    #[test]
    fn test_skill_validation_success() {
        let skill = Skill {
            id: SkillId::new(SkillCategory::new("Syntax"), "test", "Rust"),
            name: "Test Skill".into(),
            description: "A test skill".into(),
            content: "Some content".into(),
            examples: vec![],
            related_tools: vec![],
            metadata: SkillMetadata {
                language: "Rust".into(),
                version: "1.0".into(),
                author: None,
                tags: HashSet::from_iter(vec!["test".into()]),
            },
        };

        assert!(skill.validate().is_ok());
    }

    #[test]
    fn test_skill_validation_empty_name() {
        let skill = Skill {
            id: SkillId::new(SkillCategory::new("Syntax"), "test", "Rust"),
            name: "".into(),
            description: "A test skill".into(),
            content: "Some content".into(),
            examples: vec![],
            related_tools: vec![],
            metadata: SkillMetadata {
                language: "Rust".into(),
                version: "1.0".into(),
                author: None,
                tags: HashSet::new(),
            },
        };

        assert!(skill.validate().is_err());
    }

    #[test]
    fn test_category_from_str() {
        let cat: SkillCategory = "TestCategory".into();
        assert_eq!(cat.as_str(), "TestCategory");
    }

    #[test]
    fn test_category_display() {
        let cat = SkillCategory::new("MyCategory");
        assert_eq!(cat.to_string(), "MyCategory");
    }
}
