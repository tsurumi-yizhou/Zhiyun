use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Skill identifier uniquely identifying a skill
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

/// Category of the skill
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SkillCategory {
    /// Syntax-related skills (parsing, AST, patterns)
    Syntax,
    /// Semantic understanding (type checking, liveness, reachability)
    Semantic,
    /// Project-specific knowledge (architecture, conventions)
    Project,
    /// Refactoring patterns and transformations
    Refactoring,
    /// Language-specific idioms and best practices
    LanguageSpecific,
}

/// A skill definition with structured knowledge
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

/// An example demonstrating the skill usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExample {
    pub input: String,
    pub output: String,
    pub explanation: String,
}

/// Metadata associated with a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub language: String,
    pub version: String,
    pub author: Option<String>,
    pub tags: HashSet<String>,
}

/// Errors related to skills
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
        let id1 = SkillId::new(SkillCategory::Syntax, "macro_rules", "Rust");
        let id2 = SkillId::new(SkillCategory::Syntax, "macro_rules", "Rust");
        let id3 = SkillId::new(SkillCategory::Semantic, "macro_rules", "Rust");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_skill_validation_success() {
        let skill = Skill {
            id: SkillId::new(SkillCategory::Syntax, "test", "Rust"),
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
            id: SkillId::new(SkillCategory::Syntax, "test", "Rust"),
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
}
