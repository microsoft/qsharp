// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    compile::PackageId,
    resolve::{DefId, PackageSrc, Resolutions},
};
use qsc_ast::{
    ast::{
        self, BinOp, Block, CallableBody, CallableDecl, CallableKind, Expr, ExprKind, Functor,
        FunctorExpr, FunctorExprKind, Lit, NodeId, Pat, PatKind, SetOp, SpecBody, TernOp, TyKind,
        TyPrim, UnOp,
    },
    visit::Visitor,
};
use std::{
    collections::{HashMap, HashSet},
    mem,
};

pub type Tys = HashMap<NodeId, Ty>;

#[derive(Clone, Debug)]
pub enum Ty {
    App(Box<Ty>, Vec<Ty>),
    Arrow(CallableKind, Box<Ty>, Box<Ty>, Option<FunctorExpr>),
    DefId(DefId),
    Prim(TyPrim),
    Rigid(String),
    Tuple(Vec<Ty>),
    Var(u32),
    Void,
}

pub(super) struct Checker<'a> {
    resolutions: &'a Resolutions,
    globals: HashMap<DefId, Ty>,
    tys: Tys,
}

impl Checker<'_> {
    pub(super) fn into_tys(self) -> Tys {
        self.tys
    }
}

impl Visitor<'_> for Checker<'_> {
    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        match &decl.body {
            CallableBody::Block(block) => {
                let mut inferrer = Inferrer::new(self.resolutions, &self.globals);
                inferrer.infer_pat(&decl.input);
                let decl_output = inferrer.convert_ty(&decl.output);
                let block_output = inferrer.infer_block(block);
                inferrer.constrain(Constraint::Eq(decl_output, block_output));
                self.tys.extend(inferrer.solve());
            }
            CallableBody::Specs(specs) => {
                for spec in specs {
                    match &spec.body {
                        SpecBody::Gen(_) => {}
                        SpecBody::Impl(input, block) => {
                            let mut inferrer = Inferrer::new(self.resolutions, &self.globals);
                            inferrer.infer_pat(&decl.input);
                            inferrer.infer_pat(input);
                            let decl_output = inferrer.convert_ty(&decl.output);
                            let block_output = inferrer.infer_block(block);
                            inferrer.constrain(Constraint::Eq(decl_output, block_output));
                            self.tys.extend(inferrer.solve());
                        }
                    }
                }
            }
        }
    }
}

pub(super) struct GlobalTable<'a> {
    resolutions: &'a Resolutions,
    globals: HashMap<DefId, Ty>,
    package: PackageSrc,
}

impl<'a> GlobalTable<'a> {
    pub(super) fn new(resolutions: &'a Resolutions) -> Self {
        Self {
            resolutions,
            globals: HashMap::new(),
            package: PackageSrc::Local,
        }
    }

    pub(super) fn set_package(&mut self, package: PackageId) {
        self.package = PackageSrc::Extern(package);
    }

    pub(super) fn into_checker(self) -> Checker<'a> {
        Checker {
            resolutions: self.resolutions,
            globals: self.globals,
            tys: Tys::new(),
        }
    }
}

impl Visitor<'_> for GlobalTable<'_> {
    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        let id = DefId {
            package: self.package,
            node: decl.name.id,
        };
        let ty = callable_ty(self.resolutions, decl).expect("callable should have a valid type");
        self.globals.insert(id, ty);
    }
}

enum Constraint {
    Eq(Ty, Ty),
    Class(Class),
}

#[derive(Clone)]
enum Class {
    Add(Ty),
    Adj(Ty),
    Call {
        callee: Ty,
        input: Ty,
        output: Ty,
    },
    Ctl {
        op: Ty,
        with_ctls: Ty,
    },
    Eq(Ty),
    HasField {
        record: Ty,
        name: String,
        item: Ty,
    },
    HasFunctorsIfOp {
        callee: Ty,
        functors: HashSet<Functor>,
    },
    HasIndex {
        container: Ty,
        index: Ty,
        item: Ty,
    },
    HasPartialApp {
        callee: Ty,
        missing: Ty,
        with_app: Ty,
    },
    Integral(Ty),
    Iterable {
        container: Ty,
        item: Ty,
    },
    Num(Ty),
    Unwrap {
        wrapper: Ty,
        base: Ty,
    },
}

impl Class {
    fn dependencies(&self) -> Vec<&Ty> {
        match self {
            Self::Add(ty) | Self::Adj(ty) | Self::Eq(ty) | Self::Integral(ty) | Self::Num(ty) => {
                vec![ty]
            }
            Self::Call { callee, .. }
            | Self::HasFunctorsIfOp { callee, .. }
            | Self::HasPartialApp { callee, .. } => vec![callee],
            Self::Ctl { op, .. } => vec![op],
            Self::HasField { record, .. } => vec![record],
            Self::HasIndex {
                container, index, ..
            } => vec![container, index],
            Self::Iterable { container, .. } => vec![container],
            Self::Unwrap { wrapper, .. } => vec![wrapper],
        }
    }

    fn map(self, mut f: impl FnMut(Ty) -> Ty) -> Self {
        match self {
            Self::Add(ty) => Self::Add(f(ty)),
            Self::Adj(ty) => Self::Adj(f(ty)),
            Self::Call {
                callee,
                input,
                output,
            } => Self::Call {
                callee: f(callee),
                input: f(input),
                output: f(output),
            },
            Self::Ctl { op, with_ctls } => Self::Ctl {
                op: f(op),
                with_ctls: f(with_ctls),
            },
            Self::Eq(ty) => Self::Eq(f(ty)),
            Self::HasField { record, name, item } => Self::HasField {
                record: f(record),
                name,
                item: f(item),
            },
            Self::HasFunctorsIfOp { callee, functors } => Self::HasFunctorsIfOp {
                callee: f(callee),
                functors,
            },
            Self::HasIndex {
                container,
                index,
                item,
            } => Self::HasIndex {
                container: f(container),
                index: f(index),
                item: f(item),
            },
            Self::HasPartialApp {
                callee,
                missing,
                with_app,
            } => Self::HasPartialApp {
                callee: f(callee),
                missing: f(missing),
                with_app: f(with_app),
            },
            Self::Integral(ty) => Self::Integral(f(ty)),
            Self::Iterable { container, item } => Self::Iterable {
                container: f(container),
                item: f(item),
            },
            Self::Num(ty) => Self::Num(f(ty)),
            Self::Unwrap { wrapper, base } => Self::Unwrap {
                wrapper: f(wrapper),
                base: f(base),
            },
        }
    }
}

struct Inferrer<'a> {
    resolutions: &'a Resolutions,
    globals: &'a HashMap<DefId, Ty>,
    constraints: Vec<Constraint>,
    tys: Tys,
    next_var: u32,
}

impl<'a> Inferrer<'a> {
    fn new(resolutions: &'a Resolutions, globals: &'a HashMap<DefId, Ty>) -> Self {
        Self {
            resolutions,
            globals,
            constraints: Vec::new(),
            tys: Tys::new(),
            next_var: 0,
        }
    }

    #[allow(clippy::too_many_lines)]
    fn infer_expr(&mut self, expr: &Expr) -> Ty {
        let ty = match &expr.kind {
            ExprKind::Array(items) => match items.split_first() {
                Some((first, rest)) => {
                    let first = self.infer_expr(first);
                    for item in rest {
                        let item = self.infer_expr(item);
                        self.constrain(Constraint::Eq(first.clone(), item));
                    }
                    Ty::App(Box::new(Ty::Prim(TyPrim::Array)), vec![first])
                }
                None => Ty::App(Box::new(Ty::Prim(TyPrim::Array)), vec![self.fresh()]),
            },
            ExprKind::ArrayRepeat(item, size) => {
                let item = self.infer_expr(item);
                let size = self.infer_expr(size);
                self.constrain(Constraint::Eq(size, Ty::Prim(TyPrim::Int)));
                Ty::App(Box::new(Ty::Prim(TyPrim::Array)), vec![item])
            }
            ExprKind::Assign(lhs, rhs) => {
                let lhs = self.infer_expr(lhs);
                let rhs = self.infer_expr(rhs);
                self.constrain(Constraint::Eq(lhs, rhs));
                Ty::Tuple(Vec::new())
            }
            ExprKind::AssignOp(op, lhs, rhs) => {
                self.infer_binop(*op, lhs, rhs);
                Ty::Tuple(Vec::new())
            }
            ExprKind::AssignUpdate(container, index, item) => {
                self.infer_update(container, index, item);
                Ty::Tuple(Vec::new())
            }
            ExprKind::BinOp(op, lhs, rhs) => self.infer_binop(*op, lhs, rhs),
            ExprKind::Block(block) => self.infer_block(block),
            ExprKind::Call(callee, input) => {
                let callee = self.infer_expr(callee);
                let input = self.infer_expr(input);
                let output = self.fresh();
                self.constrain(Constraint::Class(Class::Call {
                    callee,
                    input,
                    output: output.clone(),
                }));
                output
            }
            ExprKind::Conjugate(within, apply) => {
                let within = self.infer_block(within);
                let apply = self.infer_block(apply);
                self.constrain(Constraint::Eq(within, Ty::Tuple(Vec::new())));
                apply
            }
            ExprKind::Fail(message) => {
                let message = self.infer_expr(message);
                self.constrain(Constraint::Eq(message, Ty::Prim(TyPrim::String)));
                Ty::Void
            }
            ExprKind::Field(record, name) => {
                let record = self.infer_expr(record);
                let item = self.fresh();
                self.constrain(Constraint::Class(Class::HasField {
                    record,
                    name: name.name.clone(),
                    item: item.clone(),
                }));
                item
            }
            ExprKind::For(item, container, body) => {
                let item = self.infer_pat(item);
                let container = self.infer_expr(container);
                self.constrain(Constraint::Class(Class::Iterable { container, item }));
                let body = self.infer_block(body);
                self.constrain(Constraint::Eq(body, Ty::Tuple(Vec::new())));
                Ty::Tuple(Vec::new())
            }
            ExprKind::If(cond, if_true, if_false) => {
                let cond = self.infer_expr(cond);
                self.constrain(Constraint::Eq(cond, Ty::Prim(TyPrim::Bool)));
                let if_true = self.infer_block(if_true);
                let if_false = if_false
                    .as_ref()
                    .map_or(Ty::Tuple(Vec::new()), |e| self.infer_expr(e));
                self.constrain(Constraint::Eq(if_true.clone(), if_false));
                if_true
            }
            ExprKind::Index(container, index) => {
                let container = self.infer_expr(container);
                let index = self.infer_expr(index);
                let item = self.fresh();
                self.constrain(Constraint::Class(Class::HasIndex {
                    container,
                    index,
                    item: item.clone(),
                }));
                item
            }
            ExprKind::Lambda(kind, input, body) => {
                let input = self.infer_pat(input);
                let body = self.infer_expr(body);
                Ty::Arrow(*kind, Box::new(input), Box::new(body), None)
            }
            ExprKind::Lit(Lit::BigInt(_)) => Ty::Prim(TyPrim::BigInt),
            ExprKind::Lit(Lit::Bool(_)) => Ty::Prim(TyPrim::Bool),
            ExprKind::Lit(Lit::Double(_)) => Ty::Prim(TyPrim::Double),
            ExprKind::Lit(Lit::Int(_)) => Ty::Prim(TyPrim::Int),
            ExprKind::Lit(Lit::Pauli(_)) => Ty::Prim(TyPrim::Pauli),
            ExprKind::Lit(Lit::Result(_)) => Ty::Prim(TyPrim::Result),
            ExprKind::Lit(Lit::String(_)) => Ty::Prim(TyPrim::String),
            ExprKind::Paren(expr) => self.infer_expr(expr),
            ExprKind::Path(path) => {
                let def = self
                    .resolutions
                    .get(&path.id)
                    .expect("path should be resolved");

                if let Some(ty) = self.globals.get(def) {
                    // TODO: instantiate type parameters
                    ty.clone()
                } else if def.package == PackageSrc::Local {
                    self.tys
                        .get(&def.node)
                        .expect("local variable should have inferred type")
                        .clone()
                } else {
                    panic!("path resolves to external package, but definition not found in globals")
                }
            }
            ExprKind::Range(start, step, end) => {
                for expr in start.iter().chain(step).chain(end) {
                    let ty = self.infer_expr(expr);
                    self.constrain(Constraint::Eq(ty, Ty::Prim(TyPrim::Int)));
                }
                Ty::Prim(TyPrim::Range)
            }
            ExprKind::Repeat(body, until, fixup) => {
                let body = self.infer_block(body);
                self.constrain(Constraint::Eq(body, Ty::Tuple(Vec::new())));
                let until = self.infer_expr(until);
                self.constrain(Constraint::Eq(until, Ty::Prim(TyPrim::Bool)));
                if let Some(fixup) = fixup {
                    let fixup = self.infer_block(fixup);
                    self.constrain(Constraint::Eq(fixup, Ty::Tuple(Vec::new())));
                }

                Ty::Tuple(Vec::new())
            }
            ExprKind::Return(expr) => {
                self.infer_expr(expr);
                Ty::Void
            }
            ExprKind::TernOp(TernOp::Cond, cond, if_true, if_false) => {
                let cond = self.infer_expr(cond);
                self.constrain(Constraint::Eq(cond, Ty::Prim(TyPrim::Bool)));
                let if_true = self.infer_expr(if_true);
                let if_false = self.infer_expr(if_false);
                self.constrain(Constraint::Eq(if_true.clone(), if_false));
                if_true
            }
            ExprKind::TernOp(TernOp::Update, container, index, item) => {
                self.infer_update(container, index, item)
            }
            ExprKind::Tuple(items) => {
                let items = items.iter().map(|e| self.infer_expr(e)).collect();
                Ty::Tuple(items)
            }
            ExprKind::UnOp(op, expr) => self.infer_unop(*op, expr),
            ExprKind::While(cond, body) => {
                let cond = self.infer_expr(cond);
                self.constrain(Constraint::Eq(cond, Ty::Prim(TyPrim::Bool)));
                let body = self.infer_block(body);
                self.constrain(Constraint::Eq(body, Ty::Tuple(Vec::new())));
                Ty::Tuple(Vec::new())
            }
            ExprKind::Err | ExprKind::Hole => Ty::Void,
        };

        self.tys.insert(expr.id, ty.clone());
        ty
    }

    fn infer_block(&mut self, block: &Block) -> Ty {
        todo!()
    }

    fn infer_pat(&mut self, pat: &Pat) -> Ty {
        todo!()
    }

    fn infer_unop(&mut self, op: UnOp, expr: &Expr) -> Ty {
        todo!()
    }

    fn infer_binop(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr) -> Ty {
        todo!()
    }

    fn infer_update(&mut self, container: &Expr, index: &Expr, item: &Expr) -> Ty {
        todo!()
    }

    fn fresh(&mut self) -> Ty {
        let var = self.next_var;
        self.next_var += 1;
        Ty::Var(var)
    }

    fn convert_ty(&mut self, ty: &ast::Ty) -> Ty {
        match &ty.kind {
            TyKind::App(base, args) => Ty::App(
                Box::new(self.convert_ty(base)),
                args.iter().map(|ty| self.convert_ty(ty)).collect(),
            ),
            TyKind::Arrow(kind, input, output, functors) => Ty::Arrow(
                *kind,
                Box::new(self.convert_ty(input)),
                Box::new(self.convert_ty(output)),
                functors.clone(),
            ),
            TyKind::Hole => self.fresh(),
            TyKind::Paren(inner) => self.convert_ty(inner),
            TyKind::Path(path) => Ty::DefId(
                *self
                    .resolutions
                    .get(&path.id)
                    .expect("path should be resolved"),
            ),
            &TyKind::Prim(prim) => Ty::Prim(prim),
            TyKind::Tuple(items) => {
                Ty::Tuple(items.iter().map(|item| self.convert_ty(item)).collect())
            }
            TyKind::Var(name) => Ty::Rigid(name.name.clone()),
        }
    }

    fn constrain(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    fn solve(self) -> Tys {
        let mut substs = HashMap::new();
        let mut pending_classes: HashMap<_, Vec<_>> = HashMap::new();
        let mut constraints = self.constraints;
        let mut new_constraints = Vec::new();

        loop {
            for constraint in constraints {
                match constraint {
                    Constraint::Eq(ty1, ty2) => {
                        let ty1 = substitute(&substs, ty1);
                        let ty2 = substitute(&substs, ty2);
                        let new_substs = unify(ty1, ty2);

                        for (var, _) in &new_substs {
                            if let Some(classes) = pending_classes.remove(var) {
                                new_constraints.extend(classes.into_iter().map(Constraint::Class));
                            }
                        }

                        substs.extend(new_substs);
                    }
                    Constraint::Class(class) => {
                        let unsolved: Vec<_> = class
                            .dependencies()
                            .into_iter()
                            .filter_map(|ty| is_unsolved(&substs, ty))
                            .collect();

                        if unsolved.is_empty() {
                            new_constraints
                                .extend(classify(class.map(|ty| substitute(&substs, ty))));
                        } else {
                            for var in unsolved {
                                pending_classes.entry(var).or_default().push(class.clone());
                            }
                        }
                    }
                }
            }

            if new_constraints.is_empty() {
                break;
            }

            constraints = mem::take(&mut new_constraints);
        }

        self.tys
            .into_iter()
            .map(|(id, ty)| (id, substitute(&substs, ty)))
            .collect()
    }
}

fn unify(ty1: Ty, ty2: Ty) -> Vec<(u32, Ty)> {
    match (ty1, ty2) {
        (Ty::App(base1, args1), Ty::App(base2, args2)) if args1.len() == args2.len() => {
            let mut substs = unify(*base1, *base2);
            for (arg1, arg2) in args1.into_iter().zip(args2) {
                substs.extend(unify(arg1, arg2));
            }
            substs
        }
        (
            Ty::Arrow(kind1, input1, output1, functors1),
            Ty::Arrow(kind2, input2, output2, functors2),
        ) if kind1 == kind2
            && functor_set(functors1.as_ref()) == functor_set(functors2.as_ref()) =>
        {
            let mut substs = unify(*input1, *input2);
            substs.extend(unify(*output1, *output2));
            substs
        }
        (Ty::DefId(def1), Ty::DefId(def2)) if def1 == def2 => Vec::new(),
        (Ty::Prim(prim1), Ty::Prim(prim2)) if prim1 == prim2 => Vec::new(),
        (Ty::Tuple(items1), Ty::Tuple(items2)) if items1.len() == items2.len() => {
            let mut substs = Vec::new();
            for (item1, item2) in items1.into_iter().zip(items2) {
                substs.extend(unify(item1, item2));
            }
            substs
        }
        (Ty::Var(var1), Ty::Var(var2)) if var1 == var2 => Vec::new(),
        (Ty::Var(var), ty) | (ty, Ty::Var(var)) => vec![(var, ty)],
        (Ty::Void, Ty::Void) => Vec::new(),
        _ => panic!("types do not unify"),
    }
}

fn classify(class: Class) -> Vec<Constraint> {
    match class {
        Class::Eq(Ty::Prim(
            TyPrim::BigInt
            | TyPrim::Bool
            | TyPrim::Double
            | TyPrim::Int
            | TyPrim::Qubit
            | TyPrim::Result
            | TyPrim::String
            | TyPrim::Pauli,
        ))
        | Class::Integral(Ty::Prim(TyPrim::BigInt | TyPrim::Int))
        | Class::Num(Ty::Prim(TyPrim::BigInt | TyPrim::Double | TyPrim::Int)) => Vec::new(),
        Class::Add(Ty::Prim(TyPrim::BigInt | TyPrim::Double | TyPrim::Int | TyPrim::String)) => {
            Vec::new()
        }
        Class::Add(Ty::App(base, _)) if matches!(*base, Ty::Prim(TyPrim::Array)) => Vec::new(),
        Class::Adj(Ty::Arrow(_, _, _, functors))
            if functor_set(functors.as_ref()).contains(&Functor::Adj) =>
        {
            Vec::new()
        }
        Class::Call {
            callee: Ty::Arrow(_, callee_input, callee_output, _),
            input,
            output,
        } => vec![
            Constraint::Eq(input, *callee_input),
            Constraint::Eq(output, *callee_output),
        ],
        Class::Ctl {
            op: Ty::Arrow(kind, input, output, functors),
            with_ctls,
        } if functor_set(functors.as_ref()).contains(&Functor::Ctl) => {
            let qubit_array = Ty::App(
                Box::new(Ty::Prim(TyPrim::Array)),
                vec![Ty::Prim(TyPrim::Qubit)],
            );
            let ctl_input = Box::new(Ty::Tuple(vec![qubit_array, *input]));
            vec![Constraint::Eq(
                with_ctls,
                Ty::Arrow(kind, ctl_input, output, functors),
            )]
        }
        Class::HasField { .. } => todo!("user-defined types not supported"),
        Class::HasFunctorsIfOp { callee, functors } => match callee {
            Ty::Arrow(CallableKind::Operation, _, _, callee_functors)
                if functor_set(callee_functors.as_ref()).is_superset(&functors) =>
            {
                Vec::new()
            }
            Ty::Arrow(CallableKind::Operation, _, _, _) => panic!("operation is missing functors"),
            _ => Vec::new(),
        },
        Class::HasIndex {
            container: Ty::App(base, mut args),
            index,
            item,
        } if matches!(*base, Ty::Prim(TyPrim::Array)) && args.len() == 1 => match index {
            Ty::Prim(TyPrim::Int) => vec![Constraint::Eq(
                args.pop().expect("type arguments should not be empty"),
                item,
            )],
            Ty::Prim(TyPrim::Range) => vec![Constraint::Eq(Ty::App(base, args), item)],
            _ => panic!("invalid index for array"),
        },
        Class::HasPartialApp { .. } => todo!("partial application not supported"),
        Class::Iterable {
            container: Ty::Prim(TyPrim::Range),
            item,
        } => vec![Constraint::Eq(Ty::Prim(TyPrim::Int), item)],
        Class::Iterable {
            container: Ty::App(base, mut args),
            item,
        } if matches!(*base, Ty::Prim(TyPrim::Array)) => {
            vec![Constraint::Eq(
                args.pop().expect("type arguments should not be empty"),
                item,
            )]
        }
        Class::Unwrap { .. } => todo!("user-defined types not supported"),
        _ => panic!("falsified class"),
    }
}

fn substitute(substs: &HashMap<u32, Ty>, ty: Ty) -> Ty {
    match ty {
        Ty::App(base, args) => Ty::App(
            Box::new(substitute(substs, *base)),
            args.into_iter()
                .map(|arg| substitute(substs, arg))
                .collect(),
        ),
        Ty::Arrow(kind, input, output, functors) => Ty::Arrow(
            kind,
            Box::new(substitute(substs, *input)),
            Box::new(substitute(substs, *output)),
            functors,
        ),
        Ty::DefId(id) => Ty::DefId(id),
        Ty::Prim(prim) => Ty::Prim(prim),
        Ty::Rigid(name) => Ty::Rigid(name),
        Ty::Tuple(items) => Ty::Tuple(
            items
                .into_iter()
                .map(|item| substitute(substs, item))
                .collect(),
        ),
        Ty::Var(var) => match substs.get(&var) {
            Some(new_ty) => new_ty.clone(),
            None => Ty::Var(var),
        },
        Ty::Void => Ty::Void,
    }
}

fn functor_set(expr: Option<&FunctorExpr>) -> HashSet<Functor> {
    match expr {
        None => HashSet::new(),
        Some(expr) => match &expr.kind {
            FunctorExprKind::BinOp(op, lhs, rhs) => {
                let lhs = functor_set(Some(lhs));
                let rhs = functor_set(Some(rhs));
                match op {
                    SetOp::Union => lhs.union(&rhs).copied().collect(),
                    SetOp::Intersect => lhs.intersection(&rhs).copied().collect(),
                }
            }
            &FunctorExprKind::Lit(functor) => HashSet::from([functor]),
            FunctorExprKind::Paren(expr) => functor_set(Some(expr)),
        },
    }
}

fn is_unsolved(substs: &HashMap<u32, Ty>, ty: &Ty) -> Option<u32> {
    match ty {
        &Ty::Var(var) if !substs.contains_key(&var) => Some(var),
        _ => None,
    }
}

fn callable_ty(resolutions: &Resolutions, decl: &CallableDecl) -> Option<Ty> {
    let input = try_pat_ty(resolutions, &decl.input)?;
    let output = try_convert_ty(resolutions, &decl.output)?;
    Some(Ty::Arrow(
        decl.kind,
        Box::new(input),
        Box::new(output),
        decl.functors.clone(),
    ))
}

fn try_convert_ty(resolutions: &Resolutions, ty: &ast::Ty) -> Option<Ty> {
    match &ty.kind {
        TyKind::App(base, args) => {
            let base = try_convert_ty(resolutions, base)?;
            let args = args
                .iter()
                .map(|arg| try_convert_ty(resolutions, arg))
                .collect::<Option<_>>()?;
            Some(Ty::App(Box::new(base), args))
        }
        TyKind::Arrow(kind, input, output, functors) => {
            let input = try_convert_ty(resolutions, input)?;
            let output = try_convert_ty(resolutions, output)?;
            Some(Ty::Arrow(
                *kind,
                Box::new(input),
                Box::new(output),
                functors.clone(),
            ))
        }
        TyKind::Hole => None,
        TyKind::Paren(inner) => try_convert_ty(resolutions, inner),
        TyKind::Path(path) => Some(Ty::DefId(
            *resolutions.get(&path.id).expect("path should be resolved"),
        )),
        &TyKind::Prim(prim) => Some(Ty::Prim(prim)),
        TyKind::Tuple(items) => {
            let items = items
                .iter()
                .map(|item| try_convert_ty(resolutions, item))
                .collect::<Option<_>>()?;
            Some(Ty::Tuple(items))
        }
        TyKind::Var(name) => Some(Ty::Rigid(name.name.clone())),
    }
}

fn try_pat_ty(resolutions: &Resolutions, pat: &Pat) -> Option<Ty> {
    match &pat.kind {
        PatKind::Bind(_, None) | PatKind::Discard(None) | PatKind::Elided => None,
        PatKind::Bind(_, Some(ty)) | PatKind::Discard(Some(ty)) => try_convert_ty(resolutions, ty),
        PatKind::Paren(inner) => try_pat_ty(resolutions, inner),
        PatKind::Tuple(items) => items
            .iter()
            .map(|item| try_pat_ty(resolutions, item))
            .collect::<Option<_>>()
            .map(Ty::Tuple),
    }
}
