// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::compilation::Compilation;
use qsc::line_column::{Encoding, Range};
use qsc::location::Location;
use qsc::{ast, hir::PackageId, SourceMap, Span};
use qsc::{
    compile::{self},
    completion::Prediction,
};

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
    span: Span,
    package_id: PackageId,
) -> Location {
    Location::from(
        span,
        package_id,
        &compilation.package_store,
        position_encoding,
    )
}

pub(crate) fn find_ident(
    node_id: ast::NodeId,
    callable: &ast::CallableDecl,
) -> Option<&ast::Ident> {
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
    pub node_id: ast::NodeId,
    pub ident: Option<&'a ast::Ident>,
}

impl<'a> ast::visit::Visitor<'a> for AstIdentFinder<'a> {
    fn visit_pat(&mut self, pat: &'a ast::Pat) {
        match &*pat.kind {
            ast::PatKind::Bind(ident, _) => {
                if ident.id == self.node_id {
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

pub(crate) fn whats_next(source: &str, cursor_offset: u32, notebook: bool) -> Vec<Prediction> {
    compile::whats_next(source, cursor_offset, notebook)
}

// pub(crate) fn find_item<'a>(
//     compilation: &'a Compilation,
//     id: &ItemId,
// ) -> (Option<&'a Item>, Option<&'a Package>) {
//     let package = if let Some(package_id) = id.package {
//         match compilation.package_store.get(package_id) {
//             Some(compilation) => &compilation.package,
//             None => return (None, None),
//         }
//     } else {
//         &compilation.unit.package
//     };
//     (package.items.get(id.item), Some(package))
// }
