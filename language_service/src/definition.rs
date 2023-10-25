// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::cursor_locator::{CursorLocator, CursorLocatorAPI, LocatorContext};
use crate::protocol::Definition;
use crate::qsc_utils::{map_offset, Compilation, QSHARP_LIBRARY_URI_SCHEME};
use qsc::ast::visit::Visitor;
use qsc::hir::PackageId;
use qsc::{ast, hir};

pub(crate) fn get_definition(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> Option<Definition> {
    // Map the file offset into a SourceMap offset
    let offset = map_offset(&compilation.unit.sources, source_name, offset);

    let mut definition_finder = DefinitionFinder {
        compilation,
        definition: None,
    };

    let mut locator = CursorLocator::new(&mut definition_finder, offset, compilation);
    locator.visit_package(&compilation.unit.ast.package);

    definition_finder
        .definition
        .map(|(name, offset)| Definition {
            source: name,
            offset,
        })
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
            None => &self.compilation.unit.sources,
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

impl<'a> CursorLocatorAPI<'a> for DefinitionFinder<'a> {
    fn at_callable_def(&mut self, context: &LocatorContext<'a>, decl: &'a ast::CallableDecl) {
        self.set_definition_from_position(decl.name.span.lo, None);
    }

    fn at_callable_ref(
        &mut self,
        path: &'a ast::Path,
        item_id: &'a hir::ItemId,
        item: &'a hir::Item,
        package: &'a hir::Package,
        decl: &'a hir::CallableDecl,
    ) {
        self.set_definition_from_position(decl.name.span.lo, item_id.package);
    }

    fn at_new_type_def(&mut self, type_name: &'a ast::Ident, def: &'a ast::TyDef) {
        self.set_definition_from_position(type_name.span.lo, None);
    }

    fn at_new_type_ref(
        &mut self,
        path: &'a ast::Path,
        item_id: &'a hir::ItemId,
        item: &'a hir::Item,
        package: &'a hir::Package,
        type_name: &'a hir::Ident,
        udt: &'a hir::ty::Udt,
    ) {
        self.set_definition_from_position(type_name.span.lo, item_id.package);
    }

    fn at_field_def(&mut self, field_name: &'a ast::Ident, ty: &'a ast::Ty) {
        self.set_definition_from_position(field_name.span.lo, None);
    }

    fn at_field_ref(
        &mut self,
        expr_id: &'a ast::NodeId,
        field_ref: &'a ast::Ident,
        item_id: &'a hir::ItemId,
        field_def: &'a hir::ty::UdtField,
    ) {
        let span = field_def
            .name_span
            .expect("field found via name should have a name");
        self.set_definition_from_position(span.lo, item_id.package);
    }

    fn at_local_def(
        &mut self,
        context: &LocatorContext<'a>,
        pat: &'a ast::Pat,
        ident: &'a ast::Ident,
    ) {
        self.set_definition_from_position(ident.span.lo, None);
    }

    fn at_local_ref(
        &mut self,
        context: &LocatorContext<'a>,
        path: &'a ast::Path,
        node_id: &'a ast::NodeId,
        ident: &'a ast::Ident,
    ) {
        self.set_definition_from_position(ident.span.lo, None);
    }
}
