use serde::{Deserialize, Serialize};

/// 统一不同编译器的诊断格式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Diagnostic {
    pub message: String,
    pub severity: Severity,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

pub struct DiagnosticManager {
    diagnostics: Vec<Diagnostic>,
}

impl Default for DiagnosticManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticManager {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    /// 添加诊断信息
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// 获取所有诊断信息
    pub fn get_diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// 清除诊断信息
    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_manager() {
        let mut manager = DiagnosticManager::new();
        manager.add_diagnostic(Diagnostic {
            message: "error message".to_string(),
            severity: Severity::Error,
            line: 1,
            column: 1,
        });
        assert_eq!(manager.get_diagnostics().len(), 1);
        manager.clear();
        assert_eq!(manager.get_diagnostics().len(), 0);
    }
}
