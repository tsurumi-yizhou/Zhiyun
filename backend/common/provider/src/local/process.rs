use crate::traits::{ProcessManager, ProcessOutput, ProviderError, StreamOutput};
use async_trait::async_trait;
use futures::stream::Stream;
use std::collections::HashMap;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::Receiver;
use tokio::time::timeout;

/// 本地进程管理器
///
/// 基于 `tokio::process` 实现异步进程管理
#[derive(Debug, Clone)]
pub struct LocalProcessManager {
    /// 工作目录
    cwd: Option<PathBuf>,
    /// 环境变量
    env: HashMap<String, String>,
    /// 清除环境变量标志
    clear_env: bool,
    /// 命令超时时间（秒）
    command_timeout: Option<u64>,
}

impl LocalProcessManager {
    /// 创建新的本地进程管理器
    pub fn new() -> Self {
        Self {
            cwd: None,
            env: HashMap::new(),
            clear_env: false,
            command_timeout: None,
        }
    }

    /// 构建命令
    fn build_command(&self, program: &str, args: &[&str]) -> Command {
        let mut cmd = Command::new(program);

        // 设置参数
        cmd.args(args);

        // 设置工作目录
        if let Some(ref cwd) = self.cwd {
            cmd.current_dir(cwd);
        }

        // 设置环境变量
        if self.clear_env {
            cmd.env_clear();
        }
        for (key, value) in &self.env {
            cmd.env(key, value);
        }

        cmd
    }
}

impl Default for LocalProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProcessManager for LocalProcessManager {
    async fn execute(&self, command: &str, args: &[&str]) -> Result<ProcessOutput, ProviderError> {
        let mut cmd = self.build_command(command, args);

        // 执行命令
        let output = if let Some(timeout_secs) = self.command_timeout {
            let duration = Duration::from_secs(timeout_secs);
            timeout(duration, cmd.output())
                .await
                .map_err(|_| ProviderError::Timeout)?
                .map_err(|e| ProviderError::ProcessError(format!("Failed to execute: {}", e)))?
        } else {
            cmd.output()
                .await
                .map_err(|e| ProviderError::ProcessError(format!("Failed to execute: {}", e)))?
        };

        Ok(ProcessOutput {
            stdout: output.stdout,
            stderr: output.stderr,
            exit_code: output.status.code(),
        })
    }

    async fn execute_stream(
        &self,
        command: &str,
        args: &[&str],
    ) -> Result<Pin<Box<dyn Stream<Item = StreamOutput> + Send + Unpin>>, ProviderError> {
        let mut cmd = self.build_command(command, args);

        // 创建 stdout 和 stderr 的流
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| ProviderError::ProcessError(format!("Failed to spawn: {}", e)))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ProviderError::ProcessError("Failed to capture stdout".to_string()))?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| ProviderError::ProcessError("Failed to capture stderr".to_string()))?;

        // 创建一个 Channel 来传递输出
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        // 启动任务来读取输出
        tokio::spawn(async move {
            let mut stdout_reader = BufReader::new(stdout);
            let mut stderr_reader = BufReader::new(stderr);
            let mut stdout_buf = vec![0; 8192];
            let mut stderr_buf = vec![0; 8192];
            let mut stdout_done = false;
            let mut stderr_done = false;

            loop {
                tokio::select! {
                    // 读取 stdout
                    result = async {
                        if !stdout_done {
                            stdout_reader.read(&mut stdout_buf).await
                        } else {
                            Ok(0)
                        }
                    }, if !stdout_done => {
                        match result {
                            Ok(0) => stdout_done = true,
                            Ok(n) => {
                                stdout_buf.truncate(n);
                                let _ = tx.send(StreamOutput::Stdout(stdout_buf.clone())).await;
                                stdout_buf = vec![0; 8192];
                            }
                            Err(_) => stdout_done = true,
                        }
                    }
                    // 读取 stderr
                    result = async {
                        if !stderr_done {
                            stderr_reader.read(&mut stderr_buf).await
                        } else {
                            Ok(0)
                        }
                    }, if !stderr_done => {
                        match result {
                            Ok(0) => stderr_done = true,
                            Ok(n) => {
                                stderr_buf.truncate(n);
                                let _ = tx.send(StreamOutput::Stderr(stderr_buf.clone())).await;
                                stderr_buf = vec![0; 8192];
                            }
                            Err(_) => stderr_done = true,
                        }
                    }
                    // 两者都完成，退出
                    else => break,
                }
            }

            // 等待进程结束
            let exit_status = child.wait().await.ok();
            let exit_code = exit_status.and_then(|s| s.code()).unwrap_or(-1);
            let _ = tx.send(StreamOutput::Exit(exit_code)).await;
        });

        // 使用 UnpinStream 包装器
        struct ReceiverStream {
            rx: Receiver<StreamOutput>,
        }

        impl Stream for ReceiverStream {
            type Item = StreamOutput;

            fn poll_next(
                mut self: Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> Poll<Option<Self::Item>> {
                self.rx.poll_recv(cx)
            }
        }

        impl Unpin for ReceiverStream {}

        let output_stream = ReceiverStream { rx };
        Ok(Box::pin(output_stream))
    }

    fn with_cwd(&mut self, cwd: PathBuf) -> &mut Self {
        self.cwd = Some(cwd);
        self
    }

    fn with_env(&mut self, key: String, value: String) -> &mut Self {
        self.env.insert(key, value);
        self
    }

    fn clear_env(&mut self) -> &mut Self {
        self.clear_env = true;
        self
    }

    fn with_timeout(&mut self, timeout: u64) -> &mut Self {
        self.command_timeout = Some(timeout);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_execute_echo() {
        let manager = LocalProcessManager::new();
        let output = manager.execute("echo", &["hello", "world"]).await.unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("hello"));
        assert!(stdout.contains("world"));
        assert_eq!(output.exit_code, Some(0));
    }

    #[tokio::test]
    async fn test_execute_with_error() {
        let manager = LocalProcessManager::new();
        let output = manager.execute("ls", &["/nonexistent"]).await.unwrap();

        // 应该有错误输出
        assert!(!output.stderr.is_empty() || output.exit_code != Some(0));
    }

    #[tokio::test]
    async fn test_execute_with_cwd() {
        let mut manager = LocalProcessManager::new();
        manager.with_cwd(PathBuf::from("/tmp"));

        let output = manager.execute("pwd", &[]).await.unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // macOS 上 /tmp 是符号链接指向 /private/tmp
        assert!(stdout == "/tmp" || stdout == "/private/tmp");
    }

    #[tokio::test]
    async fn test_execute_with_env() {
        let mut manager = LocalProcessManager::new();
        manager.with_env("TEST_VAR".to_string(), "test_value".to_string());

        let output = manager
            .execute("sh", &["-c", "echo $TEST_VAR"])
            .await
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

        assert_eq!(stdout, "test_value");
    }

    #[tokio::test]
    async fn test_execute_stream() {
        let manager = LocalProcessManager::new();

        let mut stream = manager
            .execute_stream("echo", &["streaming", "test"])
            .await
            .unwrap();

        let mut outputs = Vec::new();
        while let Some(output) = stream.next().await {
            outputs.push(output);
        }

        // 检查是否有输出
        assert!(!outputs.is_empty());
        // 最后一个应该是 Exit
        if let Some(StreamOutput::Exit(code)) = outputs.last() {
            assert_eq!(*code, 0);
        } else {
            panic!("Expected Exit as last output");
        }
    }

    #[tokio::test]
    async fn test_timeout() {
        let mut manager = LocalProcessManager::new();
        manager.with_timeout(1); // 1秒超时

        // 在 Linux/Mac 上使用 sleep，Windows 上使用 timeout
        #[cfg(target_os = "windows")]
        let result = manager.execute("timeout", &["10"]).await;

        #[cfg(not(target_os = "windows"))]
        let result = manager.execute("sleep", &["10"]).await;

        assert!(matches!(result, Err(ProviderError::Timeout)));
    }
}
