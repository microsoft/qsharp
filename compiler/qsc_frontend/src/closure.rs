// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::lower::Local;
use qsc_data_structures::{index_map::IndexMap, span::Span};
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
    kind: CallableKind,
    input: Pat,
    mut body: Expr,
    span: Span,
) -> (Vec<NodeId>, CallableDecl) {
    let mut finder = VarFinder {
        bindings: HashSet::new(),
        uses: HashSet::new(),
    };
    finder.visit_pat(&input);
    finder.visit_expr(&body);

    let free_vars = finder.free_vars();
    let substitutions: HashMap<_, _> = free_vars
        .iter()
        .map(|&id| (id, assigner.next_id()))
        .collect();

    VarReplacer {
        substitutions: &substitutions,
    }
    .visit_expr(&mut body);

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

    let mut input = close(substituted_vars, input, span);
    assigner.visit_pat(&mut input);

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

fn close(vars: impl IntoIterator<Item = (NodeId, Ty)>, input: Pat, span: Span) -> Pat {
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
