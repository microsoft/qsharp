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
                                    if let qsc::hir::ItemKind::Callable(callee) = &item.kind {
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
                                                active_parameter: process_args(args, self.offset),
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
    /// ```
    /// operation Foo(bar: Int, baz: Double) : Unit {}
    ///               └──┬───┘  └──┬──────┘
    ///               param 1    param 2
    /// ```
    fn get_params(&self, decl: &hir::CallableDecl) -> Vec<ParameterInformation> {
        let offset = self.display.get_param_offset(decl);

        match &decl.input.kind {
            hir::PatKind::Discard | hir::PatKind::Bind(_) => {
                vec![self.make_param_with_offset(offset, &decl.input)]
            }
            hir::PatKind::Tuple(items) => {
                let mut cumulative_offset = offset;
                items
                    .iter()
                    .map(|item| {
                        let info = self.make_param_with_offset(cumulative_offset, item);
                        cumulative_offset = info.label.end + 2; // 2 for the comma and space
                        info
                    })
                    .collect()
            }
        }
    }

    fn make_param_with_offset(&self, offset: u32, pat: &hir::Pat) -> ParameterInformation {
        let length = usize_to_u32(self.display.hir_pat(pat).to_string().len());
        ParameterInformation {
            label: Span {
                start: offset,
                end: offset + length,
            },
            documentation: None,
        }
    }
}

fn usize_to_u32(x: usize) -> u32 {
    u32::try_from(x).expect("failed to cast usize to u32 while generating signature help")
}

fn process_args(args: &ast::Expr, location: u32) -> u32 {
    match &*args.kind {
        ast::ExprKind::Tuple(items) => {
            let len = items.len();
            let mut i = 0;
            while i < len && items.get(i).expect("").span.hi < location {
                i += 1;
            }
            usize_to_u32(i)
        }
        _ => 0,
    }
}
