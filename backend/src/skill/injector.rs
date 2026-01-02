use crate::skill::registry::SkillRegistry;
use crate::skill::types::{Skill, SkillCategory};
use std::sync::Arc;

/// 技能注入配置
#[derive(Debug, Clone)]
pub struct InjectionConfig {
    /// 最大注入技能数量
    pub max_skills: usize,
    /// 每个技能的最大示例数
    pub max_examples_per_skill: usize,
    /// 任务的目标语言
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

/// 用于将相关技能添加到 LLM 提示的技能注入器
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

    /// 将相关技能注入到提示中
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

    /// 为任务查找相关技能
    pub fn find_relevant_skills(&self, task: &str) -> Vec<Arc<Skill>> {
        let category = self.infer_category(task);
        let language = self.config.target_language.as_deref();

        // 首先按类别查找
        let category_skills = self.registry.by_category(category);

        // 同时进行语义搜索
        let semantic_skills =
            self.registry
                .find_relevant(task, language, self.config.max_skills * 2);

        // 合并并去重
        let mut combined: Vec<_> = category_skills.into_iter().chain(semantic_skills).collect();

        // 去重并保持顺序
        let mut seen = std::collections::HashSet::new();
        combined.retain(|s: &Arc<Skill>| {
            let id = format!("{:?}", s.id);
            seen.insert(id)
        });

        // 取前 N 个
        combined.into_iter().take(self.config.max_skills).collect()
    }

    /// 将技能格式化为 Markdown 用于提示注入
    pub fn format_skills(&self, skills: &[Arc<Skill>]) -> String {
        skills
            .iter()
            .map(|skill| self.format_skill(skill.as_ref()))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n")
    }

    /// 将单个技能格式化为 Markdown
    pub fn format_skill(&self, skill: &Skill) -> String {
        let mut parts = vec![];

        // 标题
        parts.push(format!("### {}", skill.name));
        parts.push(format!("*{}*", skill.description));

        // 内容
        parts.push("".to_string());
        parts.push("**Knowledge:**".to_string());
        parts.push(skill.content.clone());

        // 示例（限制数量）
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

        // 相关工具
        if !skill.related_tools.is_empty() {
            parts.push("".to_string());
            parts.push(format!(
                "**Related Tools:** {}",
                skill.related_tools.join(", ")
            ));
        }

        parts.join("\n")
    }

    /// 从任务描述中推断类别
    pub fn infer_category(&self, task: &str) -> SkillCategory {
        let task_lower = task.to_lowercase();

        // 每个类别的关键词
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

        // 统计每个类别的匹配数
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

        // 返回得分最高的类别，默认为 LanguageSpecific
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
    use crate::skill::types::{SkillExample, SkillId, SkillMetadata};
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

        // 验证注册表包含该技能
        assert_eq!(registry.count(), 1);

        let injector = SkillInjector::new(registry);

        // 使用包含语法关键字的任务，以便类别推断匹配
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
