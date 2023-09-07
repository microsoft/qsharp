// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use qsc::{
    ast::{
        self,
        visit::{walk_expr, walk_item, Visitor},
    },
    hir, resolve,
};

use crate::{
    display::CodeDisplay,
    protocol::{ParameterInformation, SignatureHelp, SignatureInformation, Span},
    qsc_utils::{find_item, map_offset, span_contains, Compilation},
};

pub(crate) fn get_signature_help(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> Option<SignatureHelp> {
    // Map the file offset into a SourceMap offset
    let offset = map_offset(&compilation.unit.sources, source_name, offset);
    let package = &compilation.unit.ast.package;

    let mut finder = SignatureHelpFinder {
        compilation,
        offset,
        signature_help: None,
        display: CodeDisplay { compilation },
    };

    finder.visit_package(package);

    finder.signature_help.map(|signature| SignatureHelp {
        signatures: vec![signature],
        active_signature: 0,
        active_parameter: 0,
    })
}

struct SignatureHelpFinder<'a> {
    compilation: &'a Compilation,
    offset: u32,
    signature_help: Option<SignatureInformation>,
    display: CodeDisplay<'a>,
}

impl<'a> Visitor<'a> for SignatureHelpFinder<'a> {
    fn visit_item(&mut self, item: &'a ast::Item) {
        if span_contains(item.span, self.offset) {
            walk_item(self, item);
        }
    }

    fn visit_expr(&mut self, expr: &'a ast::Expr) {
        if span_contains(expr.span, self.offset) {
            match &*expr.kind {
                ast::ExprKind::Call(callee, _) => {
                    let callee = unwrap_parens(callee);
                    if let ast::ExprKind::Path(path) = &*callee.kind {
                        if let Some(resolve::Res::Item(item_id)) =
                            self.compilation.unit.ast.names.get(path.id)
                        {
                            if let (Some(item), _) = find_item(self.compilation, item_id) {
                                if let qsc::hir::ItemKind::Callable(callee) = &item.kind {
                                    self.signature_help = Some(SignatureInformation {
                                        label: self.display.hir_callable_decl(callee).to_string(),
                                        documentation: None,
                                        parameters: get_params(callee.span.lo, &callee.input),
                                    });
                                }
                            }
                        }
                    }
                }
                _ => walk_expr(self, expr),
            }
        }
    }
}

fn unwrap_parens(expr: &ast::Expr) -> &ast::Expr {
    match &*expr.kind {
        ast::ExprKind::Paren(inner) => unwrap_parens(inner),
        _ => expr,
    }
}

fn get_params(offset: u32, input: &hir::Pat) -> Vec<ParameterInformation> {
    fn populate_params(offset: u32, input: &hir::Pat, params: &mut Vec<ParameterInformation>) {
        match &input.kind {
            hir::PatKind::Bind(_) => params.push(ParameterInformation {
                label: Span {
                    start: input.span.lo - offset,
                    end: input.span.hi - offset,
                },
                documentation: None,
            }),
            hir::PatKind::Discard => todo!(),
            hir::PatKind::Tuple(items) => items
                .iter()
                .for_each(|item| populate_params(offset, item, params)),
        }
    }

    let mut params: Vec<ParameterInformation> = vec![];
    populate_params(offset, input, &mut params);
    params
}
