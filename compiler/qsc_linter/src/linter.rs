pub mod ast;

use qsc::Span;
use qsc_ast::ast::NodeId;

#[derive(Debug, Default)]
pub struct LintBuffer {
    pub data: Vec<Lint>,
}

impl LintBuffer {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, lint: Lint) {
        self.data.push(lint);
    }
}

#[derive(Debug)]
pub struct Lint {
    pub node_id: NodeId,
    pub span: Span,
    pub message: String,
    pub level: LintLevel,
}

#[derive(Debug)]
pub enum LintLevel {
    Allow,
    Warn,
    ForceWarn,
    Deny,
    ForceDeny,
}
