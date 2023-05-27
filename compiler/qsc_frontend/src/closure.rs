// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::lower::Local;
use qsc_data_structures::{index_map::IndexMap, span::Span};
use qsc_hir::{
    assigner::Assigner,
    hir::{
        Block, CallableBody, CallableDecl, CallableKind, Expr, ExprKind, FunctorSet, Ident,
        Mutability, NodeId, Pat, PatKind, Res, Stmt, StmtKind, Ty,
    },
    mut_visit::{self, MutVisitor},
    visit::{self, Visitor},
};
use std::{
    collections::{HashMap, HashSet},
    iter,
};

pub(super) struct Lambda {
    pub(super) kind: CallableKind,
    pub(super) input: Pat,
    pub(super) body: Expr,
    pub(super) functors: FunctorSet,
}

pub(super) struct PartialApp {
    pub(super) bindings: Vec<Stmt>,
    pub(super) input: Option<Pat>,
}

struct VarFinder {
    bindings: HashSet<NodeId>,
    uses: HashSet<NodeId>,
}

impl VarFinder {
    fn free_vars(&self) -> Vec<NodeId> {
        let mut vars: Vec<_> = self.uses.difference(&self.bindings).copied().collect();
        vars.sort_unstable();
        vars
    }
}

impl Visitor<'_> for VarFinder {
    fn visit_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Closure(args, _) => self.uses.extend(args.iter().copied()),
            &ExprKind::Var(Res::Local(id)) => {
                self.uses.insert(id);
            }
            _ => visit::walk_expr(self, expr),
        }
    }

    fn visit_pat(&mut self, pat: &Pat) {
        if let PatKind::Bind(name) = &pat.kind {
            self.bindings.insert(name.id);
        } else {
            visit::walk_pat(self, pat);
        }
    }
}

struct VarReplacer<'a> {
    substitutions: &'a HashMap<NodeId, NodeId>,
}

impl VarReplacer<'_> {
    fn replace(&self, id: &mut NodeId) {
        if let Some(&new_id) = self.substitutions.get(id) {
            *id = new_id;
        }
    }
}

impl MutVisitor for VarReplacer<'_> {
    fn visit_expr(&mut self, expr: &mut Expr) {
        match &mut expr.kind {
            ExprKind::Closure(args, _) => args.iter_mut().for_each(|arg| self.replace(arg)),
            ExprKind::Var(Res::Local(id)) => self.replace(id),
            _ => mut_visit::walk_expr(self, expr),
        }
    }
}

pub(super) fn lift(
    assigner: &mut Assigner,
    locals: &IndexMap<NodeId, Local>,
    mut lambda: Lambda,
    span: Span,
) -> (Vec<NodeId>, CallableDecl) {
    let mut finder = VarFinder {
        bindings: HashSet::new(),
        uses: HashSet::new(),
    };
    finder.visit_pat(&lambda.input);
    finder.visit_expr(&lambda.body);

    let free_vars = finder.free_vars();
    let substitutions: HashMap<_, _> = free_vars
        .iter()
        .map(|&id| (id, assigner.next_node()))
        .collect();

    VarReplacer {
        substitutions: &substitutions,
    }
    .visit_expr(&mut lambda.body);

    let substituted_vars = free_vars.iter().map(|&id| {
        let &new_id = substitutions
            .get(&id)
            .expect("free variable should have substitution");
        let ty = locals
            .get(id)
            .expect("free variable should be a local")
            .ty
            .clone();
        (new_id, ty)
    });

    let mut input = closure_input(substituted_vars, lambda.input, span);
    assigner.visit_pat(&mut input);

    let callable = CallableDecl {
        id: assigner.next_node(),
        span,
        kind: lambda.kind,
        name: Ident {
            id: assigner.next_node(),
            span,
            name: "lambda".into(),
        },
        ty_params: Vec::new(),
        input,
        output: lambda.body.ty.clone(),
        functors: lambda.functors,
        body: CallableBody::Block(Block {
            id: assigner.next_node(),
            span: lambda.body.span,
            ty: lambda.body.ty.clone(),
            stmts: vec![Stmt {
                id: assigner.next_node(),
                span: lambda.body.span,
                kind: StmtKind::Expr(lambda.body),
            }],
        }),
    };

    (free_vars, callable)
}

pub(super) fn partial_app_block(
    close: impl FnOnce(Lambda) -> ExprKind,
    callee: Expr,
    arg: Expr,
    app: PartialApp,
    ty: Ty,
    span: Span,
) -> Block {
    let input = app.input.expect("partial application should have input");
    let Ty::Arrow(kind, _, output, functors) = &ty else {
        panic!("partial application should arrow type");
    };
    let call = Expr {
        id: NodeId::default(),
        span,
        ty: Ty::clone(output),
        kind: ExprKind::Call(Box::new(callee), Box::new(arg)),
    };
    let lambda = Lambda {
        kind: *kind,
        input,
        body: call,
        functors: *functors,
    };
    let closure = Expr {
        id: NodeId::default(),
        span,
        ty: ty.clone(),
        kind: close(lambda),
    };
    let mut stmts = app.bindings;
    stmts.push(Stmt {
        id: NodeId::default(),
        span,
        kind: StmtKind::Expr(closure),
    });
    Block {
        id: NodeId::default(),
        span,
        ty,
        stmts,
    }
}

pub(super) fn partial_app_hole(
    assigner: &mut Assigner,
    locals: &mut IndexMap<NodeId, Local>,
    ty: Ty,
    span: Span,
) -> (Expr, PartialApp) {
    let local_id = assigner.next_node();
    let local = Local {
        mutability: Mutability::Immutable,
        ty: ty.clone(),
    };
    locals.insert(local_id, local);

    let app = PartialApp {
        bindings: Vec::new(),
        input: Some(Pat {
            id: assigner.next_node(),
            span,
            ty: ty.clone(),
            kind: PatKind::Bind(Ident {
                id: local_id,
                span,
                name: "hole".into(),
            }),
        }),
    };

    let var = Expr {
        id: assigner.next_node(),
        span,
        ty,
        kind: ExprKind::Var(Res::Local(local_id)),
    };

    (var, app)
}

fn closure_input(vars: impl IntoIterator<Item = (NodeId, Ty)>, input: Pat, span: Span) -> Pat {
    let bindings: Vec<_> = vars
        .into_iter()
        .map(|(id, ty)| Pat {
            id: NodeId::default(),
            span,
            ty,
            kind: PatKind::Bind(Ident {
                id,
                span,
                name: "closed".into(),
            }),
        })
        .collect();

    let tys = bindings
        .iter()
        .map(|p| p.ty.clone())
        .chain(iter::once(input.ty.clone()))
        .collect();

    Pat {
        id: NodeId::default(),
        span,
        ty: Ty::Tuple(tys),
        kind: PatKind::Tuple(bindings.into_iter().chain(iter::once(input)).collect()),
    }
}
