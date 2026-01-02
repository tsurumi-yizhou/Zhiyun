use crate::skill::types::{Skill, SkillCategory, SkillError, SkillId};
use std::collections::HashMap;
use std::sync::Arc;

/// Registry for managing and indexing skills
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

    /// Register a new skill, updating all indexes
    pub fn register(&mut self, skill: Skill) -> Result<(), SkillError> {
        skill.validate()?;

        let id = skill.id.clone();
        let skill = Arc::new(skill);

        // Insert into main storage
        self.skills.insert(id.clone(), skill.clone());

        // Update category index
        self.by_category
            .entry(id.category)
            .or_default()
            .push(skill.clone());

        // Update language index
        self.by_language
            .entry(id.language.clone())
            .or_default()
            .push(skill.clone());

        // Update tag index
        for tag in &skill.metadata.tags {
            self.by_tag
                .entry(tag.clone())
                .or_default()
                .push(skill.clone());
        }

        Ok(())
    }

    /// Register multiple skills at once
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

    /// Find relevant skills based on task description
    /// This is a simplified version - in production, use vector embeddings for semantic search
    pub fn find_relevant(
        &self,
        task: &str,
        language: Option<&str>,
        limit: usize,
    ) -> Vec<Arc<Skill>> {
        let mut candidates: Vec<Arc<Skill>> = Vec::new();
        let task_lower = task.to_lowercase();

        // If language specified, start with language-specific skills
        if let Some(lang) = language {
            candidates.extend(self.by_language(lang));
        } else {
            // Otherwise consider all skills
            candidates.extend(self.skills.values().cloned());
        }

        // Simple relevance scoring based on keyword matching
        let mut scored: Vec<_> = candidates
            .into_iter()
            .map(|skill| {
                let score = calculate_relevance(&skill, &task_lower);
                (score, skill)
            })
            .filter(|(score, _)| *score > 0)
            .collect();

        // Sort by relevance score descending
        scored.sort_by(|a, b| b.0.cmp(&a.0));

        // Take top N
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

/// Calculate a simple relevance score for a skill against a task
fn calculate_relevance(skill: &Skill, task: &str) -> usize {
    let mut score = 0;

    // Check name match
    if skill.name.to_lowercase().contains(task) {
        score += 10;
    }

    // Check description match
    if skill.description.to_lowercase().contains(task) {
        score += 5;
    }

    // Check content match
    if skill.content.to_lowercase().contains(task) {
        score += 3;
    }

    // Check tags match
    for tag in &skill.metadata.tags {
        if tag.to_lowercase().contains(task) {
            score += 7;
        }
    }

    // Check related tools
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
    use crate::skill::types::SkillMetadata;

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

        // Check category index
        let by_cat = registry.by_category(SkillCategory::new("Syntax"));
        assert_eq!(by_cat.len(), 1);
        assert_eq!(by_cat[0].name, "parse_macro");

        // Check language index
        let by_lang = registry.by_language("Rust");
        assert_eq!(by_lang.len(), 1);

        // Check tag index
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

        // Find Rust parsing skills
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
