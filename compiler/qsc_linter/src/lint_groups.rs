// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{LintKind, LintLevel};
use serde::{Deserialize, Serialize};

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
    Deprecations,
}

impl LintGroup {
    pub fn unfold(self) -> Vec<LintKind> {
        use crate::AstLint::*;
        use crate::HirLint::*;
        match self {
            LintGroup::Deprecations => {
                vec![
                    LintKind::Ast(DeprecatedNewtype),
                    LintKind::Ast(DeprecatedSet),
                    LintKind::Hir(DeprecatedFunctionConstructor),
                    LintKind::Hir(DeprecatedWithOperator),
                    LintKind::Hir(DeprecatedDoubleColonOperator),
                ]
            }
        }
    }
}
