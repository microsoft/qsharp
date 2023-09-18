// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use std::iter::zip;

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
                ast::ExprKind::Call(callee, args) if span_contains(args.span, self.offset) => {
                    walk_expr(self, args);
                    if self.signature_help.is_none() {
                        let callee = unwrap_parens(callee);
                        if let ast::ExprKind::Path(path) = &*callee.kind {
                            if let Some(resolve::Res::Item(item_id)) =
                                self.compilation.unit.ast.names.get(path.id)
                            {
                                if let (Some(item), _) = find_item(self.compilation, item_id) {
                                    if let hir::ItemKind::Callable(callee) = &item.kind {
                                        // Check that the callee has parameters to give help for
                                        if !matches!(&callee.input.kind, hir::PatKind::Tuple(items) if items.is_empty())
                                        {
                                            let sig_info = SignatureInformation {
                                                label: self
                                                    .display
                                                    .hir_callable_decl(callee)
                                                    .to_string(),
                                                documentation: None,
                                                parameters: self.get_params(callee),
                                            };

                                            self.signature_help = Some(SignatureHelp {
                                                signatures: vec![sig_info],
                                                active_signature: 0,
                                                //active_parameter: process_args(args, self.offset),
                                                active_parameter: process_args(
                                                    args,
                                                    self.offset,
                                                    &callee.input,
                                                ),
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

impl SignatureHelpFinder<'_> {
    /// Takes a callable declaration node an generates the Parameter Information objects for it.
    /// Example:
    /// ```qsharp
    /// operation Foo(bar: Int, baz: Double) : Unit {}
    ///               └──┬───┘  └──┬──────┘
    ///               param 1    param 2
    /// ```
    fn get_params(&self, decl: &hir::CallableDecl) -> Vec<ParameterInformation> {
        let mut offset = self.display.get_param_offset(decl);

        self.make_param_with_offset(&mut offset, &decl.input)
    }

    fn make_param_with_offset(
        &self,
        offset: &mut u32,
        pat: &hir::Pat,
    ) -> Vec<ParameterInformation> {
        match &pat.kind {
            hir::PatKind::Discard | hir::PatKind::Bind(_) => {
                let len = usize_to_u32(self.display.hir_pat(pat).to_string().len());
                let start = *offset;
                *offset += len;
                vec![ParameterInformation {
                    label: Span {
                        start,
                        end: *offset,
                    },
                    documentation: None,
                }]
            }
            hir::PatKind::Tuple(items) => {
                let len = usize_to_u32(self.display.hir_pat(pat).to_string().len());
                let mut rtrn = vec![ParameterInformation {
                    label: Span {
                        start: *offset,
                        end: *offset + len,
                    },
                    documentation: None,
                }];
                *offset += 1; // for the open parenthesis
                let mut is_first = true;
                for item in items {
                    if is_first {
                        is_first = false;
                    } else {
                        *offset += 2; // 2 for the comma and space
                    }
                    rtrn.extend(self.make_param_with_offset(offset, item));
                }
                *offset += 1; // for the close parenthesis
                rtrn
            }
        }
    }
}

fn usize_to_u32(x: usize) -> u32 {
    u32::try_from(x).expect("failed to cast usize to u32 while generating signature help")
}

fn process_args(args: &ast::Expr, location: u32, params: &hir::Pat) -> u32 {
    fn is_tuple<'a, 'b>(
        args: &'a ast::Expr,
        params: &'b hir::Pat,
    ) -> Option<(Vec<&'a ast::Expr>, &'b Vec<hir::Pat>)> {
        match (&*args.kind, &params.kind) {
            (ast::ExprKind::Tuple(arg_items), hir::PatKind::Tuple(param_items)) => {
                let temp: Vec<&ast::Expr> = arg_items.iter().map(|i| &**i).collect();
                Some((temp, param_items))
            }
            (_, hir::PatKind::Tuple(param_items)) => Some((vec![args], param_items)),
            _ => None,
        }
    }

    fn increment_until_cursor(args: &ast::Expr, cursor: u32, params: &hir::Pat, i: &mut i32) {
        if let Some((arg_items, param_items)) = is_tuple(args, params) {
            let items: Vec<(&&ast::Expr, &hir::Pat)> = zip(&arg_items, param_items).collect();

            if let Some(last) = items.last() {
                // Case Tuple, cursor after last elem but before end:
                // - if params > args: increment, recurse over elems
                // Case Tuple, cursor *after* starting '(': increment, recurse over elems
                if (last.0.span.hi < cursor
                    && cursor < args.span.hi
                    && param_items.len() > arg_items.len())
                    || args.span.lo < cursor
                {
                    *i += 1;
                    for (arg, param) in items {
                        increment_until_cursor(arg, cursor, param, i);
                    }
                }
                // Case Tuple, cursor before or *at* starting '(': do nothing
            } else if args.span.lo < cursor {
                // Case Empty Tuple, cursor after starting `(`: increment
                *i += 1;
            }
        } else if args.span.hi < cursor {
            // Case Non-Tuple, cursor after: increment
            *i += 1;
        }
        // Case Non-Tuple, cursor before end (cursor inside): do nothing
    }

    let mut i = 0;
    increment_until_cursor(args, location, params, &mut i);
    i.try_into().expect("")
}
