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
    pub user_unit: CompileUnit,
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
        user_unit: unit,
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
    let lo_source = source_map
        .find_by_offset(span.lo)
        .expect("source should exist for offset");

    let hi_source = source_map
        .find_by_offset(span.hi)
        .expect("source should exist for offset");

    // Note that lo and hi offsets must always come from the same source.
    assert!(
        lo_source.name == hi_source.name,
        "span start and end must come from the same source"
    );
    protocol::Span {
        start: span.lo - lo_source.offset,
        end: span.hi - hi_source.offset,
    }
}

pub(crate) fn protocol_location(
    compilation: &Compilation,
    location: Span,
    package_id: Option<PackageId>,
) -> protocol::Location {
    let package = if let Some(library_package_id) = package_id {
        compilation
            .package_store
            .get(library_package_id)
            .expect("package should exist in store")
    } else {
        &compilation.user_unit
    };
    let source = package
        .sources
        .find_by_offset(location.lo)
        .expect("source should exist in package");

    // Note: Having a package_id means the position references a foreign package.
    // Currently the only supported foreign packages are our library packages,
    // URI's to which need to include our custom library scheme.
    let source_name = match package_id {
        Some(_) => format!("{}:{}", QSHARP_LIBRARY_URI_SCHEME, source.name),
        None => source.name.to_string(),
    };

    protocol::Location {
        source: source_name,
        span: protocol::Span {
            start: location.lo - source.offset,
            end: location.hi - source.offset,
        },
    }
}

pub(crate) fn map_offset(source_map: &SourceMap, source_name: &str, source_offset: u32) -> u32 {
    source_map
        .find_by_name(source_name)
        .expect("source should exist in the source map")
        .offset
        + source_offset
}

/// Returns the hir `Item` node referred to by `item_id`,
/// along with the `Package` and `PackageId` for the package
/// that it was found in.
pub(crate) fn resolve_item_relative_to_user_package<'a>(
    compilation: &'a Compilation,
    item_id: &ItemId,
) -> (&'a Item, &'a Package, ItemId) {
    resolve_item(compilation, None, item_id)
}

/// Returns the hir `Item` node referred to by `res`.
/// `Res`s can resolve to external packages, and the references
/// are relative, so here we also need the
/// local `PackageId` that the `res` itself came from.
pub(crate) fn resolve_item_res<'a>(
    compilation: &'a Compilation,
    local_package_id: Option<PackageId>,
    res: &hir::Res,
) -> (&'a Item, ItemId) {
    match res {
        hir::Res::Item(item_id) => {
            let (item, _, resolved_item_id) = resolve_item(compilation, local_package_id, item_id);
            (item, resolved_item_id)
        }
        _ => panic!("expected to find item"),
    }
}

/// Returns the hir `Item` node referred to by `item_id`.
/// `ItemId`s can refer to external packages, and the references
/// are relative, so here we also need the local `PackageId`
/// that the `ItemId` originates from.
pub(crate) fn resolve_item<'a>(
    compilation: &'a Compilation,
    local_package_id: Option<PackageId>,
    item_id: &ItemId,
) -> (&'a Item, &'a Package, ItemId) {
    // If the `ItemId` contains a package id, use that.
    // Lack of a package id means the item is in the
    // same package as the one this `ItemId` reference
    // came from. So use the local package id passed in.
    let package_id = item_id.package.or(local_package_id);
    let package = if let Some(library_package_id) = package_id {
        // stdlib or core
        &compilation
            .package_store
            .get(library_package_id)
            .expect("package should exist in store")
            .package
    } else {
        // user code
        &compilation.user_unit.package
    };
    (
        package
            .items
            .get(item_id.item)
            .expect("item id should exist"),
        package,
        ItemId {
            package: package_id,
            item: item_id.item,
        },
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
