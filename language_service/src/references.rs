// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use std::rc::Rc;

use crate::name_locator::{Handler, Locator, LocatorContext};
use crate::protocol::Location;
use crate::qsc_utils::{
    map_offset, resolve_item_relative_to_user_package, Compilation, QSHARP_LIBRARY_URI_SCHEME,
};
use qsc::ast::visit::{walk_expr, walk_ty, Visitor};
use qsc::hir::ty::Ty;
use qsc::hir::{PackageId, Res};
use qsc::{ast, hir, resolve, Span};

pub(crate) fn get_references(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> Vec<Location> {
    // Map the file offset into a SourceMap offset
    let offset = map_offset(&compilation.user_unit.sources, source_name, offset);

    let mut definition_finder = DefinitionFinder {
        compilation,
        definition: None,
    };

    let mut locator = Locator::new(&mut definition_finder, offset, compilation);
    locator.visit_package(&compilation.user_unit.ast.package);

    definition_finder
        .definition
        .map(|(name, offset)| Location {
            source: name,
            offset,
        })
        .into_iter()
        .collect()
}

struct DefinitionFinder<'a> {
    compilation: &'a Compilation,
    definition: Option<(String, u32)>,
}

impl DefinitionFinder<'_> {
    fn set_definition_from_position(&mut self, lo: u32, package_id: Option<PackageId>) {
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

        self.definition = Some((source_name, lo - source.offset));
    }
}

impl<'a> Handler<'a> for DefinitionFinder<'a> {
    fn at_callable_def(
        &mut self,
        _: &LocatorContext<'a>,
        name: &'a ast::Ident,
        _: &'a ast::CallableDecl,
    ) {
        self.set_definition_from_position(name.span.lo, None);
    }

    fn at_callable_ref(
        &mut self,
        _: &'a ast::Path,
        item_id: &'_ hir::ItemId,
        _: &'a hir::Item,
        _: &'a hir::Package,
        decl: &'a hir::CallableDecl,
    ) {
        self.set_definition_from_position(decl.name.span.lo, item_id.package);
    }

    fn at_new_type_def(&mut self, type_name: &'a ast::Ident, _: &'a ast::TyDef) {
        self.set_definition_from_position(type_name.span.lo, None);
    }

    fn at_new_type_ref(
        &mut self,
        _: &'a ast::Path,
        item_id: &'_ hir::ItemId,
        _: &'a hir::Package,
        type_name: &'a hir::Ident,
        _: &'a hir::ty::Udt,
    ) {
        self.set_definition_from_position(type_name.span.lo, item_id.package);
    }

    fn at_field_def(&mut self, _: &LocatorContext<'a>, field_name: &'a ast::Ident, _: &'a ast::Ty) {
        self.set_definition_from_position(field_name.span.lo, None);
    }

    fn at_field_ref(
        &mut self,
        _: &'a ast::Ident,
        _: &'a ast::NodeId,
        item_id: &'_ hir::ItemId,
        field_def: &'a hir::ty::UdtField,
    ) {
        let span = field_def
            .name_span
            .expect("field found via name should have a name");
        self.set_definition_from_position(span.lo, item_id.package);
    }

    fn at_local_def(&mut self, _: &LocatorContext<'a>, ident: &'a ast::Ident, _: &'a ast::Pat) {
        self.set_definition_from_position(ident.span.lo, None);
    }

    fn at_local_ref(
        &mut self,
        _: &LocatorContext<'a>,
        _: &'a ast::Path,
        _: &'a ast::NodeId,
        ident: &'a ast::Ident,
    ) {
        self.set_definition_from_position(ident.span.lo, None);
    }
}

pub(crate) fn find_item_locations(item_id: &hir::ItemId, compilation: &Compilation) -> Vec<Span> {
    let (def, _) = resolve_item_relative_to_user_package(compilation, item_id);
    let def_span = match &def.kind {
        hir::ItemKind::Callable(decl) => decl.name.span,
        hir::ItemKind::Namespace(name, _) | hir::ItemKind::Ty(name, _) => name.span,
    };
    let mut find_refs = FindItemRefs {
        item_id,
        compilation,
        locations: vec![],
    };
    find_refs.visit_package(&compilation.user_unit.ast.package);
    find_refs.locations.push(def_span);
    find_refs.locations
}

pub(crate) fn find_field_locations(
    ty_item_id: &hir::ItemId,
    field_name: Rc<str>,
    compilation: &Compilation,
) -> Vec<Span> {
    let (ty_def, _) = resolve_item_relative_to_user_package(compilation, ty_item_id);
    if let hir::ItemKind::Ty(_, udt) = &ty_def.kind {
        let ty_field = udt
            .find_field_by_name(&field_name)
            .expect("field name should exist");
        let def_span = ty_field
            .name_span
            .expect("field found via name should have a name");
        let mut rename = FindFieldRefs {
            ty_item_id,
            field_name,
            compilation,
            locations: vec![],
        };
        rename.visit_package(&compilation.user_unit.ast.package);
        rename.locations.push(def_span);
        rename.locations
    } else {
        vec![]
    }
}

pub(crate) fn find_local_locations(
    node_id: ast::NodeId,
    callable: &ast::CallableDecl,
    compilation: &Compilation,
) -> Vec<Span> {
    let mut rename = FindLocalLocations {
        node_id,
        compilation,
        locations: vec![],
    };
    rename.visit_callable_decl(callable);
    rename.locations
}

struct FindItemRefs<'a> {
    item_id: &'a hir::ItemId,
    compilation: &'a Compilation,
    locations: Vec<Span>,
}

impl<'a> Visitor<'_> for FindItemRefs<'a> {
    fn visit_path(&mut self, path: &'_ ast::Path) {
        let res = self.compilation.user_unit.ast.names.get(path.id);
        if let Some(resolve::Res::Item(item_id)) = res {
            if *item_id == *self.item_id {
                self.locations.push(path.name.span);
            }
        }
    }

    fn visit_ty(&mut self, ty: &'_ ast::Ty) {
        if let ast::TyKind::Path(ty_path) = &*ty.kind {
            let res = self.compilation.user_unit.ast.names.get(ty_path.id);
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
    compilation: &'a Compilation,
    locations: Vec<Span>,
}

impl<'a> Visitor<'_> for FindFieldRefs<'a> {
    fn visit_expr(&mut self, expr: &'_ ast::Expr) {
        if let ast::ExprKind::Field(qualifier, field_name) = &*expr.kind {
            self.visit_expr(qualifier);
            if field_name.name == self.field_name {
                if let Some(Ty::Udt(Res::Item(id))) =
                    self.compilation.user_unit.ast.tys.terms.get(qualifier.id)
                {
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
