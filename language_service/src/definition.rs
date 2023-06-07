// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::qsc_utils::{span_contains, Compilation};
use qsc::hir::{visit::Visitor, ExprKind, ItemKind, Package, Res};
use qsc::SourceMap;

#[derive(Debug, PartialEq)]
pub struct Definition {
    pub source: String,
    pub offset: u32,
}

pub(crate) fn get_definition(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> Option<Definition> {
    let compile_unit = &compilation.compile_unit;
    // Map the file offset into a SourceMap offset
    let offset = compile_unit.sources.map_offset(source_name, offset);
    let package = &compile_unit.package;

    let mut definition_finder = DefinitionFinder {
        package,
        source_map: &compile_unit.sources,
        offset,
        definition: None,
    };
    definition_finder.visit_package(package);

    definition_finder
        .definition
        .map(|(name, offset)| Definition {
            source: name,
            offset,
        })
}

struct DefinitionFinder<'a> {
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
                            // Handling std library is tricky for now
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
                        ));
                    }
                }
            }
        }
        qsc_hir::visit::walk_expr(self, expr);
    }
}
