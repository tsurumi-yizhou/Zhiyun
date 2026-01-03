use crate::common::provider::traits::{ExecuteOptions, ExecutionProvider};
use crate::compiler::diagnostic::Diagnostic;
use anyhow::Result;
use std::sync::Arc;

/// 触发项目级的全量或增量检查
pub struct ProjectAnalyzer {
    executor: Arc<dyn ExecutionProvider>,
}

impl ProjectAnalyzer {
    pub fn new(executor: Arc<dyn ExecutionProvider>) -> Self {
        Self { executor }
    }

    /// 运行分析
    pub async fn analyze(&self, project_path: &str) -> Result<Vec<Diagnostic>> {
        // 通过 provider 执行编译/检查命令，屏蔽平台细节
        let _result = self
            .executor
            .execute(
                "cargo check",
                ExecuteOptions {
                    cwd: Some(project_path.to_string()),
                    ..Default::default()
                },
            )
            .await?;

        // Mock 逻辑：解析 _result 并返回诊断列表
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::provider::traits::ExecuteResult;
    use async_trait::async_trait;

    struct MockExecutor;
    #[async_trait]
    impl ExecutionProvider for MockExecutor {
        async fn execute(&self, _cmd: &str, _opts: ExecuteOptions) -> Result<ExecuteResult> {
            Ok(ExecuteResult {
                exit_code: 0,
                stdout: "".to_string(),
                stderr: "".to_string(),
            })
        }
        async fn kill(&self, _id: &str) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_project_analyzer() {
        let executor = Arc::new(MockExecutor);
        let analyzer = ProjectAnalyzer::new(executor);
        let results = analyzer.analyze(".").await.unwrap();
        assert!(results.is_empty());
    }
}
