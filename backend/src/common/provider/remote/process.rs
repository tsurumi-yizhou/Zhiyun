use crate::common::provider::traits::{ExecuteOptions, ExecuteResult, ExecutionProvider};
use async_trait::async_trait;

pub struct RemoteProcess;

#[async_trait]
impl ExecutionProvider for RemoteProcess {
    async fn execute(
        &self,
        _command: &str,
        _options: ExecuteOptions,
    ) -> anyhow::Result<ExecuteResult> {
        // Mock: 远程执行逻辑（如通过 SSH）
        Ok(ExecuteResult {
            exit_code: 0,
            stdout: "remote output".to_string(),
            stderr: "".to_string(),
        })
    }

    async fn kill(&self, _task_id: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_remote_process_mock() {
        let process = RemoteProcess;
        let result = process
            .execute("ls", ExecuteOptions::default())
            .await
            .unwrap();
        assert_eq!(result.stdout, "remote output");
    }
}
