use crate::skill::traits::{Skill, SkillCategory, SkillError, SkillId};
use std::collections::HashMap;
use std::sync::Arc;

/// 用于管理和索引技能的注册表
#[derive(Debug, Clone)]
pub struct SkillRegistry {
    skills: HashMap<SkillId, Arc<Skill>>,
    by_category: HashMap<SkillCategory, Vec<Arc<Skill>>>,
    by_language: HashMap<String, Vec<Arc<Skill>>>,
    by_tag: HashMap<String, Vec<Arc<Skill>>>,
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            by_category: HashMap::new(),
            by_language: HashMap::new(),
            by_tag: HashMap::new(),
        }
    }

    /// 注册新技能，更新所有索引
    pub fn register(&mut self, skill: Skill) -> Result<(), SkillError> {
        skill.validate()?;

        let id = skill.id.clone();
        let skill = Arc::new(skill);

        // 插入主存储
        self.skills.insert(id.clone(), skill.clone());

        // 更新类别索引
        self.by_category
            .entry(id.category)
            .or_default()
            .push(skill.clone());

        // 更新语言索引
        self.by_language
            .entry(id.language.clone())
            .or_default()
            .push(skill.clone());

        // 更新标签索引
        for tag in &skill.metadata.tags {
            self.by_tag
                .entry(tag.clone())
                .or_default()
                .push(skill.clone());
        }

        Ok(())
    }

    /// 一次性注册多个技能
    pub fn register_all(
        &mut self,
        skills: impl IntoIterator<Item = Skill>,
    ) -> Result<(), SkillError> {
        for skill in skills {
            self.register(skill)?;
        }
        Ok(())
    }

    /// Get a skill by its ID
    pub fn get(&self, id: &SkillId) -> Option<Arc<Skill>> {
        self.skills.get(id).cloned()
    }

    /// Find all skills in a category
    pub fn by_category(&self, category: SkillCategory) -> Vec<Arc<Skill>> {
        self.by_category.get(&category).cloned().unwrap_or_default()
    }

    /// Find all skills for a language
    pub fn by_language(&self, language: &str) -> Vec<Arc<Skill>> {
        self.by_language.get(language).cloned().unwrap_or_default()
    }

    /// Find all skills with a specific tag
    pub fn by_tag(&self, tag: &str) -> Vec<Arc<Skill>> {
        self.by_tag.get(tag).cloned().unwrap_or_default()
    }

    /// 根据任务描述查找相关技能
    /// 这是简化版本 - 在生产环境中，应使用向量嵌入进行语义搜索
    pub fn find_relevant(
        &self,
        task: &str,
        language: Option<&str>,
        limit: usize,
    ) -> Vec<Arc<Skill>> {
        let mut candidates: Vec<Arc<Skill>> = Vec::new();
        let task_lower = task.to_lowercase();

        // 如果指定了语言，则从特定语言的技能开始
        if let Some(lang) = language {
            candidates.extend(self.by_language(lang));
        } else {
            // 否则考虑所有技能
            candidates.extend(self.skills.values().cloned());
        }

        // 基于关键字匹配的简单相关性评分
        let mut scored: Vec<_> = candidates
            .into_iter()
            .map(|skill| {
                let score = calculate_relevance(&skill, &task_lower);
                (score, skill)
            })
            .filter(|(score, _)| *score > 0)
            .collect();

        // 按相关性分数降序排序
        scored.sort_by(|a, b| b.0.cmp(&a.0));

        // 取前 N 个
        scored
            .into_iter()
            .take(limit)
            .map(|(_, skill)| skill)
            .collect()
    }

    /// Get all registered skills
    pub fn all(&self) -> Vec<Arc<Skill>> {
        self.skills.values().cloned().collect()
    }

    /// Get count of registered skills
    pub fn count(&self) -> usize {
        self.skills.len()
    }

    /// Check if a skill is registered
    pub fn contains(&self, id: &SkillId) -> bool {
        self.skills.contains_key(id)
    }
}

/// 计算技能与任务的相关性分数
fn calculate_relevance(skill: &Skill, task: &str) -> usize {
    let mut score = 0;

    // 检查名称匹配
    if skill.name.to_lowercase().contains(task) {
        score += 10;
    }

    // 检查描述匹配
    if skill.description.to_lowercase().contains(task) {
        score += 5;
    }

    // 检查内容匹配
    if skill.content.to_lowercase().contains(task) {
        score += 3;
    }

    // 检查标签匹配
    for tag in &skill.metadata.tags {
        if tag.to_lowercase().contains(task) {
            score += 7;
        }
    }

    // 检查相关工具
    for tool in &skill.related_tools {
        if tool.to_lowercase().contains(task) {
            score += 4;
        }
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::traits::SkillMetadata;

    fn create_test_skill(
        category: SkillCategory,
        name: &str,
        language: &str,
        tags: Vec<&str>,
    ) -> Skill {
        Skill {
            id: SkillId::new(category, name, language),
            name: name.into(),
            description: format!("{} skill for {}", name, language),
            content: format!("Content for {}", name),
            examples: vec![],
            related_tools: vec![],
            metadata: SkillMetadata {
                language: language.into(),
                version: "1.0".into(),
                author: None,
                tags: tags.into_iter().map(String::from).collect(),
            },
        }
    }

    #[test]
    fn test_register_and_retrieve() {
        let mut registry = SkillRegistry::new();
        let skill = create_test_skill(
            SkillCategory::new("Syntax"),
            "test_skill",
            "Rust",
            vec!["test"],
        );

        let id = skill.id.clone();
        registry.register(skill).unwrap();

        assert_eq!(registry.count(), 1);
        assert!(registry.contains(&id));
        assert!(registry.get(&id).is_some());
    }

    #[test]
    fn test_index_consistency() {
        let mut registry = SkillRegistry::new();
        let skill = create_test_skill(
            SkillCategory::new("Syntax"),
            "parse_macro",
            "Rust",
            vec!["macro", "syntax"],
        );

        registry.register(skill.clone()).unwrap();

        // 检查类别索引
        let by_cat = registry.by_category(SkillCategory::new("Syntax"));
        assert_eq!(by_cat.len(), 1);
        assert_eq!(by_cat[0].name, "parse_macro");

        // 检查语言索引
        let by_lang = registry.by_language("Rust");
        assert_eq!(by_lang.len(), 1);

        // 检查标签索引
        let by_tag = registry.by_tag("macro");
        assert_eq!(by_tag.len(), 1);
    }

    #[test]
    fn test_find_relevant() {
        let mut registry = SkillRegistry::new();
        registry
            .register(create_test_skill(
                SkillCategory::new("Syntax"),
                "parse_rust",
                "Rust",
                vec!["parser"],
            ))
            .unwrap();
        registry
            .register(create_test_skill(
                SkillCategory::new("Semantic"),
                "type_check",
                "Rust",
                vec!["types"],
            ))
            .unwrap();
        registry
            .register(create_test_skill(
                SkillCategory::new("Syntax"),
                "parse_python",
                "Python",
                vec!["parser"],
            ))
            .unwrap();

        // 查找 Rust 解析技能
        let results = registry.find_relevant("parse", Some("Rust"), 10);
        assert!(!results.is_empty());
        assert!(results.iter().any(|s| s.name.contains("parse_rust")));
    }

    #[test]
    fn test_register_all() {
        let mut registry = SkillRegistry::new();
        let skills = vec![
            create_test_skill(SkillCategory::new("Syntax"), "skill1", "Rust", vec![]),
            create_test_skill(SkillCategory::new("Semantic"), "skill2", "Rust", vec![]),
        ];

        registry.register_all(skills).unwrap();
        assert_eq!(registry.count(), 2);
    }
}
