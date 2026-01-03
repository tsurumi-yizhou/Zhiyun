use anyhow::Result;

/// 使用 LLM 将用户意图分解为一系列 Skill 调用
pub struct Planner;

impl Default for Planner {
    fn default() -> Self {
        Self::new()
    }
}

impl Planner {
    pub fn new() -> Self {
        Self
    }

    /// 生成计划
    pub async fn plan(&self, _intent: &str) -> Result<Vec<String>> {
        // Mock 逻辑：返回固定步骤
        Ok(vec!["Analyze".to_string(), "Execute".to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_planner() {
        let planner = Planner::new();
        let steps = planner.plan("fix bug").await.unwrap();
        assert_eq!(steps.len(), 2);
    }
}
