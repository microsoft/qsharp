pub mod ast;
use qsc::Span;
use qsc_ast::ast::NodeId;
use std::fmt::Display;

static mut LINT_BUFFER: Vec<Lint> = Vec::new();

#[must_use]
pub fn drain() -> std::vec::Drain<'static, Lint> {
    // SAFETY: mutable statics can be mutated by multiple threads,
    // our compiler is single threaded, so this should be fine.
    unsafe { LINT_BUFFER.drain(..) }
}

pub fn push(lint: Lint) {
    // SAFETY: mutable statics can be mutated by multiple threads,
    // our compiler is single threaded, so this should be fine.
    unsafe { LINT_BUFFER.push(lint) }
}

#[derive(Debug)]
pub struct Lint {
    pub node_id: NodeId,
    pub span: Span,
    pub message: &'static str,
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

impl Display for LintLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = match self {
            LintLevel::Allow => "",
            LintLevel::Warn | LintLevel::ForceWarn => "warning",
            LintLevel::Deny | LintLevel::ForceDeny => "error",
        };

        write!(f, "{level}")
    }
}
