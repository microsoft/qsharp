// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    compilation::Compilation,
    protocol::{self},
};
use qsc::line_column::{Encoding, Range};
use qsc::{ast, hir::PackageId, SourceMap, Span};

pub(crate) const QSHARP_LIBRARY_URI_SCHEME: &str = "qsharp-library-source";

pub(crate) fn span_contains(span: Span, offset: u32) -> bool {
    offset >= span.lo && offset < span.hi
}

pub(crate) fn span_touches(span: Span, offset: u32) -> bool {
    offset >= span.lo && offset <= span.hi
}

pub(crate) fn into_range(encoding: Encoding, span: Span, source_map: &SourceMap) -> Range {
    let lo_source = source_map
        .find_by_offset(span.lo)
        .expect("source should exist for offset");

    let hi_source = source_map
        .find_by_offset(span.hi)
        .expect("source should exist for offset");

    // Note that lo and hi offsets must always come from the same source.
    assert!(
        lo_source.offset == hi_source.offset,
        "span start and end must come from the same source"
    );

    Range::from_span(encoding, &lo_source.contents, &(span - lo_source.offset))
}

pub(crate) fn into_location(
    position_encoding: Encoding,
    compilation: &Compilation,
    location: Span,
    package_id: PackageId,
) -> protocol::Location {
    let source_map = &compilation
        .package_store
        .get(package_id)
        .expect("package id must exist in store")
        .sources;
    let source = source_map
        .find_by_offset(location.lo)
        .expect("source should exist for offset");
    let source_name = if package_id == compilation.user_package_id {
        source.name.to_string()
    } else {
        // Currently the only supported external packages are our library packages,
        // URI's to which need to include our custom library scheme.
        format!("{}:{}", QSHARP_LIBRARY_URI_SCHEME, source.name)
    };

    protocol::Location {
        source: source_name,
        span: Range::from_span(
            position_encoding,
            &source.contents,
            &(location - source.offset),
        ),
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
