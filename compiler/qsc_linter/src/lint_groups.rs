use serde::{Deserialize, Serialize};

use crate::{LintKind, LintLevel};

/// End-user configuration for a lint group.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GroupConfig {
    #[serde(rename = "group")]
    /// The lint group.
    pub lint_group: LintGroup,
    /// The group level.
    pub level: LintLevel,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LintGroup {
    Pedantic,
}

impl LintGroup {
    pub fn unfold(self) -> Vec<LintKind> {
        use crate::AstLint::*;
        use crate::HirLint::*;
        match self {
            LintGroup::Pedantic => {
                vec![
                    LintKind::Ast(DivisionByZero),
                    LintKind::Ast(NeedlessParens),
                    LintKind::Ast(RedundantSemicolons),
                    LintKind::Ast(DeprecatedNewtype),
                    LintKind::Ast(DeprecatedSet),
                    LintKind::Ast(DiscourageChainAssignment),
                    LintKind::Hir(NeedlessOperation),
                    LintKind::Hir(DeprecatedFunctionConstructor),
                    LintKind::Hir(DeprecatedWithOperator),
                    LintKind::Hir(DeprecatedDoubleColonOperator),
                ]
            }
        }
    }
}
