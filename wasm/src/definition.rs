// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::language_service_wasm::Definition;
use crate::ls_utils::{get_compilation, span_contains};
use qsc::SourceMap;
use qsc_hir::hir::{ExprKind, ItemKind, Package, Res};
use qsc_hir::visit::Visitor;
use wasm_bindgen::prelude::*;

pub(crate) fn get_definition(
    source_path: &str,
    code: &str,
    offset: u32, // TODO: return a range
) -> Result<JsValue, JsValue> {
    let (_, package, source_map, _, _) = get_compilation(source_path, code);

    let mut definition_finder = DefinitionFinder {
        //std_package: &std_package,
        package: &package,
        source_map: &source_map,
        offset,
        definition: None,
    };
    definition_finder.visit_package(&package);

    let definition = match definition_finder.definition {
        Some((name, offset)) => Definition {
            source: name,
            offset,
        },
        None => Definition {
            source: "".to_string(),
            offset: 0,
        },
    };
    Ok(serde_wasm_bindgen::to_value(&definition)?)
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
