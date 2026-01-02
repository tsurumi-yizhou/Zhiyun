use crate::common::meta::MetaNode;
use anyhow::Result;
use async_trait::async_trait;

/// 语言解析器接口
#[async_trait]
pub trait Parser: Send + Sync {
    /// 适用的语言名称
    fn language(&self) -> &str;

    /// 解析源代码为元 AST
    async fn parse(&self, source: &str) -> Result<MetaNode>;

    /// 加载 SCM 查询文件（用于赋予 Tree-sitter 语义提取能力）
    async fn load_scm(&self, query_name: &str, scm_content: &str) -> Result<()>;
}

/// 解析引擎，管理多个 Parser
pub struct SyntaxEngine {
    // 实际实现中会使用 Registry 查找
}
