// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use std::rc::Rc;

use crate::name_locator::{Handler, Locator, LocatorContext};
use crate::protocol::{Location, LocationSpan};
use crate::qsc_utils::{
    map_offset, resolve_item_relative_to_user_package, Compilation, QSHARP_LIBRARY_URI_SCHEME,
};
use qsc::ast::visit::{walk_expr, walk_namespace, walk_ty, Visitor};
use qsc::hir::ty::Ty;
use qsc::hir::{ItemId, PackageId, Res};
use qsc::{ast, hir, resolve, AstPackage, Span};

pub(crate) fn get_references(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> Vec<Location> {
    // Map the file offset into a SourceMap offset
    let offset = map_offset(&compilation.user_unit.sources, source_name, offset);

    let mut references_finder = ReferencesFinder {
        compilation,
        references: vec![],
    };

    let mut locator = Locator::new(&mut references_finder, offset, compilation);
    locator.visit_package(&compilation.user_unit.ast.package);

    references_finder.references
}

struct ReferencesFinder<'a> {
    compilation: &'a Compilation,
    references: Vec<Location>,
}

impl ReferencesFinder<'_> {
    fn add_ref_from_location(&mut self, location: LocationSpan) {
        self.references.push(Location {
            source: location.source,
            offset: location.span.start,
        });
    }

    fn add_ref_from_position(&mut self, lo: u32, package_id: Option<PackageId>) {
        let source_map = match package_id {
            Some(id) => {
                &self
                    .compilation
                    .package_store
                    .get(id)
                    .unwrap_or_else(|| panic!("package should exist for id {id}"))
                    .sources
            }
            None => &self.compilation.user_unit.sources,
        };
        let source = source_map
            .find_by_offset(lo)
            .expect("source should exist for offset");
        // Note: Having a package_id means the position references a foreign package.
        // Currently the only supported foreign packages are our library packages,
        // URI's to which need to include our custom library scheme.
        let source_name = match package_id {
            Some(_) => format!("{}:{}", QSHARP_LIBRARY_URI_SCHEME, source.name),
            None => source.name.to_string(),
        };
        //let source_name = source.name.to_string();

        self.references.push(Location {
            source: source_name,
            offset: lo - source.offset,
        });
    }
}

impl<'a> Handler<'a> for ReferencesFinder<'a> {
    fn at_callable_def(
        &mut self,
        _: &LocatorContext<'a>,
        name: &'a ast::Ident,
        _: &'a ast::CallableDecl,
    ) {
        if let Some(resolve::Res::Item(item_id)) = self.compilation.user_unit.ast.names.get(name.id)
        {
            for reference in find_item_locations(item_id, self.compilation) {
                self.add_ref_from_location(reference);
            }
        }
    }

    fn at_callable_ref(
        &mut self,
        _: &'a ast::Path,
        item_id: &'_ hir::ItemId,
        _: &'a hir::Item,
        _: &'a hir::Package,
        _: &'a hir::CallableDecl,
    ) {
        for reference in find_item_locations(item_id, self.compilation) {
            self.add_ref_from_location(reference);
        }
    }

    fn at_new_type_def(&mut self, type_name: &'a ast::Ident, _: &'a ast::TyDef) {
        if let Some(resolve::Res::Item(item_id)) =
            self.compilation.user_unit.ast.names.get(type_name.id)
        {
            for reference in find_item_locations(item_id, self.compilation) {
                self.add_ref_from_location(reference);
            }
        }
    }

    fn at_new_type_ref(
        &mut self,
        _: &'a ast::Path,
        item_id: &'_ hir::ItemId,
        _: &'a hir::Package,
        _: &'a hir::Ident,
        _: &'a hir::ty::Udt,
    ) {
        for reference in find_item_locations(item_id, self.compilation) {
            self.add_ref_from_location(reference);
        }
    }

    fn at_field_def(
        &mut self,
        context: &LocatorContext<'a>,
        field_name: &'a ast::Ident,
        _: &'a ast::Ty,
    ) {
        if let Some(ty_item_id) = context.current_udt_id {
            for reference in
                find_field_locations(ty_item_id, field_name.name.clone(), self.compilation)
            {
                self.add_ref_from_location(reference);
            }
        }
    }

    fn at_field_ref(
        &mut self,
        field_ref: &'a ast::Ident,
        _: &'a ast::NodeId,
        item_id: &'_ hir::ItemId,
        _: &'a hir::ty::UdtField,
    ) {
        for reference in find_field_locations(item_id, field_ref.name.clone(), self.compilation) {
            self.add_ref_from_location(reference);
        }
    }

    fn at_local_def(
        &mut self,
        context: &LocatorContext<'a>,
        ident: &'a ast::Ident,
        _: &'a ast::Pat,
    ) {
        if let Some(curr) = context.current_callable {
            for reference in find_local_locations(ident.id, curr, self.compilation) {
                self.add_ref_from_location(reference);
            }
        }
    }

    fn at_local_ref(
        &mut self,
        context: &LocatorContext<'a>,
        _: &'a ast::Path,
        _: &'a ast::NodeId,
        ident: &'a ast::Ident,
    ) {
        if let Some(curr) = context.current_callable {
            for reference in find_local_locations(ident.id, curr, self.compilation) {
                self.add_ref_from_location(reference);
            }
        }
    }
}

fn get_location_span(
    compilation: &Compilation,
    location: Span,
    package_id: Option<PackageId>,
) -> LocationSpan {
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

    LocationSpan {
        source: source_name,
        span: crate::protocol::Span {
            start: location.lo - source.offset,
            end: location.hi - source.offset,
        },
    }
}

pub(crate) fn find_item_locations(
    item_id: &hir::ItemId,
    compilation: &Compilation,
) -> Vec<LocationSpan> {
    let (def, _) = resolve_item_relative_to_user_package(compilation, item_id);
    let def_span = match &def.kind {
        hir::ItemKind::Callable(decl) => decl.name.span,
        hir::ItemKind::Namespace(name, _) | hir::ItemKind::Ty(name, _) => name.span,
    };

    let mut find_refs = FindItemRefs {
        item_id,
        ast_package: &compilation.user_unit.ast,
        locations: vec![],
    };

    //////////////////////////////// VERY SIMILAR CODE BLOCK /////////////////////////////////////////////
    let mut locations = vec![];
    locations.push(get_location_span(compilation, def_span, item_id.package));
    if let Some(library_package_id) = item_id.package {
        let def_unit = compilation
            .package_store
            .get(library_package_id)
            .expect("package should exist in store");

        let mut find_refs = FindItemRefs {
            item_id: &ItemId {
                package: None,
                item: item_id.item,
            },
            ast_package: &def_unit.ast,
            locations: vec![],
        };

        find_refs.visit_package(&def_unit.ast.package);
        locations.extend(
            find_refs
                .locations
                .drain(..)
                .map(|l| get_location_span(compilation, l, item_id.package)),
        );
    }

    find_refs.visit_package(&compilation.user_unit.ast.package);
    locations.extend(
        find_refs
            .locations
            .drain(..)
            .map(|l| get_location_span(compilation, l, None)),
    );

    locations
    //////////////////////////////////////////////////////////////////////////////////////////////////////
}

pub(crate) fn find_field_locations(
    ty_item_id: &hir::ItemId,
    field_name: Rc<str>,
    compilation: &Compilation,
) -> Vec<LocationSpan> {
    let (ty_def, _) = resolve_item_relative_to_user_package(compilation, ty_item_id);
    if let hir::ItemKind::Ty(_, udt) = &ty_def.kind {
        let ty_field = udt
            .find_field_by_name(&field_name)
            .expect("field name should exist");
        let def_span = ty_field
            .name_span
            .expect("field found via name should have a name");

        let mut find_refs = FindFieldRefs {
            ty_item_id,
            field_name: field_name.clone(),
            ast_package: &compilation.user_unit.ast,
            locations: vec![],
        };

        //////////////////////////////// VERY SIMILAR CODE BLOCK /////////////////////////////////////////////
        let mut locations = vec![];
        locations.push(get_location_span(compilation, def_span, ty_item_id.package));
        if let Some(library_package_id) = ty_item_id.package {
            let def_unit = compilation
                .package_store
                .get(library_package_id)
                .expect("package should exist in store");

            let mut find_refs = FindFieldRefs {
                ty_item_id: &ItemId {
                    package: None,
                    item: ty_item_id.item,
                },
                field_name,
                ast_package: &def_unit.ast,
                locations: vec![],
            };

            find_refs.visit_package(&def_unit.ast.package);
            locations.extend(
                find_refs
                    .locations
                    .drain(..)
                    .map(|l| get_location_span(compilation, l, ty_item_id.package)),
            );
        }

        find_refs.visit_package(&compilation.user_unit.ast.package);
        locations.extend(
            find_refs
                .locations
                .drain(..)
                .map(|l| get_location_span(compilation, l, None)),
        );

        locations
        //////////////////////////////////////////////////////////////////////////////////////////////////////
    } else {
        vec![]
    }
}

pub(crate) fn find_local_locations(
    node_id: ast::NodeId,
    callable: &ast::CallableDecl,
    compilation: &Compilation,
) -> Vec<LocationSpan> {
    let mut find_refs = FindLocalLocations {
        node_id,
        compilation,
        locations: vec![],
    };
    find_refs.visit_callable_decl(callable);
    find_refs
        .locations
        .into_iter()
        .map(|l| get_location_span(compilation, l, None))
        .collect()
}

struct FindItemRefs<'a> {
    item_id: &'a hir::ItemId,
    ast_package: &'a AstPackage,
    locations: Vec<Span>,
}

impl<'a> Visitor<'_> for FindItemRefs<'a> {
    fn visit_path(&mut self, path: &'_ ast::Path) {
        let res = self.ast_package.names.get(path.id);
        if let Some(resolve::Res::Item(item_id)) = res {
            if *item_id == *self.item_id {
                self.locations.push(path.name.span);
            }
        }
    }

    fn visit_ty(&mut self, ty: &'_ ast::Ty) {
        if let ast::TyKind::Path(ty_path) = &*ty.kind {
            let res = self.ast_package.names.get(ty_path.id);
            if let Some(resolve::Res::Item(item_id)) = res {
                if *item_id == *self.item_id {
                    self.locations.push(ty_path.name.span);
                }
            }
        } else {
            walk_ty(self, ty);
        }
    }
}

struct FindFieldRefs<'a> {
    ty_item_id: &'a hir::ItemId,
    field_name: Rc<str>,
    ast_package: &'a AstPackage,
    locations: Vec<Span>,
}

impl<'a> Visitor<'_> for FindFieldRefs<'a> {
    fn visit_expr(&mut self, expr: &'_ ast::Expr) {
        if let ast::ExprKind::Field(qualifier, field_name) = &*expr.kind {
            self.visit_expr(qualifier);
            if field_name.name == self.field_name {
                if let Some(Ty::Udt(Res::Item(id))) = self.ast_package.tys.terms.get(qualifier.id) {
                    if id == self.ty_item_id {
                        self.locations.push(field_name.span);
                    }
                }
            }
        } else {
            walk_expr(self, expr);
        }
    }
}

struct FindLocalLocations<'a> {
    node_id: ast::NodeId,
    compilation: &'a Compilation,
    locations: Vec<Span>,
}

impl<'a> Visitor<'_> for FindLocalLocations<'a> {
    fn visit_pat(&mut self, pat: &'_ ast::Pat) {
        match &*pat.kind {
            ast::PatKind::Bind(ident, _) => {
                if ident.id == self.node_id {
                    self.locations.push(ident.span);
                }
            }
            _ => ast::visit::walk_pat(self, pat),
        }
    }

    fn visit_path(&mut self, path: &'_ ast::Path) {
        let res = self.compilation.user_unit.ast.names.get(path.id);
        if let Some(resolve::Res::Local(node_id)) = res {
            if *node_id == self.node_id {
                self.locations.push(path.name.span);
            }
        }
    }
}
