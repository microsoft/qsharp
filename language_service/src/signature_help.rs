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
    protocol::{
        ParameterInformation, SignatureHelp, SignatureHelpContext, SignatureInformation, Span,
    },
    qsc_utils::{find_item, map_offset, span_contains, Compilation},
};

pub(crate) fn get_signature_help(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
    context: SignatureHelpContext,
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

    finder.signature_help
}

struct SignatureHelpFinder<'a> {
    compilation: &'a Compilation,
    offset: u32,
    signature_help: Option<SignatureHelp>,
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
                ast::ExprKind::Call(callee, args) => {
                    let callee = unwrap_parens(callee);
                    if let ast::ExprKind::Path(path) = &*callee.kind {
                        if let Some(resolve::Res::Item(item_id)) =
                            self.compilation.unit.ast.names.get(path.id)
                        {
                            if let (Some(item), _) = find_item(self.compilation, item_id) {
                                if let qsc::hir::ItemKind::Callable(callee) = &item.kind {
                                    // Check that the callee has parameters to give help for
                                    if !matches!(&callee.input.kind, hir::PatKind::Tuple(items) if items.is_empty())
                                    {
                                        // Get active parameter
                                        if let Some(active_parameter) =
                                            process_args(args, self.offset)
                                        {
                                            // Get signature information
                                            let sig_info = SignatureInformation {
                                                label: self
                                                    .display
                                                    .hir_callable_decl(callee)
                                                    .to_string(),
                                                documentation: None,
                                                parameters: get_params(
                                                    callee.span.lo,
                                                    &callee.input,
                                                ),
                                            };

                                            self.signature_help = Some(SignatureHelp {
                                                signatures: vec![sig_info],
                                                active_signature: 0,
                                                active_parameter,
                                            });
                                        }
                                    }
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

fn process_args(args: &ast::Expr, location: u32) -> Option<u32> {
    match &*args.kind {
        ast::ExprKind::Tuple(items) => {
            // if items.is_empty() {
            //     Some(0)
            // } else {
            //     items.iter().enumerate().find_map(|(i, arg)| {
            //         if span_contains(arg.span, location) {
            //             Some(u32::try_from(i).expect("failed to cast usize to u32 for parameter index while generating signature help"))
            //         } else {
            //             None
            //         }
            //     })
            // }
            let i = u32::try_from(items.len()).expect(
                "failed to cast usize to u32 for parameter index while generating signature help",
            );
            Some(i)
        }
        _ => None,
    }
}
