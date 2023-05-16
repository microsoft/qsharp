// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::span::Span;
use qsc_hir::{
    assigner::Assigner,
    hir::{
        Block, CallableBody, CallableDecl, CallableKind, Expr, ExprKind, Ident, NodeId, Pat,
        PatKind, Res, Stmt, StmtKind, Ty,
    },
    mut_visit::{self, MutVisitor},
    visit::{self, Visitor},
};
use std::{
    collections::{HashMap, HashSet},
    iter,
};

struct VarFinder {
    free: HashMap<NodeId, Ty>,
    bound: HashSet<NodeId>,
}

impl Visitor<'_> for VarFinder {
    fn visit_expr(&mut self, expr: &Expr) {
        if let ExprKind::Var(Res::Local(id)) = expr.kind {
            if !self.bound.contains(&id) {
                self.free.insert(id, expr.ty.clone());
            }
        } else {
            visit::walk_expr(self, expr);
        }
    }

    fn visit_pat(&mut self, pat: &Pat) {
        if let PatKind::Bind(name) = &pat.kind {
            self.free.remove(&name.id);
            self.bound.insert(name.id);
        } else {
            visit::walk_pat(self, pat);
        }
    }
}

struct VarReplacer<'a> {
    substitutions: &'a HashMap<NodeId, NodeId>,
}

impl MutVisitor for VarReplacer<'_> {
    fn visit_expr(&mut self, expr: &mut Expr) {
        if let ExprKind::Var(Res::Local(id)) = &mut expr.kind {
            if let Some(&new_id) = self.substitutions.get(id) {
                *id = new_id;
            }
        } else {
            mut_visit::walk_expr(self, expr);
        }
    }
}

pub(super) fn lift(
    assigner: &mut Assigner,
    kind: CallableKind,
    input: Pat,
    mut body: Expr,
    span: Span,
) -> (Vec<NodeId>, CallableDecl) {
    let mut finder = VarFinder {
        free: HashMap::new(),
        bound: HashSet::new(),
    };
    finder.visit_pat(&input);
    finder.visit_expr(&body);

    let substitutions: HashMap<_, _> = finder
        .free
        .iter()
        .map(|(&var, _)| (var, assigner.next_id()))
        .collect();

    VarReplacer {
        substitutions: &substitutions,
    }
    .visit_expr(&mut body);

    let free_vars = finder.free.keys().copied().collect();
    let input = close_params(
        finder.free.into_iter().map(|(id, ty)| {
            let &new_id = substitutions
                .get(&id)
                .expect("free variable should have substitution");
            (new_id, ty)
        }),
        input,
        span,
    );

    let callable = CallableDecl {
        id: assigner.next_id(),
        span,
        kind,
        name: Ident {
            id: assigner.next_id(),
            span,
            name: "lambda".into(),
        },
        ty_params: Vec::new(),
        input,
        output: body.ty.clone(),
        functors: HashSet::new(),
        body: CallableBody::Block(Block {
            id: assigner.next_id(),
            span: body.span,
            ty: body.ty.clone(),
            stmts: vec![Stmt {
                id: assigner.next_id(),
                span: body.span,
                kind: StmtKind::Expr(body),
            }],
        }),
    };

    (free_vars, callable)
}

fn close_params(vars: impl IntoIterator<Item = (NodeId, Ty)>, input: Pat, span: Span) -> Pat {
    let bindings: Vec<_> = vars
        .into_iter()
        .map(|(id, ty)| Pat {
            id: NodeId::default(),
            span,
            ty,
            kind: PatKind::Bind(Ident {
                id,
                span,
                name: "var".into(),
            }),
        })
        .collect();

    let ty = Ty::Tuple(
        bindings
            .iter()
            .map(|p| p.ty.clone())
            .chain(iter::once(input.ty.clone()))
            .collect(),
    );

    let kind = PatKind::Tuple(bindings.into_iter().chain(iter::once(input)).collect());
    Pat {
        id: NodeId::default(),
        span,
        ty,
        kind,
    }
}
