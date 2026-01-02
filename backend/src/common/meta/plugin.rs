/// 插件基础接口
pub trait Plugin: Send + Sync {
    /// 插件唯一名称
    fn name(&self) -> &str;

    /// 插件版本
    fn version(&self) -> &str;

    /// Mock 实现：获取元数据
    fn mock_metadata(&self) -> String {
        format!("{}:{}", self.name(), self.version())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPlugin;
    impl Plugin for MockPlugin {
        fn name(&self) -> &str {
            "mock-plugin"
        }
        fn version(&self) -> &str {
            "1.0.0"
        }
    }

    #[test]
    fn test_plugin_mock() {
        let plugin = MockPlugin;
        assert_eq!(plugin.mock_metadata(), "mock-plugin:1.0.0");
    }
}
