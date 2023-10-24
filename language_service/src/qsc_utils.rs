// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::{
    ast,
    compile::{self, Error},
    hir::{self, Item, ItemId, Package, PackageId},
    CompileUnit, PackageStore, PackageType, SourceMap, Span, TargetProfile,
};

use crate::protocol;

pub(crate) const QSHARP_LIBRARY_URI_SCHEME: &str = "qsharp-library-source";

/// Represents an immutable compilation state that can be used
/// to implement language service features.
pub(crate) struct Compilation {
    pub package_store: PackageStore,
    pub std_package_id: PackageId,
    pub unit: CompileUnit,
    pub errors: Vec<Error>,
}

pub(crate) fn compile_document(
    source_name: &str,
    source_contents: &str,
    package_type: PackageType,
    target_profile: TargetProfile,
) -> Compilation {
    let mut package_store = PackageStore::new(compile::core());
    let std_package_id = package_store.insert(compile::std(&package_store, target_profile));

    // Source map only contains the current document.
    let source_map = SourceMap::new([(source_name.into(), source_contents.into())], None);
    let (unit, errors) = compile::compile(
        &package_store,
        &[std_package_id],
        source_map,
        package_type,
        target_profile,
    );
    Compilation {
        package_store,
        std_package_id,
        unit,
        errors,
    }
}

pub(crate) fn span_contains(span: Span, offset: u32) -> bool {
    offset >= span.lo && offset < span.hi
}

pub(crate) fn span_touches(span: Span, offset: u32) -> bool {
    offset >= span.lo && offset <= span.hi
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

pub(crate) fn map_offset(source_map: &SourceMap, source_name: &str, source_offset: u32) -> u32 {
    source_map
        .find_by_name(source_name)
        .expect("source should exist in the source map")
        .offset
        + source_offset
}

pub(crate) fn resolve_item_from_current_package<'a>(
    compilation: &'a Compilation,
    id: &ItemId,
) -> (&'a Item, &'a Package, Option<PackageId>) {
    resolve(compilation, None, id)
}

pub(crate) fn resolve_udt_res<'a>(
    compilation: &'a Compilation,
    local_package: Option<PackageId>,
    res: &hir::Res,
) -> (&'a Item, Option<PackageId>) {
    match res {
        hir::Res::Item(item_id) => {
            let (item, _, package_id) = resolve(compilation, local_package, item_id);
            (item, package_id)
        }
        _ => panic!("expected to find item"),
    }
}

fn resolve<'a>(
    compilation: &'a Compilation,
    local_package_id: Option<PackageId>,
    id: &ItemId,
) -> (&'a Item, &'a Package, Option<PackageId>) {
    let package_id = id.package.or(local_package_id);
    let package = if let Some(external_package_id) = package_id {
        &compilation
            .package_store
            .get(external_package_id)
            .expect("package should exist in store")
            .package
    } else {
        &compilation.unit.package
    };
    (
        package.items.get(id.item).expect("item id should exist"),
        package,
        package_id,
    )
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
