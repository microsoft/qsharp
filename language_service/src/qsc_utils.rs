// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::compilation::Compilation;
use qsc::{ast, AstPackage, SourceMap, Span};

use crate::protocol;

pub(crate) const QSHARP_LIBRARY_URI_SCHEME: &str = "qsharp-library-source";

pub(crate) fn span_contains(span: Span, offset: u32) -> bool {
    offset >= span.lo && offset < span.hi
}

pub(crate) fn span_touches(span: Span, offset: u32) -> bool {
    offset >= span.lo && offset <= span.hi
}

pub(crate) fn resolve_offset<'a>(
    compilation: &'a Compilation,
    source_name: &str,
    offset: u32,
) -> (&'a AstPackage, u32) {
    let unit = compilation.current_unit();

    // Map the file offset into a SourceMap offset
    let offset = unit
        .sources
        .find_by_name(source_name)
        .expect("source should exist in the source map")
        .offset
        + offset;

    let ast_package = &unit.ast;
    (ast_package, offset)
}

pub(crate) fn protocol_span(span: Span, source_map: &SourceMap) -> protocol::Span {
    // Note that lo and hi offsets will usually be the same as
    // the span will usually come from a single source.
    let lo_offset = source_map
        .find_by_offset(span.lo)
        .expect("source should exist for offset")
        .offset;
    let hi_offset = source_map
        .find_by_offset(span.hi)
        .expect("source should exist for offset")
        .offset;
    protocol::Span {
        start: span.lo - lo_offset,
        end: span.hi - hi_offset,
    }
}

pub(crate) fn find_ident<'a>(
    node_id: &'a ast::NodeId,
    callable: &'a ast::CallableDecl,
) -> Option<&'a ast::Ident> {
    let mut finder = AstIdentFinder {
        node_id,
        ident: None,
    };
    {
        use ast::visit::Visitor;
        finder.visit_callable_decl(callable);
    }
    finder.ident
}

struct AstIdentFinder<'a> {
    pub node_id: &'a ast::NodeId,
    pub ident: Option<&'a ast::Ident>,
}

impl<'a> ast::visit::Visitor<'a> for AstIdentFinder<'a> {
    fn visit_pat(&mut self, pat: &'a ast::Pat) {
        match &*pat.kind {
            ast::PatKind::Bind(ident, _) => {
                if ident.id == *self.node_id {
                    self.ident = Some(ident);
                }
            }
            _ => ast::visit::walk_pat(self, pat),
        }
    }

    fn visit_expr(&mut self, expr: &'a ast::Expr) {
        if self.ident.is_none() {
            ast::visit::walk_expr(self, expr);
        }
    }
}
