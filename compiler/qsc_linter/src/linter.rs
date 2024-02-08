pub(crate) mod ast;
pub(crate) mod hir;

use qsc::Span;
use qsc_ast::ast::NodeId;
use std::fmt::Display;

static mut LINT_BUFFER: Vec<Lint> = Vec::new();

#[must_use]
fn drain() -> std::vec::Drain<'static, Lint> {
    // SAFETY: mutable statics can be mutated by multiple threads,
    // our compiler is single threaded, so this should be fine.
    unsafe { LINT_BUFFER.drain(..) }
}

pub(crate) fn push(lint: Lint) {
    // SAFETY: mutable statics can be mutated by multiple threads,
    // our compiler is single threaded, so this should be fine.
    unsafe { LINT_BUFFER.push(lint) }
}

/// A lint emited by the linter.
#[allow(missing_docs)]
#[derive(Debug)]
pub struct Lint {
    pub node_id: NodeId,
    pub span: Span,
    pub message: &'static str,
    pub level: LintLevel,
}

/// A lint level. This defines if a lint will be treated as a warning or an error,
/// and if the lint level can be overriden by the user.
#[derive(Debug)]
pub enum LintLevel {
    /// The lint is effectively disabled.
    Allow,
    /// The lint will be treated as a warning.
    Warn,
    /// The lint will be treated as a warning and cannot be overriden by the user.
    ForceWarn,
    /// The lint will be treated as an error.
    Deny,
    /// The lint will be treated as an error and cannot be overriden by the user.
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
