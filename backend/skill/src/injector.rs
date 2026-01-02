use crate::registry::SkillRegistry;
use crate::types::{Skill, SkillCategory};
use std::sync::Arc;

/// Configuration for skill injection
#[derive(Debug, Clone)]
pub struct InjectionConfig {
    /// Maximum number of skills to inject
    pub max_skills: usize,
    /// Maximum examples per skill
    pub max_examples_per_skill: usize,
    /// Target language for the task
    pub target_language: Option<String>,
}

impl Default for InjectionConfig {
    fn default() -> Self {
        Self {
            max_skills: 5,
            max_examples_per_skill: 2,
            target_language: None,
        }
    }
}

/// Skill injector for adding relevant skills to LLM prompts
#[derive(Debug, Clone)]
pub struct SkillInjector {
    registry: SkillRegistry,
    config: InjectionConfig,
}

impl SkillInjector {
    pub fn new(registry: SkillRegistry) -> Self {
        Self {
            registry,
            config: InjectionConfig::default(),
        }
    }

    pub fn with_config(registry: SkillRegistry, config: InjectionConfig) -> Self {
        Self { registry, config }
    }

    /// Inject relevant skills into a prompt
    pub fn inject_to_prompt(&self, task: &str, base_prompt: &str) -> String {
        let skills = self.find_relevant_skills(task);

        if skills.is_empty() {
            return base_prompt.to_string();
        }

        let skills_section = self.format_skills(&skills);
        format!(
            "{}\n\n## Relevant Skills\n\n{}",
            base_prompt, skills_section
        )
    }

    /// Find relevant skills for a task
    pub fn find_relevant_skills(&self, task: &str) -> Vec<Arc<Skill>> {
        let category = self.infer_category(task);
        let language = self.config.target_language.as_deref();

        // First try to find by category
        let category_skills = self.registry.by_category(category);

        // Also do semantic search
        let semantic_skills =
            self.registry
                .find_relevant(task, language, self.config.max_skills * 2);

        // Combine and deduplicate
        let mut combined: Vec<_> = category_skills.into_iter().chain(semantic_skills).collect();

        // Deduplicate while preserving order
        let mut seen = std::collections::HashSet::new();
        combined.retain(|s: &Arc<Skill>| {
            let id = format!("{:?}", s.id);
            seen.insert(id)
        });

        // Take top N
        combined.into_iter().take(self.config.max_skills).collect()
    }

    /// Format skills as markdown for prompt injection
    pub fn format_skills(&self, skills: &[Arc<Skill>]) -> String {
        skills
            .iter()
            .map(|skill| self.format_skill(skill.as_ref()))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n")
    }

    /// Format a single skill as markdown
    pub fn format_skill(&self, skill: &Skill) -> String {
        let mut parts = vec![];

        // Header
        parts.push(format!("### {}", skill.name));
        parts.push(format!("*{}*", skill.description));

        // Content
        parts.push("".to_string());
        parts.push("**Knowledge:**".to_string());
        parts.push(skill.content.clone());

        // Examples (limited)
        if !skill.examples.is_empty() {
            parts.push("".to_string());
            parts.push("**Examples:**".to_string());
            for example in skill
                .examples
                .iter()
                .take(self.config.max_examples_per_skill)
            {
                parts.push(format!("- Input: `{}`", example.input));
                parts.push(format!("  Output: `{}`", example.output));
                if !example.explanation.is_empty() {
                    parts.push(format!("  *{}*", example.explanation));
                }
            }
        }

        // Related tools
        if !skill.related_tools.is_empty() {
            parts.push("".to_string());
            parts.push(format!(
                "**Related Tools:** {}",
                skill.related_tools.join(", ")
            ));
        }

        parts.join("\n")
    }

    /// Infer the category from a task description
    pub fn infer_category(&self, task: &str) -> SkillCategory {
        let task_lower = task.to_lowercase();

        // Keywords for each category
        let syntax_keywords = [
            "parse", "syntax", "ast", "tree", "grammar", "token", "lexer",
        ];
        let semantic_keywords = ["type", "check", "analyze", "semantic", "meaning", "scope"];
        let refactoring_keywords = [
            "refactor",
            "extract",
            "rename",
            "inline",
            "restructure",
            "transform",
        ];
        let project_keywords = [
            "project",
            "architecture",
            "structure",
            "codebase",
            "convention",
        ];

        // Count matches for each category
        let mut scores = std::collections::HashMap::new();

        for keyword in syntax_keywords {
            if task_lower.contains(keyword) {
                *scores.entry(SkillCategory::new("Syntax")).or_insert(0) += 1;
            }
        }

        for keyword in semantic_keywords {
            if task_lower.contains(keyword) {
                *scores.entry(SkillCategory::new("Semantic")).or_insert(0) += 1;
            }
        }

        for keyword in refactoring_keywords {
            if task_lower.contains(keyword) {
                *scores.entry(SkillCategory::new("Refactoring")).or_insert(0) += 1;
            }
        }

        for keyword in project_keywords {
            if task_lower.contains(keyword) {
                *scores.entry(SkillCategory::new("Project")).or_insert(0) += 1;
            }
        }

        // Return category with highest score, or LanguageSpecific as default
        scores
            .into_iter()
            .max_by_key(|(_, score)| *score)
            .map(|(cat, _)| cat)
            .unwrap_or(SkillCategory::new("LanguageSpecific"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SkillExample, SkillId, SkillMetadata};
    use std::collections::HashSet;

    fn create_test_skill(
        name: &str,
        description: &str,
        content: &str,
        category: SkillCategory,
    ) -> Skill {
        Skill {
            id: SkillId::new(category, name, "Rust"),
            name: name.into(),
            description: description.into(),
            content: content.into(),
            examples: vec![SkillExample {
                input: "test input".into(),
                output: "test output".into(),
                explanation: "test explanation".into(),
            }],
            related_tools: vec!["test::tool".into()],
            metadata: SkillMetadata {
                language: "Rust".into(),
                version: "1.0".into(),
                author: None,
                tags: HashSet::from_iter(vec!["test".into()]),
            },
        }
    }

    #[test]
    fn test_inject_to_prompt() {
        let mut registry = SkillRegistry::new();
        registry
            .register(create_test_skill(
                "Test Skill",
                "A test skill",
                "Test content",
                SkillCategory::new("Syntax"),
            ))
            .unwrap();

        // Verify the registry has the skill
        assert_eq!(registry.count(), 1);

        let injector = SkillInjector::new(registry);

        // Use a task with syntax keyword so category inference matches
        let result = injector.inject_to_prompt("Parse syntax tree", "Base prompt");

        assert!(result.contains("Base prompt"));
        assert!(result.contains("Relevant Skills"));
        assert!(result.contains("Test Skill"));
    }

    #[test]
    fn test_format_skill() {
        let skill = create_test_skill(
            "Test Skill",
            "A test skill",
            "Test content",
            SkillCategory::new("Syntax"),
        );

        let injector = SkillInjector::new(SkillRegistry::new());
        let formatted = injector.format_skill(&skill);

        assert!(formatted.contains("### Test Skill"));
        assert!(formatted.contains("*A test skill*"));
        assert!(formatted.contains("Test content"));
        assert!(formatted.contains("test input"));
        assert!(formatted.contains("test output"));
        assert!(formatted.contains("test::tool"));
    }

    #[test]
    fn test_infer_category() {
        let injector = SkillInjector::new(SkillRegistry::new());

        assert_eq!(
            injector.infer_category("Parse the syntax tree").as_str(),
            "Syntax"
        );
        assert_eq!(
            injector.infer_category("Type check this code").as_str(),
            "Semantic"
        );
        assert_eq!(
            injector.infer_category("Refactor this function").as_str(),
            "Refactoring"
        );
        assert_eq!(
            injector.infer_category("Project structure").as_str(),
            "Project"
        );
    }

    #[test]
    fn test_no_skills_returns_base_prompt() {
        let injector = SkillInjector::new(SkillRegistry::new());
        let result = injector.inject_to_prompt("test task", "Base prompt");

        assert_eq!(result, "Base prompt");
    }
}
