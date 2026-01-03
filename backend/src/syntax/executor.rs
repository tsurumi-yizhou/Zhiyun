use crate::common::meta::MetaNode;
use crate::syntax::engine::interface::Parser;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

/// 负责调度注册的解析器插件
pub struct ParserExecutor {
    parsers: HashMap<String, Arc<dyn Parser>>,
}

impl Default for ParserExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserExecutor {
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
        }
    }

    /// 注册解析器
    pub fn register_parser(&mut self, language: &str, parser: Arc<dyn Parser>) {
        self.parsers.insert(language.to_string(), parser);
    }

    /// 执行解析
    pub async fn parse(&self, language: &str, source: &str) -> Result<MetaNode> {
        let parser = self
            .parsers
            .get(language)
            .ok_or_else(|| anyhow::anyhow!("No parser found for language: {}", language))?;

        parser.parse(source).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockParser;
    #[async_trait]
    impl Parser for MockParser {
        fn language(&self) -> &str {
            "mock"
        }
        async fn parse(&self, source: &str) -> Result<MetaNode> {
            Ok(MetaNode::module(source))
        }
        async fn load_scm(&self, _name: &str, _content: &str) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_parser_executor() {
        let mut executor = ParserExecutor::new();
        executor.register_parser("mock", Arc::new(MockParser));

        let node = executor.parse("mock", "test_file").await.unwrap();
        if let MetaNode::Module { name, .. } = node {
            assert_eq!(name, "test_file");
        } else {
            panic!("Expected Module node");
        }
    }
}
