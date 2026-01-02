use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::SystemTime;
use thiserror::Error;

/// SSH 认证方式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SshAuth {
    /// 密码认证
    Password {
        /// 用户名
        username: String,
        /// 密码
        password: String,
    },
    /// 密钥认证
    Key {
        /// 用户名
        username: String,
        /// 私钥路径
        key_path: PathBuf,
        /// 私钥密码（可选）
        passphrase: Option<String>,
    },
    /// 密钥内容认证
    KeyContent {
        /// 用户名
        username: String,
        /// 私钥内容
        key_content: String,
        /// 私钥密码（可选）
        passphrase: Option<String>,
    },
}

/// SSH 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    /// 主机名或 IP 地址
    pub hostname: String,
    /// SSH 端口（默认 22）
    pub port: Option<u16>,
    /// 认证方式
    pub auth: SshAuth,
    /// 工作目录
    pub work_dir: PathBuf,
    /// 连接超时（秒）
    pub connect_timeout: Option<u64>,
}

impl SshConfig {
    /// 创建基于密码认证的 SSH 配置
    pub fn with_password(
        hostname: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
        work_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            hostname: hostname.into(),
            port: None,
            auth: SshAuth::Password {
                username: username.into(),
                password: password.into(),
            },
            work_dir: work_dir.into(),
            connect_timeout: None,
        }
    }

    /// 创建基于密钥路径认证的 SSH 配置
    pub fn with_key_path(
        hostname: impl Into<String>,
        username: impl Into<String>,
        key_path: impl Into<PathBuf>,
        work_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            hostname: hostname.into(),
            port: None,
            auth: SshAuth::Key {
                username: username.into(),
                key_path: key_path.into(),
                passphrase: None,
            },
            work_dir: work_dir.into(),
            connect_timeout: None,
        }
    }

    /// 创建基于密钥内容认证的 SSH 配置
    pub fn with_key_content(
        hostname: impl Into<String>,
        username: impl Into<String>,
        key_content: impl Into<String>,
        work_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            hostname: hostname.into(),
            port: None,
            auth: SshAuth::KeyContent {
                username: username.into(),
                key_content: key_content.into(),
                passphrase: None,
            },
            work_dir: work_dir.into(),
            connect_timeout: None,
        }
    }

    /// 设置端口
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// 设置连接超时
    pub fn with_connect_timeout(mut self, timeout: u64) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// 设置私钥密码
    pub fn with_passphrase(mut self, passphrase: impl Into<String>) -> Self {
        match &mut self.auth {
            SshAuth::Key {
                passphrase: ref mut p,
                ..
            }
            | SshAuth::KeyContent {
                passphrase: ref mut p,
                ..
            } => {
                *p = Some(passphrase.into());
            }
            _ => {}
        }
        self
    }
}

/// Provider 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderConfig {
    /// 本地配置
    Local {
        /// 工作目录
        work_dir: PathBuf,
    },
    /// SSH 远程配置
    Remote {
        /// SSH 配置
        ssh: SshConfig,
    },
}

impl ProviderConfig {
    /// 创建本地配置
    pub fn local(work_dir: impl Into<PathBuf>) -> Self {
        Self::Local {
            work_dir: work_dir.into(),
        }
    }

    /// 创建 SSH 远程配置
    pub fn remote(ssh: SshConfig) -> Self {
        Self::Remote { ssh }
    }

    /// 获取工作目录
    pub fn work_dir(&self) -> &Path {
        match self {
            ProviderConfig::Local { work_dir } => work_dir,
            ProviderConfig::Remote { ssh } => &ssh.work_dir,
        }
    }

    /// 是否为本地配置
    pub fn is_local(&self) -> bool {
        matches!(self, ProviderConfig::Local { .. })
    }

    /// 是否为远程配置
    pub fn is_remote(&self) -> bool {
        matches!(self, ProviderConfig::Remote { .. })
    }
}

/// 文件元数据
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// 文件大小（字节）
    pub size: u64,
    /// 是否为目录
    pub is_dir: bool,
    /// 是否为普通文件
    pub is_file: bool,
    /// 文件权限（Unix 风格）
    pub permissions: Option<u32>,
    /// 最后修改时间
    pub modified: Option<SystemTime>,
    /// 创建时间
    pub created: Option<SystemTime>,
    /// 最后访问时间
    pub accessed: Option<SystemTime>,
}

/// 进程执行结果（一次性返回）
#[derive(Debug, Clone)]
pub struct ProcessOutput {
    /// 标准输出
    pub stdout: Vec<u8>,
    /// 标准错误输出
    pub stderr: Vec<u8>,
    /// 退出码
    pub exit_code: Option<i32>,
}

/// 进程流式输出
#[derive(Debug, Clone)]
pub enum StreamOutput {
    /// 标准输出数据
    Stdout(Vec<u8>),
    /// 标准错误输出数据
    Stderr(Vec<u8>),
    /// 进程退出
    Exit(i32),
}

/// Provider 错误类型
#[derive(Debug, Error)]
pub enum ProviderError {
    /// IO 错误
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// 权限被拒绝
    #[error("Permission denied")]
    PermissionDenied,

    /// 文件或目录不存在
    #[error("Not found: {path}")]
    NotFound { path: String },

    /// 文件或目录已存在
    #[error("Already exists: {path}")]
    AlreadyExists { path: String },

    /// 连接错误（SSH 等）
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// 进程执行错误
    #[error("Process error: {0}")]
    ProcessError(String),

    /// 认证失败
    #[error("Authentication failed")]
    AuthenticationFailed,

    /// 超时错误
    #[error("Operation timed out")]
    Timeout,

    /// 编码错误
    #[error("Encoding error: {0}")]
    EncodingError(String),

    /// 其他错误
    #[error("Other error: {0}")]
    Other(String),
}

/// 文件系统操作接口
#[async_trait]
pub trait FileProvider: Send + Sync {
    /// 读取文件内容
    ///
    /// # 参数
    /// - `path`: 文件路径
    async fn read(&self, path: &Path) -> Result<Vec<u8>, ProviderError>;

    /// 写入文件内容
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `data`: 要写入的数据
    async fn write(&self, path: &Path, data: &[u8]) -> Result<(), ProviderError>;

    /// 创建目录
    ///
    /// # 参数
    /// - `path`: 目录路径
    async fn create_dir(&self, path: &Path) -> Result<(), ProviderError>;

    /// 递归创建目录
    ///
    /// # 参数
    /// - `path`: 目录路径
    async fn create_dir_all(&self, path: &Path) -> Result<(), ProviderError>;

    /// 删除文件或目录
    ///
    /// # 参数
    /// - `path`: 文件或目录路径
    async fn remove(&self, path: &Path) -> Result<(), ProviderError>;

    /// 递归删除目录
    ///
    /// # 参数
    /// - `path`: 目录路径
    async fn remove_dir_all(&self, path: &Path) -> Result<(), ProviderError>;

    /// 获取文件元数据
    ///
    /// # 参数
    /// - `path`: 文件路径
    async fn metadata(&self, path: &Path) -> Result<FileMetadata, ProviderError>;

    /// 列出目录内容
    ///
    /// # 参数
    /// - `path`: 目录路径
    ///
    /// # 返回
    /// 目录中的文件和子目录路径列表
    async fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, ProviderError>;

    /// 检查路径是否存在
    ///
    /// # 参数
    /// - `path`: 文件或目录路径
    async fn exists(&self, path: &Path) -> Result<bool, ProviderError>;

    /// 复制文件
    ///
    /// # 参数
    /// - `from`: 源文件路径
    /// - `to`: 目标文件路径
    async fn copy(&self, from: &Path, to: &Path) -> Result<(), ProviderError>;

    /// 移动/重命名文件
    ///
    /// # 参数
    /// - `from`: 源文件路径
    /// - `to`: 目标文件路径
    async fn rename(&self, from: &Path, to: &Path) -> Result<(), ProviderError>;
}

/// 进程管理接口
#[async_trait]
pub trait ProcessManager: Send + Sync {
    /// 一次性执行命令
    ///
    /// # 参数
    /// - `command`: 命令名称
    /// - `args`: 命令参数
    ///
    /// # 返回
    /// 包含 stdout、stderr 和退出码的结果
    async fn execute(&self, command: &str, args: &[&str]) -> Result<ProcessOutput, ProviderError>;

    /// 流式执行命令
    ///
    /// # 参数
    /// - `command`: 命令名称
    /// - `args`: 命令参数
    ///
    /// # 返回
    /// 异步流，实时输出 stdout、stderr 和退出码
    async fn execute_stream(
        &self,
        command: &str,
        args: &[&str],
    ) -> Result<Pin<Box<dyn futures::Stream<Item = StreamOutput> + Unpin + Send>>, ProviderError>;

    /// 设置工作目录
    ///
    /// # 参数
    /// - `cwd`: 工作目录路径
    fn with_cwd(&mut self, cwd: PathBuf) -> &mut Self;

    /// 设置环境变量
    ///
    /// # 参数
    /// - `key`: 环境变量名
    /// - `value`: 环境变量值
    fn with_env(&mut self, key: String, value: String) -> &mut Self;

    /// 清除所有环境变量
    fn clear_env(&mut self) -> &mut Self;

    /// 设置命令超时时间（秒）
    ///
    /// # 参数
    /// - `timeout`: 超时时间（秒）
    fn with_timeout(&mut self, timeout: u64) -> &mut Self;
}
