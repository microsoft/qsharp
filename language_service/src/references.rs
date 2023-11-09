// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use std::rc::Rc;

use crate::name_locator::{Handler, Locator, LocatorContext};
use crate::protocol::Location;
use crate::qsc_utils::{
    map_offset, protocol_location, resolve_item_relative_to_user_package, Compilation,
};
use qsc::ast::visit::{walk_expr, walk_ty, Visitor};
use qsc::hir::ty::Ty;
use qsc::hir::Res;
use qsc::{ast, hir, resolve, Span};

pub(crate) fn get_references(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
    include_declaration: bool,
) -> Vec<Location> {
    // Map the file offset into a SourceMap offset
    let offset = map_offset(&compilation.user_unit.sources, source_name, offset);

    let mut references_finder = ReferencesFinder {
        compilation,
        include_declaration,
        references: vec![],
    };

    let mut locator = Locator::new(&mut references_finder, offset, compilation);
    locator.visit_package(&compilation.user_unit.ast.package);

    references_finder.references
}

struct ReferencesFinder<'a> {
    compilation: &'a Compilation,
    include_declaration: bool,
    references: Vec<Location>,
}

impl<'a> Handler<'a> for ReferencesFinder<'a> {
    fn at_callable_def(
        &mut self,
        _: &LocatorContext<'a>,
        name: &'a ast::Ident,
        _: &'a ast::CallableDecl,
    ) {
        if let Some(resolve::Res::Item(item_id, _)) =
            self.compilation.user_unit.ast.names.get(name.id)
        {
            self.references =
                find_item_locations(item_id, self.compilation, self.include_declaration);
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
        self.references = find_item_locations(item_id, self.compilation, self.include_declaration);
    }

    fn at_new_type_def(&mut self, type_name: &'a ast::Ident, _: &'a ast::TyDef) {
        if let Some(resolve::Res::Item(item_id, _)) =
            self.compilation.user_unit.ast.names.get(type_name.id)
        {
            self.references =
                find_item_locations(item_id, self.compilation, self.include_declaration);
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
        self.references = find_item_locations(item_id, self.compilation, self.include_declaration);
    }

    fn at_field_def(
        &mut self,
        context: &LocatorContext<'a>,
        field_name: &'a ast::Ident,
        _: &'a ast::Ty,
    ) {
        if let Some(ty_item_id) = context.current_udt_id {
            self.references = find_field_locations(
                ty_item_id,
                field_name.name.clone(),
                self.compilation,
                self.include_declaration,
            );
        }
    }

    fn at_field_ref(
        &mut self,
        field_ref: &'a ast::Ident,
        _: &'a ast::NodeId,
        item_id: &'_ hir::ItemId,
        _: &'a hir::ty::UdtField,
    ) {
        self.references = find_field_locations(
            item_id,
            field_ref.name.clone(),
            self.compilation,
            self.include_declaration,
        );
    }

    fn at_local_def(
        &mut self,
        context: &LocatorContext<'a>,
        ident: &'a ast::Ident,
        _: &'a ast::Pat,
    ) {
        if let Some(curr) = context.current_callable {
            self.references =
                find_local_locations(ident.id, curr, self.compilation, self.include_declaration);
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
            self.references =
                find_local_locations(ident.id, curr, self.compilation, self.include_declaration);
        }
    }
}

pub(crate) fn find_item_locations(
    item_id: &hir::ItemId,
    compilation: &Compilation,
    include_declaration: bool,
) -> Vec<Location> {
    let mut locations = vec![];

    if include_declaration {
        let (def, _, resolved_item_id) =
            resolve_item_relative_to_user_package(compilation, item_id);
        let def_span = match &def.kind {
            hir::ItemKind::Callable(decl) => decl.name.span,
            hir::ItemKind::Namespace(name, _) | hir::ItemKind::Ty(name, _) => name.span,
        };
        locations.push(protocol_location(
            compilation,
            def_span,
            resolved_item_id.package,
        ));
    }

    let mut find_refs = FindItemRefs {
        item_id,
        compilation,
        locations: vec![],
    };

    find_refs.visit_package(&compilation.user_unit.ast.package);
    locations.extend(
        find_refs
            .locations
            .drain(..)
            .map(|l| protocol_location(compilation, l, None)),
    );

    locations
}

pub(crate) fn find_field_locations(
    ty_item_id: &hir::ItemId,
    field_name: Rc<str>,
    compilation: &Compilation,
    include_declaration: bool,
) -> Vec<Location> {
    let mut locations = vec![];

    if include_declaration {
        let (ty_def, _, resolved_ty_item_id) =
            resolve_item_relative_to_user_package(compilation, ty_item_id);
        if let hir::ItemKind::Ty(_, udt) = &ty_def.kind {
            let ty_field = udt
                .find_field_by_name(&field_name)
                .expect("field name should exist");
            let def_span = ty_field
                .name_span
                .expect("field found via name should have a name");
            locations.push(protocol_location(
                compilation,
                def_span,
                resolved_ty_item_id.package,
            ));
        } else {
            panic!("item id resolved to non-type: {ty_item_id}");
        }
    }

    let mut find_refs = FindFieldRefs {
        ty_item_id,
        field_name,
        compilation,
        locations: vec![],
    };

    find_refs.visit_package(&compilation.user_unit.ast.package);
    locations.extend(
        find_refs
            .locations
            .drain(..)
            .map(|l| protocol_location(compilation, l, None)),
    );

    locations
}

pub(crate) fn find_local_locations(
    node_id: ast::NodeId,
    callable: &ast::CallableDecl,
    compilation: &Compilation,
    include_declaration: bool,
) -> Vec<Location> {
    let mut find_refs = FindLocalLocations {
        node_id,
        compilation,
        include_declaration,
        locations: vec![],
    };
    find_refs.visit_callable_decl(callable);
    find_refs
        .locations
        .into_iter()
        .map(|l| protocol_location(compilation, l, None))
        .collect()
}

struct FindItemRefs<'a> {
    item_id: &'a hir::ItemId,
    compilation: &'a Compilation,
    locations: Vec<Span>,
}

impl<'a> Visitor<'_> for FindItemRefs<'a> {
    fn visit_path(&mut self, path: &'_ ast::Path) {
        let res = self.compilation.user_unit.ast.names.get(path.id);
        if let Some(resolve::Res::Item(item_id, _)) = res {
            if *item_id == *self.item_id {
                self.locations.push(path.name.span);
            }
        }
    }

    fn visit_ty(&mut self, ty: &'_ ast::Ty) {
        if let ast::TyKind::Path(ty_path) = &*ty.kind {
            let res = self.compilation.user_unit.ast.names.get(ty_path.id);
            if let Some(resolve::Res::Item(item_id, _)) = res {
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
    include_declaration: bool,
    locations: Vec<Span>,
}

impl<'a> Visitor<'_> for FindLocalLocations<'a> {
    fn visit_pat(&mut self, pat: &'_ ast::Pat) {
        if self.include_declaration {
            match &*pat.kind {
                ast::PatKind::Bind(ident, _) => {
                    if ident.id == self.node_id {
                        self.locations.push(ident.span);
                    }
                }
                _ => ast::visit::walk_pat(self, pat),
            }
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
