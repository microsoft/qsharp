// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::{
    ast::{
        visit::{self, Visitor},
        Attr, Block, CallableDecl, Expr, ExprKind, FieldAssign, FieldDef, FunctorExpr, Ident,
        Idents, Item, ItemKind, Namespace, Package, Pat, Path, QubitInit, SpecDecl, Stmt,
        StructDecl, Ty, TyDef, TyKind,
    },
    parse::completion::PathKind,
};
use std::rc::Rc;

/// Provides the qualifier and the expected name kind for the
/// incomplete path (e.g. `foo.bar.`) at the cursor offset.
///
/// Methods may panic if the offset does not fall within an incomplete path.
#[derive(Debug)]
pub(super) struct IncompletePath<'a> {
    qualifier: Option<Vec<&'a Ident>>,
    context: Option<PathKind>,
    offset: u32,
}

impl<'a> IncompletePath<'a> {
    pub fn init(offset: u32, package: &'a Package) -> Self {
        let mut offset_visitor = OffsetVisitor {
            offset,
            visitor: IncompletePath {
                offset,
                context: None,
                qualifier: None,
            },
        };

        offset_visitor.visit_package(package);

        offset_visitor.visitor
    }
}

impl<'a> Visitor<'a> for IncompletePath<'a> {
    fn visit_item(&mut self, item: &'a Item) {
        match &*item.kind {
            ItemKind::Open(..) => self.context = Some(PathKind::Namespace),
            ItemKind::ImportOrExport(decl) => {
                self.context = Some(PathKind::Import);
                for item in &decl.items {
                    if item.is_glob
                        && item.span.touches(self.offset)
                        && item
                            .alias
                            .as_ref()
                            .map_or(true, |a| !a.span.touches(self.offset))
                    {
                        // Special case when the cursor falls *between* the
                        // `Path` and the glob asterisk,
                        // e.g. `foo.bar.|*` . In that case, the visitor
                        // will not visit the path since the cursor technically
                        // is not within the path.
                        self.visit_path_kind(&item.path);
                    }
                }
            }
            _ => {}
        }
    }

    fn visit_ty(&mut self, ty: &Ty) {
        if let TyKind::Path(..) = *ty.kind {
            self.context = Some(PathKind::Ty);
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {
        if let ExprKind::Path(..) = *expr.kind {
            self.context = Some(PathKind::Expr);
        } else if let ExprKind::Struct(..) = *expr.kind {
            self.context = Some(PathKind::Struct);
        }
    }

    fn visit_path_kind(&mut self, path: &'a qsc::ast::PathKind) {
        self.qualifier = match path {
            qsc::ast::PathKind::Ok(path) => Some(path.iter().collect()),
            qsc::ast::PathKind::Err(Some(incomplete_path)) => {
                Some(incomplete_path.segments.iter().collect())
            }
            qsc::ast::PathKind::Err(None) => None,
        };
    }
}

impl IncompletePath<'_> {
    pub fn context(&self) -> Option<(PathKind, Vec<Rc<str>>)> {
        let context = self.context?;
        let qualifier = self.segments_before_offset();

        if qualifier.is_empty() {
            return None;
        }

        Some((context, qualifier))
    }

    fn segments_before_offset(&self) -> Vec<Rc<str>> {
        self.qualifier
            .iter()
            .flatten()
            .take_while(|i| i.span.hi < self.offset)
            .map(|i| i.name.clone())
            .collect::<Vec<_>>()
    }
}

/// A [`Visitor`] wrapper that only descends into a node
/// if the given offset falls within that node.
struct OffsetVisitor<T> {
    offset: u32,
    visitor: T,
}

impl<'a, T> Visitor<'a> for OffsetVisitor<T>
where
    T: Visitor<'a>,
{
    fn visit_namespace(&mut self, namespace: &'a Namespace) {
        if namespace.span.touches(self.offset) {
            self.visitor.visit_namespace(namespace);
            visit::walk_namespace(self, namespace);
        }
    }

    fn visit_item(&mut self, item: &'a Item) {
        if item.span.touches(self.offset) {
            self.visitor.visit_item(item);
            visit::walk_item(self, item);
        }
    }

    fn visit_attr(&mut self, attr: &'a Attr) {
        if attr.span.touches(self.offset) {
            self.visitor.visit_attr(attr);
            visit::walk_attr(self, attr);
        }
    }

    fn visit_ty_def(&mut self, def: &'a TyDef) {
        if def.span.touches(self.offset) {
            self.visitor.visit_ty_def(def);
            visit::walk_ty_def(self, def);
        }
    }

    fn visit_callable_decl(&mut self, decl: &'a CallableDecl) {
        if decl.span.touches(self.offset) {
            self.visitor.visit_callable_decl(decl);
            visit::walk_callable_decl(self, decl);
        }
    }

    fn visit_struct_decl(&mut self, decl: &'a StructDecl) {
        if decl.span.touches(self.offset) {
            self.visitor.visit_struct_decl(decl);
            visit::walk_struct_decl(self, decl);
        }
    }

    fn visit_field_def(&mut self, def: &'a FieldDef) {
        if def.span.touches(self.offset) {
            self.visitor.visit_field_def(def);
            visit::walk_field_def(self, def);
        }
    }

    fn visit_spec_decl(&mut self, decl: &'a SpecDecl) {
        if decl.span.touches(self.offset) {
            self.visitor.visit_spec_decl(decl);
            visit::walk_spec_decl(self, decl);
        }
    }

    fn visit_functor_expr(&mut self, expr: &'a FunctorExpr) {
        if expr.span.touches(self.offset) {
            self.visitor.visit_functor_expr(expr);
            visit::walk_functor_expr(self, expr);
        }
    }

    fn visit_ty(&mut self, ty: &'a Ty) {
        if ty.span.touches(self.offset) {
            self.visitor.visit_ty(ty);
            visit::walk_ty(self, ty);
        }
    }

    fn visit_block(&mut self, block: &'a Block) {
        if block.span.touches(self.offset) {
            self.visitor.visit_block(block);
            visit::walk_block(self, block);
        }
    }

    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        if stmt.span.touches(self.offset) {
            self.visitor.visit_stmt(stmt);
            visit::walk_stmt(self, stmt);
        }
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        if expr.span.touches(self.offset) {
            self.visitor.visit_expr(expr);
            visit::walk_expr(self, expr);
        }
    }

    fn visit_field_assign(&mut self, assign: &'a FieldAssign) {
        if assign.span.touches(self.offset) {
            self.visitor.visit_field_assign(assign);
            visit::walk_field_assign(self, assign);
        }
    }

    fn visit_pat(&mut self, pat: &'a Pat) {
        if pat.span.touches(self.offset) {
            self.visitor.visit_pat(pat);
            visit::walk_pat(self, pat);
        }
    }

    fn visit_qubit_init(&mut self, init: &'a QubitInit) {
        if init.span.touches(self.offset) {
            self.visitor.visit_qubit_init(init);
            visit::walk_qubit_init(self, init);
        }
    }

    fn visit_path(&mut self, path: &'a Path) {
        if path.span.touches(self.offset) {
            self.visitor.visit_path(path);
            visit::walk_path(self, path);
        }
    }

    fn visit_path_kind(&mut self, path: &'a qsc::ast::PathKind) {
        let span = match path {
            qsc::ast::PathKind::Ok(path) => &path.span,
            qsc::ast::PathKind::Err(Some(incomplete_path)) => &incomplete_path.span,
            qsc::ast::PathKind::Err(None) => return,
        };

        if span.touches(self.offset) {
            self.visitor.visit_path_kind(path);
            visit::walk_path_kind(self, path);
        }
    }

    fn visit_ident(&mut self, ident: &'a Ident) {
        if ident.span.touches(self.offset) {
            self.visitor.visit_ident(ident);
        }
    }
}
