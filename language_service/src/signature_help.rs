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
    display::{parse_doc_for_param, parse_doc_for_summary, CodeDisplay},
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
                            self.process_direct_callee(path, args);
                        }
                    }
                }
                _ => walk_expr(self, expr),
            }
        }
    }
}

impl SignatureHelpFinder<'_> {
    fn process_direct_callee(&mut self, callee: &ast::Path, args: &ast::Expr) {
        if let Some(resolve::Res::Item(item_id)) = self.compilation.unit.ast.names.get(callee.id) {
            if let (Some(item), _) = find_item(self.compilation, item_id) {
                if let hir::ItemKind::Callable(callee) = &item.kind {
                    // Check that the callee has parameters to give help for
                    if !matches!(&callee.input.kind, hir::PatKind::Tuple(items) if items.is_empty())
                    {
                        let documentation = parse_doc_for_summary(&item.doc);
                        let documentation = (!documentation.is_empty()).then_some(documentation);

                        let sig_info = SignatureInformation {
                            label: self.display.hir_callable_decl(callee).to_string(),
                            documentation,
                            parameters: self.get_params(callee, &item.doc),
                        };

                        self.signature_help = Some(SignatureHelp {
                            signatures: vec![sig_info],
                            active_signature: 0,
                            active_parameter: process_args(args, self.offset, &callee.input),
                        });
                    }
                }
            }
        }
    }

    /// Takes a callable declaration node and the callable's doc string and
    /// generates the Parameter Information objects for the callable.
    /// Example:
    /// ```qsharp
    /// operation Foo(bar: Int, baz: Double) : Unit {}
    ///               └──┬───┘  └──┬──────┘
    ///               param 1    param 2
    /// ```
    fn get_params(&self, decl: &hir::CallableDecl, doc: &str) -> Vec<ParameterInformation> {
        let mut offset = self.display.get_param_offset(decl);

        match &decl.input.kind {
            hir::PatKind::Discard | hir::PatKind::Bind(_) => {
                self.make_wrapped_params(offset, &decl.input, doc)
            }
            hir::PatKind::Tuple(_) => self.make_param_with_offset(&mut offset, &decl.input, doc),
        }
    }

    /// Callables with a single parameter in their parameter list are special-cased
    /// because we need to re-wrap the parameter into a tuple.
    fn make_wrapped_params(
        &self,
        offset: u32,
        pat: &hir::Pat,
        doc: &str,
    ) -> Vec<ParameterInformation> {
        let documentation = if let hir::PatKind::Bind(name) = &pat.kind {
            let documentation = parse_doc_for_param(doc, &name.name);
            (!documentation.is_empty()).then_some(documentation)
        } else {
            None
        };

        let len = usize_to_u32(self.display.hir_pat(pat).to_string().len());
        let param = ParameterInformation {
            label: Span {
                start: offset + 1,
                end: offset + len + 1,
            },
            documentation,
        };

        let wrapper = ParameterInformation {
            label: Span {
                start: offset,
                end: offset + len + 2,
            },
            documentation: None,
        };

        vec![wrapper, param]
    }

    fn make_param_with_offset(
        &self,
        offset: &mut u32,
        pat: &hir::Pat,
        doc: &str,
    ) -> Vec<ParameterInformation> {
        match &pat.kind {
            hir::PatKind::Bind(_) | hir::PatKind::Discard => {
                let documentation = if let hir::PatKind::Bind(name) = &pat.kind {
                    let documentation = parse_doc_for_param(doc, &name.name);
                    (!documentation.is_empty()).then_some(documentation)
                } else {
                    None
                };

                let len = usize_to_u32(self.display.hir_pat(pat).to_string().len());
                let start = *offset;
                *offset += len;
                vec![ParameterInformation {
                    label: Span {
                        start,
                        end: *offset,
                    },
                    documentation,
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
                    rtrn.extend(self.make_param_with_offset(offset, item, doc));
                }
                *offset += 1; // for the close parenthesis
                rtrn
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

fn usize_to_u32(x: usize) -> u32 {
    u32::try_from(x).expect("failed to cast usize to u32 while generating signature help")
}

fn process_args(args: &ast::Expr, location: u32, params: &hir::Pat) -> u32 {
    fn count_params(params: &hir::Pat) -> i32 {
        match &params.kind {
            hir::PatKind::Bind(_) | hir::PatKind::Discard => 1,
            hir::PatKind::Tuple(items) => items.iter().map(count_params).sum::<i32>() + 1,
        }
    }

    fn try_as_tuple<'a, 'b>(
        args: &'a ast::Expr,
        params: &'b hir::Pat,
        top: bool,
    ) -> Option<(Vec<&'a ast::Expr>, Vec<&'b hir::Pat>)> {
        let args = match &*args.kind {
            ast::ExprKind::Tuple(arg_items) => Some(
                arg_items
                    .iter()
                    .map(std::convert::AsRef::as_ref)
                    .collect::<Vec<_>>(),
            ),
            ast::ExprKind::Paren(arg) => Some(vec![arg.as_ref()]),
            _ => None,
        };

        let params = if let hir::PatKind::Tuple(param_items) = &params.kind {
            Some(param_items.iter().collect::<Vec<_>>())
        } else if top {
            Some(vec![params])
        } else {
            None
        };

        match (args, params) {
            (Some(args), Some(params)) => Some((args, params)),
            _ => None,
        }
    }

    fn increment_until_cursor(
        args: &ast::Expr,
        cursor: u32,
        params: &hir::Pat,
        i: &mut i32,
        top: bool,
    ) {
        if cursor < args.span.lo {
            return;
        }

        if args.span.hi < cursor {
            *i += count_params(params);
            return;
        }

        if let Some((arg_items, param_items)) = try_as_tuple(args, params, top) {
            // check to see if cursor is inside of tuple, past the starting `(` but before the ending `)`
            if args.span.lo < cursor && cursor < args.span.hi {
                let items = zip(&arg_items, &param_items).collect::<Vec<_>>();

                // is the cursor after the last item of a *finished* parameter tuple?
                let is_inside_coda = param_items.len() <= arg_items.len()
                    && match items.last() {
                        Some(last) => last.0.span.hi < cursor,
                        None => true,
                    };

                if !is_inside_coda {
                    *i += 1;
                    for (arg, param) in items {
                        increment_until_cursor(arg, cursor, param, i, false);
                    }
                }
            }
        }
    }

    let mut i = 0;
    increment_until_cursor(args, location, params, &mut i, true);
    i.try_into().expect("got negative param index")
}
