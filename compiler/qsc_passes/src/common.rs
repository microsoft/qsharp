// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::span::Span;
use qsc_hir::{
    hir::{
        CallableKind, Expr, ExprKind, Functor, Ident, Item, ItemId, ItemKind, LocalItemId,
        Mutability, NodeId, Package, PackageId, Pat, PatKind, PrimField, PrimTy, Res, Stmt,
        StmtKind, Ty,
    },
    visit::Visitor,
};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub struct IdentTemplate {
    pub id: NodeId,
    pub span: Span,
    pub name: Rc<str>,
    pub ty: Ty,
}

impl IdentTemplate {
    pub fn gen_local_ref(&self) -> Expr {
        Expr {
            id: NodeId::default(),
            span: self.span,
            ty: self.ty.clone(),
            kind: ExprKind::Var(Res::Local(self.id)),
        }
    }

    fn gen_pat(&self) -> Pat {
        Pat {
            id: NodeId::default(),
            span: self.span,
            ty: self.ty.clone(),
            kind: PatKind::Bind(Ident {
                id: self.id,
                span: self.span,
                name: self.name.clone(),
            }),
        }
    }

    pub fn gen_field_access(&self, field: PrimField) -> Expr {
        Expr {
            id: NodeId::default(),
            span: self.span,
            ty: Ty::Prim(PrimTy::Int),
            kind: ExprKind::Field(Box::new(self.gen_local_ref()), field),
        }
    }

    pub fn gen_id_init(&self, mutability: Mutability, expr: Expr) -> Stmt {
        Stmt {
            id: NodeId::default(),
            span: self.span,
            kind: StmtKind::Local(mutability, self.gen_pat(), expr),
        }
    }
}

struct GetApiIds {
    pub map: HashMap<Rc<str>, LocalItemId>,
}

impl GetApiIds {
    pub fn new(base_package: &Package) -> GetApiIds {
        let mut visitor = GetApiIds {
            map: HashMap::new(),
        };
        visitor.visit_package(base_package);
        visitor
    }
}

impl<'a> Visitor<'a> for GetApiIds {
    fn visit_item(&mut self, item: &'a Item) {
        if let ItemKind::Callable(decl) = &item.kind {
            self.map.insert(decl.name.name.clone(), item.id);
            self.map = HashMap::new();
        }
    }
}

pub struct BuiltInApi {
    map: HashMap<Rc<str>, LocalItemId>,
}

impl BuiltInApi {
    pub fn new(base_package: &Package) -> BuiltInApi {
        BuiltInApi {
            map: GetApiIds::new(base_package).map,
        }
    }

    pub fn mock() -> BuiltInApi {
        let mut map = HashMap::new();
        map.insert(
            Rc::from("__quantum__rt__qubit_allocate"),
            LocalItemId::from(140),
        );
        map.insert(
            Rc::from("__quantum__rt__qubit_release"),
            LocalItemId::from(141),
        );
        map.insert(
            Rc::from("__quantum__rt__qubit_allocate_array"),
            LocalItemId::from(142),
        );
        map.insert(
            Rc::from("__quantum__rt__qubit_release_array"),
            LocalItemId::from(143),
        );
        BuiltInApi { map }
    }

    pub fn __quantum__rt__qubit_allocate(&self, span: Span) -> Expr {
        let id = self
            .map
            .get("__quantum__rt__qubit_allocate")
            .expect("Cannot find function __quantum__rt__qubit_allocate");
        Expr {
            id: NodeId::default(),
            span,
            ty: Ty::Arrow(
                CallableKind::Function,
                Box::new(Ty::UNIT),
                Box::new(Ty::Prim(PrimTy::Qubit)),
                HashSet::<Functor>::new(),
            ),
            kind: ExprKind::Var(Res::Item(ItemId {
                package: Some(PackageId::from(0)),
                item: *id,
            })),
        }
    }

    pub fn __quantum__rt__qubit_release(&self, span: Span) -> Expr {
        let id = self
            .map
            .get("__quantum__rt__qubit_release")
            .expect("Cannot find function __quantum__rt__qubit_release");
        Expr {
            id: NodeId::default(),
            span,
            ty: Ty::Arrow(
                CallableKind::Function,
                Box::new(Ty::Prim(PrimTy::Qubit)),
                Box::new(Ty::UNIT),
                HashSet::<Functor>::new(),
            ),
            kind: ExprKind::Var(Res::Item(ItemId {
                package: Some(PackageId::from(0)),
                item: *id,
            })),
        }
    }

    pub fn __quantum__rt__qubit_allocate_array(&self, span: Span) -> Expr {
        let id = self
            .map
            .get("__quantum__rt__qubit_allocate_array")
            .expect("Cannot find function __quantum__rt__qubit_allocate_array");
        Expr {
            id: NodeId::default(),
            span,
            ty: Ty::Arrow(
                CallableKind::Function,
                Box::new(Ty::Prim(PrimTy::Int)),
                Box::new(Ty::Array(Box::new(Ty::Prim(PrimTy::Qubit)))),
                HashSet::<Functor>::new(),
            ),
            kind: ExprKind::Var(Res::Item(ItemId {
                package: Some(PackageId::from(0)),
                item: *id,
            })),
        }
    }

    pub fn __quantum__rt__qubit_release_array(&self, span: Span) -> Expr {
        let id = self
            .map
            .get("__quantum__rt__qubit_release_array")
            .expect("Cannot find function __quantum__rt__qubit_release_array");
        Expr {
            id: NodeId::default(),
            span,
            ty: Ty::Arrow(
                CallableKind::Function,
                Box::new(Ty::Array(Box::new(Ty::Prim(PrimTy::Qubit)))),
                Box::new(Ty::UNIT),
                HashSet::<Functor>::new(),
            ),
            kind: ExprKind::Var(Res::Item(ItemId {
                package: Some(PackageId::from(0)),
                item: *id,
            })),
        }
    }
}
