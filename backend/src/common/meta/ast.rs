use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// 语言无关的元 AST 节点
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MetaNode {
    /// 模块或文件根
    Module {
        #[serde(default = "Uuid::new_v4")]
        id: Uuid,
        name: String,
        children: Vec<MetaNode>,
        #[serde(flatten)]
        metadata: HashMap<String, Value>,
    },
    /// 函数或方法定义
    Function {
        #[serde(default = "Uuid::new_v4")]
        id: Uuid,
        name: String,
        params: Vec<MetaNode>,
        body: Option<Box<MetaNode>>,
        #[serde(flatten)]
        metadata: HashMap<String, Value>,
    },
    /// 类、结构体或接口定义
    Class {
        #[serde(default = "Uuid::new_v4")]
        id: Uuid,
        name: String,
        members: Vec<MetaNode>,
        bases: Vec<String>,
        #[serde(flatten)]
        metadata: HashMap<String, Value>,
    },
    /// 变量或常量声明
    Declaration {
        #[serde(default = "Uuid::new_v4")]
        id: Uuid,
        name: String,
        kind: String, // e.g., "let", "const", "var"
        value: Option<Box<MetaNode>>,
        #[serde(flatten)]
        metadata: HashMap<String, Value>,
    },
    /// 赋值操作
    Assignment {
        #[serde(default = "Uuid::new_v4")]
        id: Uuid,
        target: Box<MetaNode>,
        value: Box<MetaNode>,
    },
    /// 函数或方法调用
    Call {
        #[serde(default = "Uuid::new_v4")]
        id: Uuid,
        callee: Box<MetaNode>,
        args: Vec<MetaNode>,
    },
    /// 标识符
    Identifier {
        #[serde(default = "Uuid::new_v4")]
        id: Uuid,
        name: String,
        #[serde(default)]
        scope_id: Option<String>,
    },
    /// 字面量
    Literal {
        #[serde(default = "Uuid::new_v4")]
        id: Uuid,
        value: Value,
    },
    /// 块级作用域
    Block {
        #[serde(default = "Uuid::new_v4")]
        id: Uuid,
        statements: Vec<MetaNode>,
    },
    /// 语言特定的扩展节点
    Extension {
        #[serde(default = "Uuid::new_v4")]
        id: Uuid,
        language: String,
        kind: String,
        data: Value,
    },
}

impl MetaNode {
    /// 获取节点 ID
    pub fn id(&self) -> Uuid {
        match self {
            MetaNode::Module { id, .. } => *id,
            MetaNode::Function { id, .. } => *id,
            MetaNode::Class { id, .. } => *id,
            MetaNode::Declaration { id, .. } => *id,
            MetaNode::Assignment { id, .. } => *id,
            MetaNode::Call { id, .. } => *id,
            MetaNode::Identifier { id, .. } => *id,
            MetaNode::Literal { id, .. } => *id,
            MetaNode::Block { id, .. } => *id,
            MetaNode::Extension { id, .. } => *id,
        }
    }

    /// 创建一个新的标识符节点
    pub fn identifier(name: &str) -> Self {
        MetaNode::Identifier {
            id: Uuid::new_v4(),
            name: name.to_string(),
            scope_id: None,
        }
    }

    /// 创建一个空的模块节点
    pub fn module(name: &str) -> Self {
        MetaNode::Module {
            id: Uuid::new_v4(),
            name: name.to_string(),
            children: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_creation() {
        let node = MetaNode::identifier("foo");
        let id = node.id();
        if let MetaNode::Identifier {
            name, id: node_id, ..
        } = node
        {
            assert_eq!(name, "foo");
            assert_eq!(id, node_id);
        } else {
            panic!("Expected Identifier node");
        }
    }
}
