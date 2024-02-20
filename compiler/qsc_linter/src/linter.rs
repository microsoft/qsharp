pub(crate) mod ast;
pub(crate) mod hir;

use qsc_ast::ast::NodeId;
use qsc_data_structures::span::Span;
use std::fmt::Display;

/// A lint emited by the linter.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Lint {
    pub node_id: NodeId,
    pub span: Span,
    pub message: &'static str,
    pub level: LintLevel,
}

/// A lint level. This defines if a lint will be treated as a warning or an error,
/// and if the lint level can be overriden by the user.
#[derive(Debug, Clone)]
pub enum LintLevel {
    /// The lint is effectively disabled.
    Allow,
    /// The lint will be treated as a warning.
    Warning,
    /// The lint will be treated as a warning and cannot be overriden by the user.
    ForceWarning,
    /// The lint will be treated as an error.
    Error,
    /// The lint will be treated as an error and cannot be overriden by the user.
    ForceError,
}

impl Display for LintLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = match self {
            LintLevel::Allow => "",
            LintLevel::Warning | LintLevel::ForceWarning => "warning",
            LintLevel::Error | LintLevel::ForceError => "error",
        };

        write!(f, "{level}")
    }
}
