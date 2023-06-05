// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::language_service::CompilationState;
use crate::ls_utils::span_contains;
use qsc::SourceMap;
use qsc_hir::hir::{ExprKind, ItemKind, Package, Res};
use qsc_hir::visit::Visitor;

pub struct Definition {
    pub source: String,
    pub offset: u32,
}

pub(crate) fn get_definition(
    compilation_state: &CompilationState,
    _uri: &str,
    offset: u32, // TODO: return a range
) -> Definition {
    let compile_unit = &compilation_state.compile_unit.as_ref().expect(
        "a compilation unit should exist for the current file - has update_code been called?",
    );
    let package = &compile_unit.package;

    let mut definition_finder = DefinitionFinder {
        //std_package: &std_package,
        package,
        source_map: &compile_unit.sources,
        offset,
        definition: None,
    };
    definition_finder.visit_package(package);

    match definition_finder.definition {
        Some((name, offset)) => Definition {
            source: name,
            offset,
        },
        None => Definition {
            source: "".to_string(),
            offset: 0,
        },
    }
}

struct DefinitionFinder<'a> {
    //std_package: &'a Package,
    package: &'a Package,
    source_map: &'a SourceMap,
    offset: u32,
    definition: Option<(String, u32)>,
}

impl<'a> Visitor<'_> for DefinitionFinder<'a> {
    fn visit_expr(&mut self, expr: &qsc_hir::hir::Expr) {
        if span_contains(expr.span, self.offset) {
            if let ExprKind::Var(res) = expr.kind {
                let item = match res {
                    Res::Err => None,
                    // Just one package plus std for now, so let's live with this hack
                    Res::Item(item) => {
                        if item.package.is_none() {
                            self.package.items.get(item.item)
                        } else {
                            // Handling library is tricky for now
                            None
                        }
                    }
                    Res::Local(_node) => None,
                };
                if let Some(def) = item {
                    if let ItemKind::Callable(decl) = &def.kind {
                        self.definition = Some((
                            self.source_map
                                .find_offset(decl.name.span.lo)
                                .name
                                .to_string(),
                            decl.name.span.lo,
                        ))
                    }
                }
            }
        }
        qsc_hir::visit::walk_expr(self, expr);
    }
}
