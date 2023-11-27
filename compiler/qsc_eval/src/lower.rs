// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::debug::map_hir_package_to_fir;
use qsc_data_structures::index_map::IndexMap;
use qsc_fir::assigner::Assigner;
use qsc_fir::fir::{Block, Expr, Pat, Stmt};
use qsc_fir::{
    fir::{self, BlockId, ExprId, LocalItemId, PatId, StmtId},
    ty::{Arrow, InferFunctorId, ParamId, Ty},
};
use qsc_frontend::compile::PackageStore;
use qsc_hir::hir;
use std::{clone::Clone, rc::Rc};

pub struct Lowerer {
    nodes: IndexMap<hir::NodeId, fir::NodeId>,
    exprs: IndexMap<ExprId, Expr>,
    pats: IndexMap<PatId, Pat>,
    stmts: IndexMap<StmtId, Stmt>,
    blocks: IndexMap<BlockId, Block>,
    assigner: Assigner,
}

impl Default for Lowerer {
    fn default() -> Self {
        Self::new()
    }
}

impl Lowerer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: IndexMap::new(),
            exprs: IndexMap::new(),
            pats: IndexMap::new(),
            stmts: IndexMap::new(),
            blocks: IndexMap::new(),
            assigner: Assigner::new(),
        }
    }

    pub fn lower_store(&mut self, store: &PackageStore) -> fir::PackageStore {
        let mut fir_store = IndexMap::new();
        for (id, unit) in store.iter() {
            fir_store.insert(
                map_hir_package_to_fir(id),
                self.lower_package(&unit.package),
            );
        }
        fir::PackageStore(fir_store)
    }

    pub fn lower_package(&mut self, package: &hir::Package) -> fir::Package {
        let entry = package.entry.as_ref().map(|e| self.lower_expr(e));
        let items: IndexMap<LocalItemId, fir::Item> = package
            .items
            .values()
            .map(|i| self.lower_item(i))
            .map(|i| (i.id, i))
            .collect();

        // Lower top-level statements
        for stmt in &package.stmts {
            self.lower_stmt(stmt);
        }

        let blocks: IndexMap<_, _> = self.blocks.drain().collect();
        let exprs: IndexMap<_, _> = self.exprs.drain().collect();
        let pats: IndexMap<_, _> = self.pats.drain().collect();
        let stmts: IndexMap<_, _> = self.stmts.drain().collect();

        let package = fir::Package {
            items,
            entry,
            blocks,
            exprs,
            pats,
            stmts,
        };
        qsc_fir::validate::validate(&package);
        package
    }

    /// Used to update the package with the lowered items.
    /// Incremental compilation requires that we update the package
    /// instead of returning a new one.
    #[allow(clippy::similar_names)]
    pub fn lower_and_update_package(
        &mut self,
        fir_package: &mut fir::Package,
        hir_package: &hir::Package,
    ) -> Vec<StmtId> {
        let items: IndexMap<LocalItemId, fir::Item> = hir_package
            .items
            .values()
            .map(|i| self.lower_item(i))
            .map(|i| (i.id, i))
            .collect();

        let new_stmts = hir_package
            .stmts
            .iter()
            .map(|s| self.lower_stmt(s))
            .collect();

        self.update_package(fir_package);

        for (k, v) in items {
            fir_package.items.insert(k, v);
        }

        qsc_fir::validate::validate(fir_package);

        new_stmts
    }

    fn update_package(&mut self, package: &mut fir::Package) {
        for (id, value) in self.blocks.drain() {
            package.blocks.insert(id, value);
        }

        for (id, value) in self.exprs.drain() {
            package.exprs.insert(id, value);
        }

        for (id, value) in self.pats.drain() {
            package.pats.insert(id, value);
        }

        for (id, value) in self.stmts.drain() {
            package.stmts.insert(id, value);
        }
    }

    fn lower_item(&mut self, item: &hir::Item) -> fir::Item {
        let kind = match &item.kind {
            hir::ItemKind::Namespace(name, items) => {
                let name = self.lower_ident(name);
                let items = items.iter().map(|i| lower_local_item_id(*i)).collect();
                fir::ItemKind::Namespace(name, items)
            }
            hir::ItemKind::Callable(callable) => {
                let callable = self.lower_callable_decl(callable);

                fir::ItemKind::Callable(callable)
            }
            hir::ItemKind::Ty(name, udt) => {
                let name = self.lower_ident(name);
                let udt = self.lower_udt(udt);

                fir::ItemKind::Ty(name, udt)
            }
        };
        let attrs = lower_attrs(&item.attrs);
        fir::Item {
            id: lower_local_item_id(item.id),
            span: item.span,
            parent: item.parent.map(lower_local_item_id),
            doc: Rc::clone(&item.doc),
            attrs,
            visibility: lower_visibility(item.visibility),
            kind,
        }
    }

    fn lower_callable_decl(&mut self, decl: &hir::CallableDecl) -> fir::CallableDecl {
        let id = self.lower_id(decl.id);
        let kind = lower_callable_kind(decl.kind);
        let name = self.lower_ident(&decl.name);
        let input = self.lower_pat(&decl.input);

        let generics = lower_generics(&decl.generics);
        let output = self.lower_ty(&decl.output);
        let functors = lower_functors(decl.functors);
        let body = self.lower_spec_decl(&decl.body);
        let adj = decl.adj.as_ref().map(|f| self.lower_spec_decl(f));
        let ctl = decl.ctl.as_ref().map(|f| self.lower_spec_decl(f));
        let ctl_adj = decl.ctl_adj.as_ref().map(|f| self.lower_spec_decl(f));

        fir::CallableDecl {
            id,
            span: decl.span,
            kind,
            name,
            generics,
            input,
            output,
            functors,
            body,
            adj,
            ctl,
            ctl_adj,
        }
    }

    fn lower_spec_decl(&mut self, decl: &hir::SpecDecl) -> fir::SpecDecl {
        fir::SpecDecl {
            id: self.lower_id(decl.id),
            span: decl.span,
            body: match &decl.body {
                hir::SpecBody::Gen(gen) => fir::SpecBody::Gen(match gen {
                    hir::SpecGen::Auto => fir::SpecGen::Auto,
                    hir::SpecGen::Distribute => fir::SpecGen::Distribute,
                    hir::SpecGen::Intrinsic => fir::SpecGen::Intrinsic,
                    hir::SpecGen::Invert => fir::SpecGen::Invert,
                    hir::SpecGen::Slf => fir::SpecGen::Slf,
                }),
                hir::SpecBody::Impl(input, block) => fir::SpecBody::Impl(
                    input.as_ref().map(|i| self.lower_spec_decl_pat(i)),
                    self.lower_block(block),
                ),
            },
        }
    }

    fn lower_spec_decl_pat(&mut self, pat: &hir::Pat) -> PatId {
        let id = self.assigner.next_pat();
        let span = pat.span;
        let ty = self.lower_ty(&pat.ty);

        let kind = match &pat.kind {
            hir::PatKind::Bind(ident) => fir::PatKind::Bind(self.lower_ident(ident)),
            hir::PatKind::Discard => fir::PatKind::Discard,
            hir::PatKind::Tuple(elems) => {
                fir::PatKind::Tuple(elems.iter().map(|pat| self.lower_pat(pat)).collect())
            }
            hir::PatKind::Err => unreachable!("error pat should not be present"),
        };

        let pat = fir::Pat { id, span, ty, kind };
        self.pats.insert(id, pat);
        id
    }

    fn lower_block(&mut self, block: &hir::Block) -> BlockId {
        let id = self.assigner.next_block();
        let block = fir::Block {
            id,
            span: block.span,
            ty: self.lower_ty(&block.ty),
            stmts: block.stmts.iter().map(|s| self.lower_stmt(s)).collect(),
        };
        self.blocks.insert(id, block);
        id
    }

    fn lower_stmt(&mut self, stmt: &hir::Stmt) -> fir::StmtId {
        let id = self.assigner.next_stmt();
        let kind = match &stmt.kind {
            hir::StmtKind::Expr(expr) => fir::StmtKind::Expr(self.lower_expr(expr)),
            hir::StmtKind::Item(item) => fir::StmtKind::Item(lower_local_item_id(*item)),
            hir::StmtKind::Local(mutability, pat, expr) => fir::StmtKind::Local(
                lower_mutability(*mutability),
                self.lower_pat(pat),
                self.lower_expr(expr),
            ),
            hir::StmtKind::Qubit(source, pat, init, block) => fir::StmtKind::Qubit(
                lower_qubit_source(*source),
                self.lower_pat(pat),
                self.lower_qubit_init(init),
                block.as_ref().map(|b| self.lower_block(b)),
            ),
            hir::StmtKind::Semi(expr) => fir::StmtKind::Semi(self.lower_expr(expr)),
        };
        let stmt = fir::Stmt {
            id,
            span: stmt.span,
            kind,
        };
        self.stmts.insert(id, stmt);
        id
    }

    #[allow(clippy::too_many_lines)]
    fn lower_expr(&mut self, expr: &hir::Expr) -> ExprId {
        let id = self.assigner.next_expr();
        let ty = self.lower_ty(&expr.ty);

        let kind = match &expr.kind {
            hir::ExprKind::Array(items) => {
                fir::ExprKind::Array(items.iter().map(|i| self.lower_expr(i)).collect())
            }
            hir::ExprKind::ArrayRepeat(value, size) => {
                fir::ExprKind::ArrayRepeat(self.lower_expr(value), self.lower_expr(size))
            }
            hir::ExprKind::Assign(lhs, rhs) => {
                fir::ExprKind::Assign(self.lower_expr(lhs), self.lower_expr(rhs))
            }
            hir::ExprKind::AssignOp(op, lhs, rhs) => fir::ExprKind::AssignOp(
                lower_binop(*op),
                self.lower_expr(lhs),
                self.lower_expr(rhs),
            ),
            hir::ExprKind::AssignField(container, field, replace) => {
                let container = self.lower_expr(container);
                let field = lower_field(field);
                let replace = self.lower_expr(replace);
                fir::ExprKind::AssignField(container, field, replace)
            }
            hir::ExprKind::AssignIndex(container, index, replace) => fir::ExprKind::AssignIndex(
                self.lower_expr(container),
                self.lower_expr(index),
                self.lower_expr(replace),
            ),
            hir::ExprKind::BinOp(op, lhs, rhs) => {
                fir::ExprKind::BinOp(lower_binop(*op), self.lower_expr(lhs), self.lower_expr(rhs))
            }
            hir::ExprKind::Block(block) => fir::ExprKind::Block(self.lower_block(block)),
            hir::ExprKind::Call(callee, arg) => {
                fir::ExprKind::Call(self.lower_expr(callee), self.lower_expr(arg))
            }
            hir::ExprKind::Fail(message) => fir::ExprKind::Fail(self.lower_expr(message)),
            hir::ExprKind::Field(container, field) => {
                let container = self.lower_expr(container);
                let field = lower_field(field);
                fir::ExprKind::Field(container, field)
            }
            hir::ExprKind::If(cond, if_true, if_false) => fir::ExprKind::If(
                self.lower_expr(cond),
                self.lower_expr(if_true),
                if_false.as_ref().map(|e| self.lower_expr(e)),
            ),
            hir::ExprKind::Index(container, index) => {
                fir::ExprKind::Index(self.lower_expr(container), self.lower_expr(index))
            }
            hir::ExprKind::Lit(lit) => lower_lit(lit),
            hir::ExprKind::Range(start, step, end) => fir::ExprKind::Range(
                start.as_ref().map(|s| self.lower_expr(s)),
                step.as_ref().map(|s| self.lower_expr(s)),
                end.as_ref().map(|e| self.lower_expr(e)),
            ),
            hir::ExprKind::Return(expr) => fir::ExprKind::Return(self.lower_expr(expr)),
            hir::ExprKind::Tuple(items) => {
                fir::ExprKind::Tuple(items.iter().map(|i| self.lower_expr(i)).collect())
            }
            hir::ExprKind::UnOp(op, operand) => {
                fir::ExprKind::UnOp(lower_unop(*op), self.lower_expr(operand))
            }
            hir::ExprKind::While(cond, body) => {
                fir::ExprKind::While(self.lower_expr(cond), self.lower_block(body))
            }
            hir::ExprKind::Closure(ids, id) => {
                let ids = ids.iter().map(|id| self.lower_id(*id)).collect();
                fir::ExprKind::Closure(ids, lower_local_item_id(*id))
            }
            hir::ExprKind::String(components) => fir::ExprKind::String(
                components
                    .iter()
                    .map(|c| self.lower_string_component(c))
                    .collect(),
            ),
            hir::ExprKind::UpdateIndex(expr1, expr2, expr3) => fir::ExprKind::UpdateIndex(
                self.lower_expr(expr1),
                self.lower_expr(expr2),
                self.lower_expr(expr3),
            ),
            hir::ExprKind::UpdateField(record, field, replace) => {
                let record = self.lower_expr(record);
                let field = lower_field(field);
                let replace = self.lower_expr(replace);
                fir::ExprKind::UpdateField(record, field, replace)
            }
            hir::ExprKind::Var(res, args) => {
                let res = self.lower_res(res);
                let args = args.iter().map(|arg| self.lower_generic_arg(arg)).collect();
                fir::ExprKind::Var(res, args)
            }
            hir::ExprKind::Conjugate(..) => panic!("conjugate should be eliminated by passes"),
            hir::ExprKind::Err => panic!("error expr should not be present"),
            hir::ExprKind::For(..) => panic!("for-loop should be eliminated by passes"),
            hir::ExprKind::Hole => fir::ExprKind::Hole, // allowed for discards
            hir::ExprKind::Repeat(..) => panic!("repeat-loop should be eliminated by passes"),
        };

        let expr = fir::Expr {
            id,
            span: expr.span,
            ty,
            kind,
        };
        self.exprs.insert(id, expr);
        id
    }

    fn lower_string_component(&mut self, component: &hir::StringComponent) -> fir::StringComponent {
        match component {
            hir::StringComponent::Expr(expr) => fir::StringComponent::Expr(self.lower_expr(expr)),
            hir::StringComponent::Lit(str) => fir::StringComponent::Lit(Rc::clone(str)),
        }
    }

    fn lower_pat(&mut self, pat: &hir::Pat) -> PatId {
        let id = self.assigner.next_pat();
        let ty = self.lower_ty(&pat.ty);
        let kind = match &pat.kind {
            hir::PatKind::Bind(name) => {
                let name = self.lower_ident(name);
                fir::PatKind::Bind(name)
            }
            hir::PatKind::Discard => fir::PatKind::Discard,
            hir::PatKind::Tuple(items) => {
                fir::PatKind::Tuple(items.iter().map(|i| self.lower_pat(i)).collect())
            }
            hir::PatKind::Err => unreachable!("error pat should not be present"),
        };

        let pat = fir::Pat {
            id,
            span: pat.span,
            ty,
            kind,
        };
        self.pats.insert(id, pat);
        id
    }

    fn lower_qubit_init(&mut self, init: &hir::QubitInit) -> fir::QubitInit {
        let id = self.lower_id(init.id);
        let ty = self.lower_ty(&init.ty);
        let kind = match &init.kind {
            hir::QubitInitKind::Array(length) => fir::QubitInitKind::Array(self.lower_expr(length)),
            hir::QubitInitKind::Single => fir::QubitInitKind::Single,
            hir::QubitInitKind::Tuple(items) => {
                fir::QubitInitKind::Tuple(items.iter().map(|i| self.lower_qubit_init(i)).collect())
            }
            hir::QubitInitKind::Err => unreachable!("error qubit init should not be present"),
        };

        fir::QubitInit {
            id,
            span: init.span,
            ty,
            kind,
        }
    }

    fn lower_id(&mut self, id: hir::NodeId) -> fir::NodeId {
        self.nodes.get(id).copied().unwrap_or_else(|| {
            let new_id = self.assigner.next_node();
            self.nodes.insert(id, new_id);
            new_id
        })
    }

    fn lower_res(&mut self, res: &hir::Res) -> fir::Res {
        match res {
            hir::Res::Item(item) => fir::Res::Item(lower_item_id(item)),
            hir::Res::Local(node) => fir::Res::Local(self.lower_id(*node)),
            hir::Res::Err => fir::Res::Err,
        }
    }

    fn lower_ident(&mut self, ident: &hir::Ident) -> fir::Ident {
        fir::Ident {
            id: self.lower_id(ident.id),
            span: ident.span,
            name: ident.name.clone(),
        }
    }

    fn lower_udt(&mut self, udt: &qsc_hir::ty::Udt) -> qsc_fir::ty::Udt {
        let span = udt.span;
        let name = udt.name.clone();
        let definition = self.lower_udt_defn(&udt.definition);
        qsc_fir::ty::Udt {
            span,
            name,
            definition,
        }
    }

    fn lower_generic_arg(&mut self, arg: &qsc_hir::ty::GenericArg) -> qsc_fir::ty::GenericArg {
        match &arg {
            qsc_hir::ty::GenericArg::Ty(ty) => qsc_fir::ty::GenericArg::Ty(self.lower_ty(ty)),
            qsc_hir::ty::GenericArg::Functor(functors) => {
                qsc_fir::ty::GenericArg::Functor(lower_functor_set(functors))
            }
        }
    }

    fn lower_udt_defn(&mut self, definition: &qsc_hir::ty::UdtDef) -> qsc_fir::ty::UdtDef {
        let span = definition.span;
        let kind = match &definition.kind {
            qsc_hir::ty::UdtDefKind::Field(field) => {
                qsc_fir::ty::UdtDefKind::Field(self.lower_udt_field(field))
            }
            qsc_hir::ty::UdtDefKind::Tuple(tup) => qsc_fir::ty::UdtDefKind::Tuple(
                tup.iter().map(|def| self.lower_udt_defn(def)).collect(),
            ),
        };
        qsc_fir::ty::UdtDef { span, kind }
    }

    fn lower_arrow(&mut self, arrow: &qsc_hir::ty::Arrow) -> Arrow {
        Arrow {
            kind: lower_callable_kind(arrow.kind),
            input: Box::new(self.lower_ty(&arrow.input)),
            output: Box::new(self.lower_ty(&arrow.output)),
            functors: lower_functor_set(&arrow.functors),
        }
    }

    fn lower_ty(&mut self, ty: &qsc_hir::ty::Ty) -> Ty {
        match ty {
            qsc_hir::ty::Ty::Array(array) => qsc_fir::ty::Ty::Array(Box::new(self.lower_ty(array))),
            qsc_hir::ty::Ty::Arrow(arrow) => {
                qsc_fir::ty::Ty::Arrow(Box::new(self.lower_arrow(arrow)))
            }
            qsc_hir::ty::Ty::Infer(id) => {
                qsc_fir::ty::Ty::Infer(qsc_fir::ty::InferTyId::from(usize::from(*id)))
            }
            qsc_hir::ty::Ty::Param(id) => {
                qsc_fir::ty::Ty::Param(qsc_fir::ty::ParamId::from(usize::from(*id)))
            }
            qsc_hir::ty::Ty::Prim(prim) => qsc_fir::ty::Ty::Prim(lower_ty_prim(*prim)),
            qsc_hir::ty::Ty::Tuple(tys) => {
                qsc_fir::ty::Ty::Tuple(tys.iter().map(|ty| self.lower_ty(ty)).collect())
            }
            qsc_hir::ty::Ty::Udt(res) => qsc_fir::ty::Ty::Udt(self.lower_res(res)),
            qsc_hir::ty::Ty::Err => qsc_fir::ty::Ty::Err,
        }
    }

    fn lower_udt_field(&mut self, field: &qsc_hir::ty::UdtField) -> qsc_fir::ty::UdtField {
        qsc_fir::ty::UdtField {
            ty: self.lower_ty(&field.ty),
            name: field.name.clone(),
            name_span: field.name_span,
        }
    }
}

fn lower_generics(generics: &[qsc_hir::ty::GenericParam]) -> Vec<qsc_fir::ty::GenericParam> {
    generics.iter().map(lower_generic_param).collect()
}

fn lower_attrs(attrs: &[hir::Attr]) -> Vec<fir::Attr> {
    attrs.iter().map(|_| fir::Attr::EntryPoint).collect()
}

fn lower_functors(functors: qsc_hir::ty::FunctorSetValue) -> qsc_fir::ty::FunctorSetValue {
    lower_functor_set_value(functors)
}

fn lower_generic_param(g: &qsc_hir::ty::GenericParam) -> qsc_fir::ty::GenericParam {
    match g {
        qsc_hir::ty::GenericParam::Ty => qsc_fir::ty::GenericParam::Ty,
        qsc_hir::ty::GenericParam::Functor(value) => {
            qsc_fir::ty::GenericParam::Functor(lower_functor_set_value(*value))
        }
    }
}

fn lower_field(field: &hir::Field) -> fir::Field {
    match field {
        hir::Field::Err => fir::Field::Err,
        hir::Field::Path(path) => fir::Field::Path(fir::FieldPath {
            indices: path.indices.clone(),
        }),
        hir::Field::Prim(field) => fir::Field::Prim(lower_prim_field(*field)),
    }
}

fn lower_functor_set(functors: &qsc_hir::ty::FunctorSet) -> qsc_fir::ty::FunctorSet {
    match *functors {
        qsc_hir::ty::FunctorSet::Value(v) => {
            qsc_fir::ty::FunctorSet::Value(lower_functor_set_value(v))
        }
        qsc_hir::ty::FunctorSet::Param(p) => {
            qsc_fir::ty::FunctorSet::Param(ParamId::from(usize::from(p)))
        }
        qsc_hir::ty::FunctorSet::Infer(i) => {
            qsc_fir::ty::FunctorSet::Infer(InferFunctorId::from(usize::from(i)))
        }
    }
}

fn lower_prim_field(field: hir::PrimField) -> fir::PrimField {
    match field {
        hir::PrimField::Start => fir::PrimField::Start,
        hir::PrimField::Step => fir::PrimField::Step,
        hir::PrimField::End => fir::PrimField::End,
    }
}

fn lower_item_id(id: &hir::ItemId) -> fir::ItemId {
    fir::ItemId {
        item: lower_local_item_id(id.item),
        package: id.package.map(|p| fir::PackageId::from(usize::from(p))),
    }
}

fn lower_ty_prim(prim: qsc_hir::ty::Prim) -> qsc_fir::ty::Prim {
    match prim {
        qsc_hir::ty::Prim::Bool => qsc_fir::ty::Prim::Bool,
        qsc_hir::ty::Prim::Double => qsc_fir::ty::Prim::Double,
        qsc_hir::ty::Prim::Int => qsc_fir::ty::Prim::Int,
        qsc_hir::ty::Prim::Qubit => qsc_fir::ty::Prim::Qubit,
        qsc_hir::ty::Prim::Result => qsc_fir::ty::Prim::Result,
        qsc_hir::ty::Prim::String => qsc_fir::ty::Prim::String,
        qsc_hir::ty::Prim::BigInt => qsc_fir::ty::Prim::BigInt,
        qsc_hir::ty::Prim::RangeTo => qsc_fir::ty::Prim::RangeTo,
        qsc_hir::ty::Prim::RangeFrom => qsc_fir::ty::Prim::RangeFrom,
        qsc_hir::ty::Prim::Pauli => qsc_fir::ty::Prim::Pauli,
        qsc_hir::ty::Prim::RangeFull => qsc_fir::ty::Prim::RangeFull,
        qsc_hir::ty::Prim::Range => qsc_fir::ty::Prim::Range,
    }
}

fn lower_visibility(visibility: hir::Visibility) -> fir::Visibility {
    match visibility {
        hir::Visibility::Public => fir::Visibility::Public,
        hir::Visibility::Internal => fir::Visibility::Internal,
    }
}

fn lower_callable_kind(kind: hir::CallableKind) -> fir::CallableKind {
    match kind {
        hir::CallableKind::Function => fir::CallableKind::Function,
        hir::CallableKind::Operation => fir::CallableKind::Operation,
    }
}

fn lower_mutability(mutability: hir::Mutability) -> fir::Mutability {
    match mutability {
        hir::Mutability::Immutable => fir::Mutability::Immutable,
        hir::Mutability::Mutable => fir::Mutability::Mutable,
    }
}

fn lower_unop(op: hir::UnOp) -> fir::UnOp {
    match op {
        hir::UnOp::Functor(f) => fir::UnOp::Functor(lower_functor(f)),
        hir::UnOp::Neg => fir::UnOp::Neg,
        hir::UnOp::NotB => fir::UnOp::NotB,
        hir::UnOp::NotL => fir::UnOp::NotL,
        hir::UnOp::Pos => fir::UnOp::Pos,
        hir::UnOp::Unwrap => fir::UnOp::Unwrap,
    }
}

fn lower_binop(op: hir::BinOp) -> fir::BinOp {
    match op {
        hir::BinOp::Add => fir::BinOp::Add,
        hir::BinOp::AndB => fir::BinOp::AndB,
        hir::BinOp::AndL => fir::BinOp::AndL,
        hir::BinOp::Div => fir::BinOp::Div,
        hir::BinOp::Eq => fir::BinOp::Eq,
        hir::BinOp::Exp => fir::BinOp::Exp,
        hir::BinOp::Gt => fir::BinOp::Gt,
        hir::BinOp::Gte => fir::BinOp::Gte,
        hir::BinOp::Lt => fir::BinOp::Lt,
        hir::BinOp::Lte => fir::BinOp::Lte,
        hir::BinOp::Mod => fir::BinOp::Mod,
        hir::BinOp::Mul => fir::BinOp::Mul,
        hir::BinOp::Neq => fir::BinOp::Neq,
        hir::BinOp::OrB => fir::BinOp::OrB,
        hir::BinOp::OrL => fir::BinOp::OrL,
        hir::BinOp::Shl => fir::BinOp::Shl,
        hir::BinOp::Shr => fir::BinOp::Shr,
        hir::BinOp::Sub => fir::BinOp::Sub,
        hir::BinOp::XorB => fir::BinOp::XorB,
    }
}

fn lower_lit(lit: &hir::Lit) -> fir::ExprKind {
    match lit {
        hir::Lit::BigInt(value) => fir::ExprKind::Lit(fir::Lit::BigInt(value.clone())),
        &hir::Lit::Bool(value) => fir::ExprKind::Lit(fir::Lit::Bool(value)),
        &hir::Lit::Double(value) => fir::ExprKind::Lit(fir::Lit::Double(value)),
        &hir::Lit::Int(value) => fir::ExprKind::Lit(fir::Lit::Int(value)),
        hir::Lit::Pauli(hir::Pauli::I) => fir::ExprKind::Lit(fir::Lit::Pauli(fir::Pauli::I)),
        hir::Lit::Pauli(hir::Pauli::X) => fir::ExprKind::Lit(fir::Lit::Pauli(fir::Pauli::X)),
        hir::Lit::Pauli(hir::Pauli::Y) => fir::ExprKind::Lit(fir::Lit::Pauli(fir::Pauli::Y)),
        hir::Lit::Pauli(hir::Pauli::Z) => fir::ExprKind::Lit(fir::Lit::Pauli(fir::Pauli::Z)),
        hir::Lit::Result(hir::Result::One) => {
            fir::ExprKind::Lit(fir::Lit::Result(fir::Result::One))
        }
        hir::Lit::Result(hir::Result::Zero) => {
            fir::ExprKind::Lit(fir::Lit::Result(fir::Result::Zero))
        }
    }
}

fn lower_functor(functor: hir::Functor) -> fir::Functor {
    match functor {
        hir::Functor::Adj => fir::Functor::Adj,
        hir::Functor::Ctl => fir::Functor::Ctl,
    }
}

fn lower_qubit_source(functor: hir::QubitSource) -> fir::QubitSource {
    match functor {
        hir::QubitSource::Dirty => fir::QubitSource::Dirty,
        hir::QubitSource::Fresh => fir::QubitSource::Fresh,
    }
}

fn lower_functor_set_value(value: qsc_hir::ty::FunctorSetValue) -> qsc_fir::ty::FunctorSetValue {
    match value {
        qsc_hir::ty::FunctorSetValue::Empty => qsc_fir::ty::FunctorSetValue::Empty,
        qsc_hir::ty::FunctorSetValue::Adj => qsc_fir::ty::FunctorSetValue::Adj,
        qsc_hir::ty::FunctorSetValue::Ctl => qsc_fir::ty::FunctorSetValue::Ctl,
        qsc_hir::ty::FunctorSetValue::CtlAdj => qsc_fir::ty::FunctorSetValue::CtlAdj,
    }
}

#[must_use]
#[allow(clippy::module_name_repetitions)]
fn lower_local_item_id(id: qsc_hir::hir::LocalItemId) -> LocalItemId {
    LocalItemId::from(usize::from(id))
}
