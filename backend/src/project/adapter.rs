use crate::common::provider::traits::{ExecuteOptions, ExecutionProvider};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// 通用任务接口
#[async_trait]
pub trait BuildSystemAdapter: Send + Sync {
    fn name(&self) -> &str;
    async fn build(&self) -> Result<()>;
    async fn test(&self) -> Result<()>;
    async fn run(&self) -> Result<()>;
}

/// Cargo 适配器
pub struct CargoAdapter {
    executor: Arc<dyn ExecutionProvider>,
    cwd: String,
}

impl CargoAdapter {
    pub fn new(executor: Arc<dyn ExecutionProvider>, cwd: String) -> Self {
        Self { executor, cwd }
    }
}

#[async_trait]
impl BuildSystemAdapter for CargoAdapter {
    fn name(&self) -> &str {
        "Cargo"
    }

    async fn build(&self) -> Result<()> {
        self.executor
            .execute(
                "cargo build",
                ExecuteOptions {
                    cwd: Some(self.cwd.clone()),
                    ..Default::default()
                },
            )
            .await?;
        Ok(())
    }

    async fn test(&self) -> Result<()> {
        self.executor
            .execute(
                "cargo test",
                ExecuteOptions {
                    cwd: Some(self.cwd.clone()),
                    ..Default::default()
                },
            )
            .await?;
        Ok(())
    }

    async fn run(&self) -> Result<()> {
        self.executor
            .execute(
                "cargo run",
                ExecuteOptions {
                    cwd: Some(self.cwd.clone()),
                    ..Default::default()
                },
            )
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::provider::traits::ExecuteResult;

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
    async fn test_cargo_adapter() {
        let executor = Arc::new(MockExecutor);
        let adapter = CargoAdapter::new(executor, ".".to_string());
        assert_eq!(adapter.name(), "Cargo");
        assert!(adapter.build().await.is_ok());
    }
}
