use crate::common::provider::traits::{ExecuteOptions, ExecuteResult, ExecutionProvider};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;

pub struct LocalProcess;

#[async_trait]
impl ExecutionProvider for LocalProcess {
    async fn execute(
        &self,
        command: &str,
        options: ExecuteOptions,
    ) -> anyhow::Result<ExecuteResult> {
        let mut parts = command.split_whitespace();
        let program = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("Empty command"))?;
        let args: Vec<&str> = parts.collect();

        let mut cmd = Command::new(program);
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

        if let Some(cwd) = options.cwd {
            cmd.current_dir(cwd);
        }

        for (key, value) in options.env {
            cmd.env(key, value);
        }

        let output = cmd.output().await?;

        Ok(ExecuteResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }

    async fn kill(&self, _task_id: &str) -> anyhow::Result<()> {
        // 在本地进程实现中，kill 通常需要更复杂的任务追踪
        // 目前先做简单的 Mock
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_local_process_execution() {
        let process = LocalProcess;
        let options = ExecuteOptions::default();

        // 使用 echo 测试 (跨平台性较好)
        #[cfg(windows)]
        let cmd = "cmd /c echo hello";
        #[cfg(not(windows))]
        let cmd = "echo hello";

        let result = process.execute(cmd, options).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.trim().contains("hello"));
    }

    #[tokio::test]
    async fn test_local_process_env() {
        let process = LocalProcess;
        let mut env = HashMap::new();
        env.insert("TEST_VAR".to_string(), "test_value".to_string());

        let options = ExecuteOptions {
            env,
            ..Default::default()
        };

        #[cfg(windows)]
        let cmd = "cmd /c echo %TEST_VAR%";
        #[cfg(not(windows))]
        let cmd = "env";

        let result = process.execute(cmd, options).await.unwrap();
        assert!(result.stdout.contains("TEST_VAR=test_value"));
    }
}
