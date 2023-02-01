// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    Attribute, Block, CallBody, CallHeader, DeclInfo, Expr, ExprKind, FunctorExpr, Ident, Item,
    Namespace, Pat, Path, Project, QubitInit, SpecBody, SpecDecl, Stage, Ty, TypeDef,
};

pub trait Transform<S1: Stage, S2: Stage>: Sized {
    fn map_project(&mut self, project: Project<S1>) -> Project<S2> {
        map_project(self, project)
    }

    fn map_namespace(&mut self, namespace: Namespace<S1>) -> Namespace<S2> {
        map_namespace(self, namespace)
    }

    fn map_item(&mut self, item: Item<S1>) -> Item<S2> {
        map_item(self, item)
    }

    fn map_decl_info(&mut self, info: DeclInfo<S1>) -> DeclInfo<S2> {
        map_decl_info(self, info)
    }

    fn map_attribute(&mut self, attr: Attribute<S1>) -> Attribute<S2> {
        map_attribute(self, attr)
    }

    fn map_type_def(&mut self, def: TypeDef<S1>) -> TypeDef<S2> {
        map_type_def(self, def)
    }

    fn map_call_header(&mut self, header: CallHeader<S1>) -> CallHeader<S2> {
        map_call_header(self, header)
    }

    fn map_call_body(&mut self, body: CallBody<S1>) -> CallBody<S2> {
        map_call_body(self, body)
    }

    fn map_spec_decl(&mut self, decl: SpecDecl<S1>) -> SpecDecl<S2> {
        map_spec_decl(self, decl)
    }

    fn map_spec_body(&mut self, body: SpecBody<S1>) -> SpecBody<S2> {
        map_spec_body(self, body)
    }

    fn map_functor_expr(&mut self, expr: FunctorExpr<S1>) -> FunctorExpr<S2> {
        map_functor_expr(self, expr)
    }

    fn map_ty(&mut self, ty: Ty<S1>) -> Ty<S2> {
        map_ty(self, ty)
    }

    fn map_expr(&mut self, expr: Expr<S1>) -> Expr<S2> {
        map_expr(self, expr)
    }

    fn map_block(&mut self, block: Block<S1>) -> Block<S2> {
        map_block(self, block)
    }

    fn map_ident(&mut self, ident: Ident<S1>) -> Ident<S2> {
        map_ident(self, ident)
    }

    fn map_path(&mut self, path: Path<S1>) -> Path<S2> {
        map_path(self, path)
    }

    fn map_pat(&mut self, pat: Pat<S1>) -> Pat<S2> {
        map_pat(self, pat)
    }

    fn map_qubit_init(&mut self, init: QubitInit<S1>) -> QubitInit<S2> {
        map_qubit_init(self, init)
    }

    fn map_stage_attribute(&mut self, stage: S1::Attribute) -> S2::Attribute;

    fn map_stage_block(&mut self, stage: S1::Block) -> S2::Block;

    fn map_stage_call_body_full(&mut self, stage: S1::CallBodyFull) -> S2::CallBodyFull;

    fn map_stage_call_body_single(&mut self, stage: S1::CallBodySingle) -> S2::CallBodySingle;

    fn map_stage_call_body_x(&mut self, stage: S1::CallBodyX) -> S2::CallBodyX;

    fn map_stage_call_header(&mut self, stage: S1::CallHeader) -> S2::CallHeader;

    fn map_stage_decl_info(&mut self, stage: S1::DeclInfo) -> S2::DeclInfo;

    fn map_stage_expr(&mut self, stage: S1::Expr) -> S2::Expr;

    fn map_stage_expr_array(&mut self, stage: S1::ExprArray) -> S2::ExprArray;

    fn map_stage_expr_array_repeat(&mut self, stage: S1::ExprArrayRepeat) -> S2::ExprArrayRepeat;

    fn map_stage_expr_assign(&mut self, stage: S1::ExprAssign) -> S2::ExprAssign;

    fn map_stage_expr_assign_op(&mut self, stage: S1::ExprAssignOp) -> S2::ExprAssignOp;

    fn map_stage_expr_assign_update(&mut self, stage: S1::ExprAssignUpdate)
        -> S2::ExprAssignUpdate;

    fn map_stage_expr_bin_op(&mut self, stage: S1::ExprBinOp) -> S2::ExprBinOp;

    fn map_stage_expr_block(&mut self, stage: S1::ExprBlock) -> S2::ExprBlock;

    fn map_stage_expr_call(&mut self, stage: S1::ExprCall) -> S2::ExprCall;

    fn map_stage_expr_conjugate(&mut self, stage: S1::ExprConjugate) -> S2::ExprConjugate;

    fn map_stage_expr_fail(&mut self, stage: S1::ExprFail) -> S2::ExprFail;

    fn map_stage_expr_field(&mut self, stage: S1::ExprField) -> S2::ExprField;

    fn map_stage_expr_for(&mut self, stage: S1::ExprFor) -> S2::ExprFor;

    fn map_stage_expr_hole(&mut self, stage: S1::ExprHole) -> S2::ExprHole;

    fn map_stage_expr_if(&mut self, stage: S1::ExprIf) -> S2::ExprIf;

    fn map_stage_expr_index(&mut self, stage: S1::ExprIndex) -> S2::ExprIndex;

    fn map_stage_expr_interp(&mut self, stage: S1::ExprInterp) -> S2::ExprInterp;

    fn map_stage_expr_lambda(&mut self, stage: S1::ExprLambda) -> S2::ExprLambda;

    fn map_stage_expr_let(&mut self, stage: S1::ExprLet) -> S2::ExprLet;

    fn map_stage_expr_lit(&mut self, stage: S1::ExprLit) -> S2::ExprLit;

    fn map_stage_expr_path(&mut self, stage: S1::ExprPath) -> S2::ExprPath;

    fn map_stage_expr_qubit(&mut self, stage: S1::ExprQubit) -> S2::ExprQubit;

    fn map_stage_expr_range(&mut self, stage: S1::ExprRange) -> S2::ExprRange;

    fn map_stage_expr_repeat(&mut self, stage: S1::ExprRepeat) -> S2::ExprRepeat;

    fn map_stage_expr_return(&mut self, stage: S1::ExprReturn) -> S2::ExprReturn;

    fn map_stage_expr_tern_op(&mut self, stage: S1::ExprTernOp) -> S2::ExprTernOp;

    fn map_stage_expr_tuple(&mut self, stage: S1::ExprTuple) -> S2::ExprTuple;

    fn map_stage_expr_un_op(&mut self, stage: S1::ExprUnOp) -> S2::ExprUnOp;

    fn map_stage_expr_while(&mut self, stage: S1::ExprWhile) -> S2::ExprWhile;

    fn map_stage_expr_x(&mut self, stage: S1::ExprX) -> S2::ExprX;

    fn map_stage_functor_expr_bin_op(
        &mut self,
        stage: S1::FunctorExprBinOp,
    ) -> S2::FunctorExprBinOp;

    fn map_stage_functor_expr_lit(&mut self, stage: S1::FunctorExprLit) -> S2::FunctorExprLit;

    fn map_stage_functor_expr_null(&mut self, stage: S1::FunctorExprNull) -> S2::FunctorExprNull;

    fn map_stage_functor_expr_x(&mut self, stage: S1::FunctorExprX) -> S2::FunctorExprX;

    fn map_stage_ident(&mut self, stage: S1::Ident) -> S2::Ident;

    fn map_stage_item_callable(&mut self, stage: S1::ItemCallable) -> S2::ItemCallable;

    fn map_stage_item_open(&mut self, stage: S1::ItemOpen) -> S2::ItemOpen;

    fn map_stage_item_type(&mut self, stage: S1::ItemType) -> S2::ItemType;

    fn map_stage_item_x(&mut self, stage: S1::ItemX) -> S2::ItemX;

    fn map_stage_namespace(&mut self, stage: S1::Namespace) -> S2::Namespace;

    fn map_stage_pat_bind(&mut self, stage: S1::PatBind) -> S2::PatBind;

    fn map_stage_pat_discard(&mut self, stage: S1::PatDiscard) -> S2::PatDiscard;

    fn map_stage_path(&mut self, stage: S1::Path) -> S2::Path;

    fn map_stage_pat_omit(&mut self, stage: S1::PatOmit) -> S2::PatOmit;

    fn map_stage_pat_tuple(&mut self, stage: S1::PatTuple) -> S2::PatTuple;

    fn map_stage_pat_x(&mut self, stage: S1::PatX) -> S2::PatX;

    fn map_stage_project(&mut self, stage: S1::Project) -> S2::Project;

    fn map_stage_qubit_init_array(&mut self, stage: S1::QubitInitArray) -> S2::QubitInitArray;

    fn map_stage_qubit_init_single(&mut self, stage: S1::QubitInitSingle) -> S2::QubitInitSingle;

    fn map_stage_qubit_init_tuple(&mut self, stage: S1::QubitInitTuple) -> S2::QubitInitTuple;

    fn map_stage_qubit_init_x(&mut self, stage: S1::QubitInitX) -> S2::QubitInitX;

    fn map_stage_spec_body_gen(&mut self, stage: S1::SpecBodyGen) -> S2::SpecBodyGen;

    fn map_stage_spec_body_impl(&mut self, stage: S1::SpecBodyImpl) -> S2::SpecBodyImpl;

    fn map_stage_spec_body_x(&mut self, stage: S1::SpecBodyX) -> S2::SpecBodyX;

    fn map_stage_spec_decl(&mut self, stage: S1::SpecDecl) -> S2::SpecDecl;

    fn map_stage_ty_app(&mut self, stage: S1::TyApp) -> S2::TyApp;

    fn map_stage_ty_arrow(&mut self, stage: S1::TyArrow) -> S2::TyArrow;

    fn map_stage_ty_hole(&mut self, stage: S1::TyHole) -> S2::TyHole;

    fn map_stage_ty_path(&mut self, stage: S1::TyPath) -> S2::TyPath;

    fn map_stage_type_def_field(&mut self, stage: S1::TypeDefField) -> S2::TypeDefField;

    fn map_stage_type_def_tuple(&mut self, stage: S1::TypeDefTuple) -> S2::TypeDefTuple;

    fn map_stage_type_def_x(&mut self, stage: S1::TypeDefX) -> S2::TypeDefX;

    fn map_stage_ty_prim(&mut self, stage: S1::TyPrim) -> S2::TyPrim;

    fn map_stage_ty_tuple(&mut self, stage: S1::TyTuple) -> S2::TyTuple;

    fn map_stage_ty_var(&mut self, stage: S1::TyVar) -> S2::TyVar;

    fn map_stage_ty_x(&mut self, stage: S1::TyX) -> S2::TyX;
}

pub fn map_project<S1: Stage, S2: Stage>(
    tr: &mut impl Transform<S1, S2>,
    project: Project<S1>,
) -> Project<S2> {
    Project {
        stage: tr.map_stage_project(project.stage),
        namespaces: project
            .namespaces
            .into_iter()
            .map(|n| tr.map_namespace(n))
            .collect(),
    }
}

pub fn map_namespace<S1: Stage, S2: Stage>(
    tr: &mut impl Transform<S1, S2>,
    namespace: Namespace<S1>,
) -> Namespace<S2> {
    Namespace {
        stage: tr.map_stage_namespace(namespace.stage),
        name: tr.map_path(namespace.name),
        items: namespace
            .items
            .into_iter()
            .map(|i| tr.map_item(i))
            .collect(),
    }
}

pub fn map_item<S1: Stage, S2: Stage>(tr: &mut impl Transform<S1, S2>, item: Item<S1>) -> Item<S2> {
    match item {
        Item::Open(stage, path, ident) => Item::Open(
            tr.map_stage_item_open(stage),
            tr.map_path(path),
            tr.map_ident(ident),
        ),
        Item::Type(stage, info, ident, def) => Item::Type(
            tr.map_stage_item_type(stage),
            tr.map_decl_info(info),
            tr.map_ident(ident),
            tr.map_type_def(def),
        ),
        Item::Callable(stage, info, header, body) => Item::Callable(
            tr.map_stage_item_callable(stage),
            tr.map_decl_info(info),
            tr.map_call_header(header),
            tr.map_call_body(body),
        ),
        Item::X(stage) => Item::X(tr.map_stage_item_x(stage)),
    }
}

pub fn map_decl_info<S1: Stage, S2: Stage>(
    tr: &mut impl Transform<S1, S2>,
    info: DeclInfo<S1>,
) -> DeclInfo<S2> {
    DeclInfo {
        stage: tr.map_stage_decl_info(info.stage),
        attributes: info
            .attributes
            .into_iter()
            .map(|a| tr.map_attribute(a))
            .collect(),
        visibility: info.visibility,
    }
}

pub fn map_attribute<S1: Stage, S2: Stage>(
    tr: &mut impl Transform<S1, S2>,
    attr: Attribute<S1>,
) -> Attribute<S2> {
    Attribute {
        stage: tr.map_stage_attribute(attr.stage),
        name: tr.map_path(attr.name),
        arg: tr.map_expr(attr.arg),
    }
}

pub fn map_type_def<S1: Stage, S2: Stage>(
    tr: &mut impl Transform<S1, S2>,
    def: TypeDef<S1>,
) -> TypeDef<S2> {
    match def {
        TypeDef::Field(stage, name, ty) => TypeDef::Field(
            tr.map_stage_type_def_field(stage),
            name.map(|n| tr.map_ident(n)),
            tr.map_ty(ty),
        ),
        TypeDef::Tuple(stage, defs) => TypeDef::Tuple(
            tr.map_stage_type_def_tuple(stage),
            defs.into_iter().map(|d| tr.map_type_def(d)).collect(),
        ),
        TypeDef::X(stage) => TypeDef::X(tr.map_stage_type_def_x(stage)),
    }
}

pub fn map_call_header<S1: Stage, S2: Stage>(
    tr: &mut impl Transform<S1, S2>,
    header: CallHeader<S1>,
) -> CallHeader<S2> {
    CallHeader {
        stage: tr.map_stage_call_header(header.stage),
        kind: header.kind,
        name: tr.map_ident(header.name),
        ty_params: header
            .ty_params
            .into_iter()
            .map(|i| tr.map_ident(i))
            .collect(),
        input: tr.map_pat(header.input),
        output: tr.map_ty(header.output),
        functors: tr.map_functor_expr(header.functors),
    }
}

pub fn map_call_body<S1: Stage, S2: Stage>(
    tr: &mut impl Transform<S1, S2>,
    body: CallBody<S1>,
) -> CallBody<S2> {
    match body {
        CallBody::Single(stage, body) => {
            CallBody::Single(tr.map_stage_call_body_single(stage), tr.map_spec_body(body))
        }
        CallBody::Full(stage, decls) => CallBody::Full(
            tr.map_stage_call_body_full(stage),
            decls.into_iter().map(|d| tr.map_spec_decl(d)).collect(),
        ),
        CallBody::X(stage) => CallBody::X(tr.map_stage_call_body_x(stage)),
    }
}

pub fn map_spec_decl<S1: Stage, S2: Stage>(
    tr: &mut impl Transform<S1, S2>,
    decl: SpecDecl<S1>,
) -> SpecDecl<S2> {
    SpecDecl {
        stage: tr.map_stage_spec_decl(decl.stage),
        spec: decl.spec,
        body: tr.map_spec_body(decl.body),
    }
}

pub fn map_spec_body<S1: Stage, S2: Stage>(
    tr: &mut impl Transform<S1, S2>,
    body: SpecBody<S1>,
) -> SpecBody<S2> {
    match body {
        SpecBody::Gen(stage, gen) => SpecBody::Gen(tr.map_stage_spec_body_gen(stage), gen),
        SpecBody::Impl(stage, pat, block) => SpecBody::Impl(
            tr.map_stage_spec_body_impl(stage),
            tr.map_pat(pat),
            tr.map_block(block),
        ),
        SpecBody::X(stage) => SpecBody::X(tr.map_stage_spec_body_x(stage)),
    }
}

pub fn map_functor_expr<S1: Stage, S2: Stage>(
    tr: &mut impl Transform<S1, S2>,
    expr: FunctorExpr<S1>,
) -> FunctorExpr<S2> {
    match expr {
        FunctorExpr::BinOp(stage, op, lhs, rhs) => FunctorExpr::BinOp(
            tr.map_stage_functor_expr_bin_op(stage),
            op,
            Box::new(tr.map_functor_expr(*lhs)),
            Box::new(tr.map_functor_expr(*rhs)),
        ),
        FunctorExpr::Lit(stage, functor) => {
            FunctorExpr::Lit(tr.map_stage_functor_expr_lit(stage), functor)
        }
        FunctorExpr::Null(stage) => FunctorExpr::Null(tr.map_stage_functor_expr_null(stage)),
        FunctorExpr::X(stage) => FunctorExpr::X(tr.map_stage_functor_expr_x(stage)),
    }
}

pub fn map_ty<S1: Stage, S2: Stage>(tr: &mut impl Transform<S1, S2>, ty: Ty<S1>) -> Ty<S2> {
    match ty {
        Ty::App(stage, ty, tys) => Ty::App(
            tr.map_stage_ty_app(stage),
            Box::new(tr.map_ty(*ty)),
            tys.into_iter().map(|t| tr.map_ty(t)).collect(),
        ),
        Ty::Arrow(stage, kind, lhs, rhs, functors) => Ty::Arrow(
            tr.map_stage_ty_arrow(stage),
            kind,
            Box::new(tr.map_ty(*lhs)),
            Box::new(tr.map_ty(*rhs)),
            tr.map_functor_expr(functors),
        ),
        Ty::Hole(stage) => Ty::Hole(tr.map_stage_ty_hole(stage)),
        Ty::Path(stage, path) => Ty::Path(tr.map_stage_ty_path(stage), tr.map_path(path)),
        Ty::Prim(stage, prim) => Ty::Prim(tr.map_stage_ty_prim(stage), prim),
        Ty::Tuple(stage, tys) => Ty::Tuple(
            tr.map_stage_ty_tuple(stage),
            tys.into_iter().map(|t| tr.map_ty(t)).collect(),
        ),
        Ty::Var(stage, var) => Ty::Var(tr.map_stage_ty_var(stage), var),
        Ty::X(stage) => Ty::X(tr.map_stage_ty_x(stage)),
    }
}

#[allow(clippy::too_many_lines)]
pub fn map_expr<S1: Stage, S2: Stage>(tr: &mut impl Transform<S1, S2>, expr: Expr<S1>) -> Expr<S2> {
    let kind = match expr.kind {
        ExprKind::Array(stage, exprs) => ExprKind::Array(
            tr.map_stage_expr_array(stage),
            exprs.into_iter().map(|e| tr.map_expr(e)).collect(),
        ),
        ExprKind::ArrayRepeat(stage, item, size) => ExprKind::ArrayRepeat(
            tr.map_stage_expr_array_repeat(stage),
            Box::new(tr.map_expr(*item)),
            Box::new(tr.map_expr(*size)),
        ),
        ExprKind::Assign(stage, lhs, rhs) => ExprKind::Assign(
            tr.map_stage_expr_assign(stage),
            Box::new(tr.map_expr(*lhs)),
            Box::new(tr.map_expr(*rhs)),
        ),
        ExprKind::AssignOp(stage, op, lhs, rhs) => ExprKind::AssignOp(
            tr.map_stage_expr_assign_op(stage),
            op,
            Box::new(tr.map_expr(*lhs)),
            Box::new(tr.map_expr(*rhs)),
        ),
        ExprKind::AssignUpdate(stage, record, index, value) => ExprKind::AssignUpdate(
            tr.map_stage_expr_assign_update(stage),
            Box::new(tr.map_expr(*record)),
            Box::new(tr.map_expr(*index)),
            Box::new(tr.map_expr(*value)),
        ),
        ExprKind::BinOp(stage, op, lhs, rhs) => ExprKind::BinOp(
            tr.map_stage_expr_bin_op(stage),
            op,
            Box::new(tr.map_expr(*lhs)),
            Box::new(tr.map_expr(*rhs)),
        ),
        ExprKind::Block(stage, block) => {
            ExprKind::Block(tr.map_stage_expr_block(stage), tr.map_block(block))
        }
        ExprKind::Call(stage, callee, arg) => ExprKind::Call(
            tr.map_stage_expr_call(stage),
            Box::new(tr.map_expr(*callee)),
            Box::new(tr.map_expr(*arg)),
        ),
        ExprKind::Conjugate(stage, within, apply) => ExprKind::Conjugate(
            tr.map_stage_expr_conjugate(stage),
            tr.map_block(within),
            tr.map_block(apply),
        ),
        ExprKind::Fail(stage, msg) => {
            ExprKind::Fail(tr.map_stage_expr_fail(stage), Box::new(tr.map_expr(*msg)))
        }
        ExprKind::Field(stage, record, name) => ExprKind::Field(
            tr.map_stage_expr_field(stage),
            Box::new(tr.map_expr(*record)),
            tr.map_ident(name),
        ),
        ExprKind::For(stage, pat, iter, block) => ExprKind::For(
            tr.map_stage_expr_for(stage),
            tr.map_pat(pat),
            Box::new(tr.map_expr(*iter)),
            tr.map_block(block),
        ),
        ExprKind::Hole(stage) => ExprKind::Hole(tr.map_stage_expr_hole(stage)),
        ExprKind::If(stage, branches, default) => ExprKind::If(
            tr.map_stage_expr_if(stage),
            branches
                .into_iter()
                .map(|(cond, block)| (tr.map_expr(cond), tr.map_block(block)))
                .collect(),
            default.map(|d| tr.map_block(d)),
        ),
        ExprKind::Index(stage, array, item) => ExprKind::Index(
            tr.map_stage_expr_index(stage),
            Box::new(tr.map_expr(*array)),
            Box::new(tr.map_expr(*item)),
        ),
        ExprKind::Interp(stage, str, exprs) => ExprKind::Interp(
            tr.map_stage_expr_interp(stage),
            str,
            exprs.into_iter().map(|e| tr.map_expr(e)).collect(),
        ),
        ExprKind::Lambda(stage, kind, pat, expr) => ExprKind::Lambda(
            tr.map_stage_expr_lambda(stage),
            kind,
            tr.map_pat(pat),
            Box::new(tr.map_expr(*expr)),
        ),
        ExprKind::Let(stage, pat, value) => ExprKind::Let(
            tr.map_stage_expr_let(stage),
            tr.map_pat(pat),
            Box::new(tr.map_expr(*value)),
        ),
        ExprKind::Lit(stage, lit) => ExprKind::Lit(tr.map_stage_expr_lit(stage), lit),
        ExprKind::Path(stage, path) => {
            ExprKind::Path(tr.map_stage_expr_path(stage), tr.map_path(path))
        }
        ExprKind::Qubit(stage, kind, pat, init, block) => ExprKind::Qubit(
            tr.map_stage_expr_qubit(stage),
            kind,
            tr.map_pat(pat),
            tr.map_qubit_init(init),
            block.map(|b| tr.map_block(b)),
        ),
        ExprKind::Range(stage, start, step, end) => ExprKind::Range(
            tr.map_stage_expr_range(stage),
            Box::new(tr.map_expr(*start)),
            Box::new(tr.map_expr(*step)),
            Box::new(tr.map_expr(*end)),
        ),
        ExprKind::Repeat(stage, body, until, fixup) => ExprKind::Repeat(
            tr.map_stage_expr_repeat(stage),
            tr.map_block(body),
            Box::new(tr.map_expr(*until)),
            fixup.map(|f| tr.map_block(f)),
        ),
        ExprKind::Return(stage, expr) => ExprKind::Return(
            tr.map_stage_expr_return(stage),
            Box::new(tr.map_expr(*expr)),
        ),
        ExprKind::TernOp(stage, op, e1, e2, e3) => ExprKind::TernOp(
            tr.map_stage_expr_tern_op(stage),
            op,
            Box::new(tr.map_expr(*e1)),
            Box::new(tr.map_expr(*e2)),
            Box::new(tr.map_expr(*e3)),
        ),
        ExprKind::Tuple(stage, exprs) => ExprKind::Tuple(
            tr.map_stage_expr_tuple(stage),
            exprs.into_iter().map(|e| tr.map_expr(e)).collect(),
        ),
        ExprKind::UnOp(stage, op, expr) => ExprKind::UnOp(
            tr.map_stage_expr_un_op(stage),
            op,
            Box::new(tr.map_expr(*expr)),
        ),
        ExprKind::While(stage, cond, block) => ExprKind::While(
            tr.map_stage_expr_while(stage),
            Box::new(tr.map_expr(*cond)),
            tr.map_block(block),
        ),
        ExprKind::X(stage) => ExprKind::X(tr.map_stage_expr_x(stage)),
    };

    Expr {
        stage: tr.map_stage_expr(expr.stage),
        kind,
    }
}

pub fn map_block<S1: Stage, S2: Stage>(
    tr: &mut impl Transform<S1, S2>,
    block: Block<S1>,
) -> Block<S2> {
    Block {
        stage: tr.map_stage_block(block.stage),
        exprs: block.exprs.into_iter().map(|e| tr.map_expr(e)).collect(),
    }
}

pub fn map_ident<S1: Stage, S2: Stage>(
    tr: &mut impl Transform<S1, S2>,
    ident: Ident<S1>,
) -> Ident<S2> {
    Ident {
        stage: tr.map_stage_ident(ident.stage),
        name: ident.name,
    }
}

pub fn map_path<S1: Stage, S2: Stage>(tr: &mut impl Transform<S1, S2>, path: Path<S1>) -> Path<S2> {
    Path {
        stage: tr.map_stage_path(path.stage),
        parts: path.parts,
    }
}

pub fn map_pat<S1: Stage, S2: Stage>(tr: &mut impl Transform<S1, S2>, pat: Pat<S1>) -> Pat<S2> {
    match pat {
        Pat::Bind(stage, mutable, name, ty) => Pat::Bind(
            tr.map_stage_pat_bind(stage),
            mutable,
            tr.map_ident(name),
            tr.map_ty(ty),
        ),
        Pat::Discard(stage, ty) => Pat::Discard(tr.map_stage_pat_discard(stage), tr.map_ty(ty)),
        Pat::Omit(stage) => Pat::Omit(tr.map_stage_pat_omit(stage)),
        Pat::Tuple(stage, pats) => Pat::Tuple(
            tr.map_stage_pat_tuple(stage),
            pats.into_iter().map(|p| tr.map_pat(p)).collect(),
        ),
        Pat::X(stage) => Pat::X(tr.map_stage_pat_x(stage)),
    }
}

pub fn map_qubit_init<S1: Stage, S2: Stage>(
    tr: &mut impl Transform<S1, S2>,
    init: QubitInit<S1>,
) -> QubitInit<S2> {
    match init {
        QubitInit::Single(stage) => QubitInit::Single(tr.map_stage_qubit_init_single(stage)),
        QubitInit::Tuple(stage, inits) => QubitInit::Tuple(
            tr.map_stage_qubit_init_tuple(stage),
            inits.into_iter().map(|i| tr.map_qubit_init(i)).collect(),
        ),
        QubitInit::Array(stage, len) => QubitInit::Array(
            tr.map_stage_qubit_init_array(stage),
            Box::new(tr.map_expr(*len)),
        ),
        QubitInit::X(stage) => QubitInit::X(tr.map_stage_qubit_init_x(stage)),
    }
}
