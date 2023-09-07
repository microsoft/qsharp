// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use qsc::ast::{
    self,
    visit::{walk_callable_decl, walk_expr, walk_item, Visitor},
};

use crate::{
    protocol::{ParameterInformation, SignatureHelp, SignatureInformation, Span},
    qsc_utils::{map_offset, span_contains, Compilation},
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
    };

    finder.visit_package(package);

    finder.signature_help.map(|_| SignatureHelp {
        signatures: vec![SignatureInformation {
            label: "operation Foo(a: Int, b: Double, c: String) : Unit".to_string(),
            documentation: None,
            parameters: vec![
                ParameterInformation {
                    label: Span { start: 14, end: 20 },
                    documentation: Some("The parameter `a`".to_string()),
                },
                ParameterInformation {
                    label: Span { start: 22, end: 31 },
                    documentation: Some("The parameter `b`".to_string()),
                },
                ParameterInformation {
                    label: Span { start: 33, end: 42 },
                    documentation: Some("The parameter `c`".to_string()),
                },
            ],
        }],
        active_signature: 0,
        active_parameter: 2,
    })
}

struct SignatureHelpFinder<'a> {
    compilation: &'a Compilation,
    offset: u32,
    signature_help: Option<()>,
}

impl<'a> Visitor<'a> for SignatureHelpFinder<'a> {
    fn visit_item(&mut self, item: &'a ast::Item) {
        if span_contains(item.span, self.offset) {
            walk_item(self, item);
        }
    }

    fn visit_callable_decl(&mut self, decl: &'a ast::CallableDecl) {
        walk_callable_decl(self, decl);
    }

    fn visit_expr(&mut self, expr: &'a ast::Expr) {
        if span_contains(expr.span, self.offset) {
            match &*expr.kind {
                ast::ExprKind::Call(callee, arguments) => {
                    self.signature_help = Some(());
                }
                _ => walk_expr(self, expr),
            }
        }
    }
}
