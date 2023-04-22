// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::resolve::{self, Resolutions};
use qsc_ast::ast;
use qsc_data_structures::index_map::IndexMap;
use qsc_hir::{
    assigner::Assigner,
    hir::{self, PackageDefId},
};

pub(super) struct Lowerer {
    assigner: Assigner,
    nodes: IndexMap<ast::NodeId, hir::NodeId>,
}

impl Lowerer {
    pub(super) fn new() -> Self {
        Self {
            assigner: Assigner::new(),
            nodes: IndexMap::new(),
        }
    }

    pub(super) fn with<'a>(&'a mut self, resolutions: &'a Resolutions) -> With {
        With {
            lowerer: self,
            resolutions,
        }
    }

    pub(super) fn get_id(&self, id: ast::NodeId) -> Option<hir::NodeId> {
        self.nodes.get(id).copied()
    }

    pub(super) fn into_assigner(self) -> Assigner {
        self.assigner
    }
}
pub(super) struct With<'a> {
    lowerer: &'a mut Lowerer,
    resolutions: &'a Resolutions,
}

impl With<'_> {
    pub(super) fn lower_package(&mut self, package: &ast::Package) -> hir::Package {
        let mut items = IndexMap::new();
        for namespace in &package.namespaces {
            let Some(&resolve::Res::Def(parent)) = self.resolutions.get(namespace.name.id) else {
                panic!("namespace should resolve to definition ID");
            };

            let mut namespace_items = Vec::new();

            for item in &namespace.items {
                if let Some((def, item)) = self.lower_item(parent.def, item) {
                    namespace_items.push(def);
                    items.insert(def, item);
                }
            }

            items.insert(
                parent.def,
                hir::Item {
                    id: self.lower_id(namespace.id),
                    span: namespace.span,
                    parent: None,
                    attrs: Vec::new(),
                    visibility: None,
                    kind: hir::ItemKind::Namespace(
                        self.lower_ident(&namespace.name),
                        namespace_items,
                    ),
                },
            );
        }

        hir::Package {
            id: self.lower_id(package.id),
            items,
            entry: package.entry.as_ref().map(|e| self.lower_expr(e)),
        }
    }

    fn lower_item(
        &mut self,
        parent: PackageDefId,
        item: &ast::Item,
    ) -> Option<(PackageDefId, hir::Item)> {
        if matches!(item.kind, ast::ItemKind::Open(..)) {
            return None;
        }

        let id = self.lower_id(item.id);
        let attrs = item.attrs.iter().map(|a| self.lower_attr(a)).collect();
        let visibility = item.visibility.as_ref().map(|v| self.lower_visibility(v));
        let (def, kind) = match &item.kind {
            ast::ItemKind::Callable(decl) => {
                let Some(&resolve::Res::Def(def_id)) = self.resolutions.get(decl.name.id) else {
                    panic!("callable declaration should have definition ID");
                };
                let kind = hir::ItemKind::Callable(self.lower_callable_decl(decl));
                (def_id.def, kind)
            }
            ast::ItemKind::Err | ast::ItemKind::Open(..) => return None,
            ast::ItemKind::Ty(name, def) => {
                let Some(&resolve::Res::Def(def_id)) = self.resolutions.get(name.id) else {
                    panic!("type declaration should have definition ID");
                };
                let kind = hir::ItemKind::Ty(self.lower_ident(name), self.lower_ty_def(def));
                (def_id.def, kind)
            }
        };

        Some((
            def,
            hir::Item {
                id,
                span: item.span,
                parent: Some(parent),
                attrs,
                visibility,
                kind,
            },
        ))
    }

    fn lower_attr(&mut self, attr: &ast::Attr) -> hir::Attr {
        hir::Attr {
            id: self.lower_id(attr.id),
            span: attr.span,
            name: self.lower_ident(&attr.name),
            arg: self.lower_expr(&attr.arg),
        }
    }

    fn lower_visibility(&mut self, visibility: &ast::Visibility) -> hir::Visibility {
        hir::Visibility {
            id: self.lower_id(visibility.id),
            span: visibility.span,
            kind: match visibility.kind {
                ast::VisibilityKind::Public => hir::VisibilityKind::Public,
                ast::VisibilityKind::Internal => hir::VisibilityKind::Internal,
            },
        }
    }

    pub(super) fn lower_callable_decl(&mut self, decl: &ast::CallableDecl) -> hir::CallableDecl {
        hir::CallableDecl {
            id: self.lower_id(decl.id),
            span: decl.span,
            kind: lower_callable_kind(decl.kind),
            name: self.lower_ident(&decl.name),
            ty_params: decl.ty_params.iter().map(|p| self.lower_ident(p)).collect(),
            input: self.lower_pat(&decl.input),
            output: self.lower_ty(&decl.output),
            functors: decl.functors.as_ref().map(|f| self.lower_functor_expr(f)),
            body: match &decl.body {
                ast::CallableBody::Block(block) => {
                    hir::CallableBody::Block(self.lower_block(block))
                }
                ast::CallableBody::Specs(specs) => hir::CallableBody::Specs(
                    specs.iter().map(|s| self.lower_spec_decl(s)).collect(),
                ),
            },
        }
    }

    fn lower_spec_decl(&mut self, decl: &ast::SpecDecl) -> hir::SpecDecl {
        hir::SpecDecl {
            id: self.lower_id(decl.id),
            span: decl.span,
            spec: match decl.spec {
                ast::Spec::Body => hir::Spec::Body,
                ast::Spec::Adj => hir::Spec::Adj,
                ast::Spec::Ctl => hir::Spec::Ctl,
                ast::Spec::CtlAdj => hir::Spec::CtlAdj,
            },
            body: match &decl.body {
                ast::SpecBody::Gen(gen) => hir::SpecBody::Gen(match gen {
                    ast::SpecGen::Auto => hir::SpecGen::Auto,
                    ast::SpecGen::Distribute => hir::SpecGen::Distribute,
                    ast::SpecGen::Intrinsic => hir::SpecGen::Intrinsic,
                    ast::SpecGen::Invert => hir::SpecGen::Invert,
                    ast::SpecGen::Slf => hir::SpecGen::Slf,
                }),
                ast::SpecBody::Impl(input, block) => {
                    hir::SpecBody::Impl(self.lower_pat(input), self.lower_block(block))
                }
            },
        }
    }

    fn lower_ty_def(&mut self, def: &ast::TyDef) -> hir::TyDef {
        hir::TyDef {
            id: self.lower_id(def.id),
            span: def.span,
            kind: match &def.kind {
                ast::TyDefKind::Field(name, ty) => hir::TyDefKind::Field(
                    name.as_ref().map(|n| self.lower_ident(n)),
                    self.lower_ty(ty),
                ),
                ast::TyDefKind::Paren(inner) => {
                    hir::TyDefKind::Paren(Box::new(self.lower_ty_def(inner)))
                }
                ast::TyDefKind::Tuple(defs) => {
                    hir::TyDefKind::Tuple(defs.iter().map(|d| self.lower_ty_def(d)).collect())
                }
            },
        }
    }

    fn lower_functor_expr(&mut self, expr: &ast::FunctorExpr) -> hir::FunctorExpr {
        hir::FunctorExpr {
            id: self.lower_id(expr.id),
            span: expr.span,
            kind: match &expr.kind {
                ast::FunctorExprKind::BinOp(op, lhs, rhs) => hir::FunctorExprKind::BinOp(
                    match op {
                        ast::SetOp::Union => hir::SetOp::Union,
                        ast::SetOp::Intersect => hir::SetOp::Intersect,
                    },
                    Box::new(self.lower_functor_expr(lhs)),
                    Box::new(self.lower_functor_expr(rhs)),
                ),
                &ast::FunctorExprKind::Lit(functor) => {
                    hir::FunctorExprKind::Lit(lower_functor(functor))
                }
                ast::FunctorExprKind::Paren(inner) => {
                    hir::FunctorExprKind::Paren(Box::new(self.lower_functor_expr(inner)))
                }
            },
        }
    }

    fn lower_ty(&mut self, ty: &ast::Ty) -> hir::Ty {
        let id = self.lower_id(ty.id);
        let kind = match &ty.kind {
            ast::TyKind::Array(item) => hir::TyKind::Array(Box::new(self.lower_ty(item))),
            ast::TyKind::Arrow(kind, input, output, functors) => hir::TyKind::Arrow(
                lower_callable_kind(*kind),
                Box::new(self.lower_ty(input)),
                Box::new(self.lower_ty(output)),
                functors.as_ref().map(|f| self.lower_functor_expr(f)),
            ),
            ast::TyKind::Hole => hir::TyKind::Hole,
            ast::TyKind::Paren(inner) => hir::TyKind::Paren(Box::new(self.lower_ty(inner))),
            ast::TyKind::Path(path) => hir::TyKind::Name(self.lower_path(path)),
            ast::TyKind::Prim(ast::TyPrim::BigInt) => hir::TyKind::Prim(hir::TyPrim::BigInt),
            ast::TyKind::Prim(ast::TyPrim::Bool) => hir::TyKind::Prim(hir::TyPrim::Bool),
            ast::TyKind::Prim(ast::TyPrim::Double) => hir::TyKind::Prim(hir::TyPrim::Double),
            ast::TyKind::Prim(ast::TyPrim::Int) => hir::TyKind::Prim(hir::TyPrim::Int),
            ast::TyKind::Prim(ast::TyPrim::Pauli) => hir::TyKind::Prim(hir::TyPrim::Pauli),
            ast::TyKind::Prim(ast::TyPrim::Qubit) => hir::TyKind::Prim(hir::TyPrim::Qubit),
            ast::TyKind::Prim(ast::TyPrim::Range) => hir::TyKind::Prim(hir::TyPrim::Range),
            ast::TyKind::Prim(ast::TyPrim::Result) => hir::TyKind::Prim(hir::TyPrim::Result),
            ast::TyKind::Prim(ast::TyPrim::String) => hir::TyKind::Prim(hir::TyPrim::String),
            ast::TyKind::Tuple(tys) => {
                hir::TyKind::Tuple(tys.iter().map(|t| self.lower_ty(t)).collect())
            }
            ast::TyKind::Var(name) => hir::TyKind::Var(self.lower_ident(name)),
        };

        hir::Ty {
            id,
            span: ty.span,
            kind,
        }
    }

    fn lower_block(&mut self, block: &ast::Block) -> hir::Block {
        hir::Block {
            id: self.lower_id(block.id),
            span: block.span,
            stmts: block.stmts.iter().map(|s| self.lower_stmt(s)).collect(),
        }
    }

    pub(super) fn lower_stmt(&mut self, stmt: &ast::Stmt) -> hir::Stmt {
        let id = self.lower_id(stmt.id);
        let kind = match &stmt.kind {
            ast::StmtKind::Empty => hir::StmtKind::Empty,
            ast::StmtKind::Expr(expr) => hir::StmtKind::Expr(self.lower_expr(expr)),
            ast::StmtKind::Local(mutability, lhs, rhs) => hir::StmtKind::Local(
                match mutability {
                    ast::Mutability::Immutable => hir::Mutability::Immutable,
                    ast::Mutability::Mutable => hir::Mutability::Mutable,
                },
                self.lower_pat(lhs),
                self.lower_expr(rhs),
            ),
            ast::StmtKind::Qubit(source, lhs, rhs, block) => hir::StmtKind::Qubit(
                match source {
                    ast::QubitSource::Fresh => hir::QubitSource::Fresh,
                    ast::QubitSource::Dirty => hir::QubitSource::Dirty,
                },
                self.lower_pat(lhs),
                self.lower_qubit_init(rhs),
                block.as_ref().map(|b| self.lower_block(b)),
            ),
            ast::StmtKind::Semi(expr) => hir::StmtKind::Semi(self.lower_expr(expr)),
        };

        hir::Stmt {
            id,
            span: stmt.span,
            kind,
        }
    }

    fn lower_expr(&mut self, expr: &ast::Expr) -> hir::Expr {
        let id = self.lower_id(expr.id);
        let kind = match &expr.kind {
            ast::ExprKind::Array(items) => {
                hir::ExprKind::Array(items.iter().map(|i| self.lower_expr(i)).collect())
            }
            ast::ExprKind::ArrayRepeat(value, size) => hir::ExprKind::ArrayRepeat(
                Box::new(self.lower_expr(value)),
                Box::new(self.lower_expr(size)),
            ),
            ast::ExprKind::Assign(lhs, rhs) => hir::ExprKind::Assign(
                Box::new(self.lower_expr(lhs)),
                Box::new(self.lower_expr(rhs)),
            ),
            ast::ExprKind::AssignOp(op, lhs, rhs) => hir::ExprKind::AssignOp(
                lower_binop(*op),
                Box::new(self.lower_expr(lhs)),
                Box::new(self.lower_expr(rhs)),
            ),
            ast::ExprKind::AssignUpdate(container, index, value) => hir::ExprKind::AssignUpdate(
                Box::new(self.lower_expr(container)),
                Box::new(self.lower_expr(index)),
                Box::new(self.lower_expr(value)),
            ),
            ast::ExprKind::BinOp(op, lhs, rhs) => hir::ExprKind::BinOp(
                lower_binop(*op),
                Box::new(self.lower_expr(lhs)),
                Box::new(self.lower_expr(rhs)),
            ),
            ast::ExprKind::Block(block) => hir::ExprKind::Block(self.lower_block(block)),
            ast::ExprKind::Call(callee, arg) => hir::ExprKind::Call(
                Box::new(self.lower_expr(callee)),
                Box::new(self.lower_expr(arg)),
            ),
            ast::ExprKind::Conjugate(within, apply) => {
                hir::ExprKind::Conjugate(self.lower_block(within), self.lower_block(apply))
            }
            ast::ExprKind::Err => hir::ExprKind::Err,
            ast::ExprKind::Fail(message) => hir::ExprKind::Fail(Box::new(self.lower_expr(message))),
            ast::ExprKind::Field(container, name) => {
                hir::ExprKind::Field(Box::new(self.lower_expr(container)), self.lower_ident(name))
            }
            ast::ExprKind::For(pat, iter, block) => hir::ExprKind::For(
                self.lower_pat(pat),
                Box::new(self.lower_expr(iter)),
                self.lower_block(block),
            ),
            ast::ExprKind::Hole => hir::ExprKind::Hole,
            ast::ExprKind::If(cond, if_true, if_false) => hir::ExprKind::If(
                Box::new(self.lower_expr(cond)),
                self.lower_block(if_true),
                if_false.as_ref().map(|e| Box::new(self.lower_expr(e))),
            ),
            ast::ExprKind::Index(container, index) => hir::ExprKind::Index(
                Box::new(self.lower_expr(container)),
                Box::new(self.lower_expr(index)),
            ),
            ast::ExprKind::Lambda(kind, input, body) => hir::ExprKind::Lambda(
                lower_callable_kind(*kind),
                self.lower_pat(input),
                Box::new(self.lower_expr(body)),
            ),
            ast::ExprKind::Lit(lit) => hir::ExprKind::Lit(lower_lit(lit)),
            ast::ExprKind::Paren(inner) => hir::ExprKind::Paren(Box::new(self.lower_expr(inner))),
            ast::ExprKind::Path(path) => hir::ExprKind::Name(self.lower_path(path)),
            ast::ExprKind::Range(start, step, end) => hir::ExprKind::Range(
                start.as_ref().map(|s| Box::new(self.lower_expr(s))),
                step.as_ref().map(|s| Box::new(self.lower_expr(s))),
                end.as_ref().map(|e| Box::new(self.lower_expr(e))),
            ),
            ast::ExprKind::Repeat(body, cond, fixup) => hir::ExprKind::Repeat(
                self.lower_block(body),
                Box::new(self.lower_expr(cond)),
                fixup.as_ref().map(|f| self.lower_block(f)),
            ),
            ast::ExprKind::Return(expr) => hir::ExprKind::Return(Box::new(self.lower_expr(expr))),
            ast::ExprKind::TernOp(op, lhs, middle, rhs) => hir::ExprKind::TernOp(
                lower_ternop(*op),
                Box::new(self.lower_expr(lhs)),
                Box::new(self.lower_expr(middle)),
                Box::new(self.lower_expr(rhs)),
            ),
            ast::ExprKind::Tuple(items) => {
                hir::ExprKind::Tuple(items.iter().map(|i| self.lower_expr(i)).collect())
            }
            ast::ExprKind::UnOp(op, operand) => {
                hir::ExprKind::UnOp(lower_unop(*op), Box::new(self.lower_expr(operand)))
            }
            ast::ExprKind::While(cond, body) => {
                hir::ExprKind::While(Box::new(self.lower_expr(cond)), self.lower_block(body))
            }
        };

        hir::Expr {
            id,
            span: expr.span,
            kind,
        }
    }

    fn lower_pat(&mut self, pat: &ast::Pat) -> hir::Pat {
        let id = self.lower_id(pat.id);
        let kind = match &pat.kind {
            ast::PatKind::Bind(name, ty) => hir::PatKind::Bind(
                self.lower_ident(name),
                ty.as_ref().map(|t| self.lower_ty(t)),
            ),
            ast::PatKind::Discard(ty) => {
                hir::PatKind::Discard(ty.as_ref().map(|t| self.lower_ty(t)))
            }
            ast::PatKind::Elided => hir::PatKind::Elided,
            ast::PatKind::Paren(inner) => hir::PatKind::Paren(Box::new(self.lower_pat(inner))),
            ast::PatKind::Tuple(items) => {
                hir::PatKind::Tuple(items.iter().map(|i| self.lower_pat(i)).collect())
            }
        };

        hir::Pat {
            id,
            span: pat.span,
            kind,
        }
    }

    fn lower_qubit_init(&mut self, init: &ast::QubitInit) -> hir::QubitInit {
        let id = self.lower_id(init.id);
        let kind = match &init.kind {
            ast::QubitInitKind::Array(length) => {
                hir::QubitInitKind::Array(Box::new(self.lower_expr(length)))
            }
            ast::QubitInitKind::Paren(inner) => {
                hir::QubitInitKind::Paren(Box::new(self.lower_qubit_init(inner)))
            }
            ast::QubitInitKind::Single => hir::QubitInitKind::Single,
            ast::QubitInitKind::Tuple(items) => {
                hir::QubitInitKind::Tuple(items.iter().map(|i| self.lower_qubit_init(i)).collect())
            }
        };

        hir::QubitInit {
            id,
            span: init.span,
            kind,
        }
    }

    fn lower_path(&mut self, path: &ast::Path) -> hir::Res {
        match self.resolutions.get(path.id) {
            None => hir::Res::Err,
            Some(&resolve::Res::Def(def)) => hir::Res::Def(def),
            Some(&resolve::Res::Local(node)) => hir::Res::Local(self.lower_id(node)),
        }
    }

    fn lower_ident(&mut self, ident: &ast::Ident) -> hir::Ident {
        hir::Ident {
            id: self.lower_id(ident.id),
            span: ident.span,
            name: ident.name.clone(),
        }
    }

    fn lower_id(&mut self, id: ast::NodeId) -> hir::NodeId {
        self.lowerer.nodes.get(id).copied().unwrap_or_else(|| {
            let new_id = self.lowerer.assigner.next_id();
            self.lowerer.nodes.insert(id, new_id);
            new_id
        })
    }
}

fn lower_callable_kind(kind: ast::CallableKind) -> hir::CallableKind {
    match kind {
        ast::CallableKind::Function => hir::CallableKind::Function,
        ast::CallableKind::Operation => hir::CallableKind::Operation,
    }
}

fn lower_unop(op: ast::UnOp) -> hir::UnOp {
    match op {
        ast::UnOp::Functor(f) => hir::UnOp::Functor(lower_functor(f)),
        ast::UnOp::Neg => hir::UnOp::Neg,
        ast::UnOp::NotB => hir::UnOp::NotB,
        ast::UnOp::NotL => hir::UnOp::NotL,
        ast::UnOp::Pos => hir::UnOp::Pos,
        ast::UnOp::Unwrap => hir::UnOp::Unwrap,
    }
}

fn lower_binop(op: ast::BinOp) -> hir::BinOp {
    match op {
        ast::BinOp::Add => hir::BinOp::Add,
        ast::BinOp::AndB => hir::BinOp::AndB,
        ast::BinOp::AndL => hir::BinOp::AndL,
        ast::BinOp::Div => hir::BinOp::Div,
        ast::BinOp::Eq => hir::BinOp::Eq,
        ast::BinOp::Exp => hir::BinOp::Exp,
        ast::BinOp::Gt => hir::BinOp::Gt,
        ast::BinOp::Gte => hir::BinOp::Gte,
        ast::BinOp::Lt => hir::BinOp::Lt,
        ast::BinOp::Lte => hir::BinOp::Lte,
        ast::BinOp::Mod => hir::BinOp::Mod,
        ast::BinOp::Mul => hir::BinOp::Mul,
        ast::BinOp::Neq => hir::BinOp::Neq,
        ast::BinOp::OrB => hir::BinOp::OrB,
        ast::BinOp::OrL => hir::BinOp::OrL,
        ast::BinOp::Shl => hir::BinOp::Shl,
        ast::BinOp::Shr => hir::BinOp::Shr,
        ast::BinOp::Sub => hir::BinOp::Sub,
        ast::BinOp::XorB => hir::BinOp::XorB,
    }
}

fn lower_ternop(op: ast::TernOp) -> hir::TernOp {
    match op {
        ast::TernOp::Cond => hir::TernOp::Cond,
        ast::TernOp::Update => hir::TernOp::Update,
    }
}

fn lower_lit(lit: &ast::Lit) -> hir::Lit {
    match lit {
        ast::Lit::BigInt(i) => hir::Lit::BigInt(i.clone()),
        &ast::Lit::Bool(b) => hir::Lit::Bool(b),
        &ast::Lit::Double(d) => hir::Lit::Double(d),
        &ast::Lit::Int(i) => hir::Lit::Int(i),
        ast::Lit::Pauli(ast::Pauli::I) => hir::Lit::Pauli(hir::Pauli::I),
        ast::Lit::Pauli(ast::Pauli::X) => hir::Lit::Pauli(hir::Pauli::X),
        ast::Lit::Pauli(ast::Pauli::Y) => hir::Lit::Pauli(hir::Pauli::Y),
        ast::Lit::Pauli(ast::Pauli::Z) => hir::Lit::Pauli(hir::Pauli::Z),
        ast::Lit::Result(ast::Result::One) => hir::Lit::Result(hir::Result::One),
        ast::Lit::Result(ast::Result::Zero) => hir::Lit::Result(hir::Result::Zero),
        ast::Lit::String(s) => hir::Lit::String(s.clone()),
    }
}

fn lower_functor(functor: ast::Functor) -> hir::Functor {
    match functor {
        ast::Functor::Adj => hir::Functor::Adj,
        ast::Functor::Ctl => hir::Functor::Ctl,
    }
}
