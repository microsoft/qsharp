// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod ctl_gen;

#[cfg(test)]
mod tests;

use self::ctl_gen::CtlDistrib;
use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_frontend::compile::{CompileUnit, Context};
use qsc_hir::{
    hir::{
        Block, CallableBody, CallableDecl, Functor, FunctorExprKind, Ident, NodeId, Package, Pat,
        PatKind, PrimTy, Res, SetOp, Spec, SpecBody, SpecDecl, SpecGen, Ty,
    },
    mut_visit::MutVisitor,
};
use std::{collections::HashSet, option::Option};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[diagnostic(transparent)]
    #[error(transparent)]
    CtlGen(ctl_gen::Error),

    #[error("missing body implementation")]
    MissingBody(#[label("specialization generation requires body implementation")] Span),
}

/// Generates specializations for the given compile unit, updating it in-place.
pub fn generate_specs(unit: &mut CompileUnit) -> Vec<Error> {
    generate_placeholders(unit);

    // TODO: Generating specialization violates the invariant of node ids being unique because of how
    // it depends on cloning parts of the tree. We should update this when HIR supports the notion of
    // generating new, properly mapped node ids such the uniqueness invariant is preserved without the burden
    // of keeping out-of-band type and symbol resolution context updated.
    generate_spec_impls(unit)
}

fn generate_placeholders(unit: &mut CompileUnit) {
    SpecPlacePass.visit_package(&mut unit.package);
}

struct SpecPlacePass;

impl MutVisitor for SpecPlacePass {
    fn visit_callable_decl(&mut self, decl: &mut CallableDecl) {
        if let Some(functors) = &decl.functors {
            let mut func_set = HashSet::new();
            collect_functors(&functors.kind, &mut func_set);
            let is_adj = func_set.contains(&Functor::Adj);
            let is_ctl = func_set.contains(&Functor::Ctl);
            let is_ctladj = is_adj && is_ctl;

            let mut spec_decl = match &decl.body {
                CallableBody::Block(body) => vec![SpecDecl {
                    id: NodeId::default(),
                    span: body.span,
                    spec: Spec::Body,
                    body: SpecBody::Impl(
                        Pat {
                            id: NodeId::default(),
                            span: body.span,
                            ty: decl.input.ty.clone(),
                            kind: PatKind::Elided,
                        },
                        body.clone(),
                    ),
                }],
                CallableBody::Specs(spec_decl) => spec_decl.clone(),
            };

            if is_adj && spec_decl.iter().all(|s| s.spec != Spec::Adj) {
                spec_decl.push(SpecDecl {
                    id: NodeId::default(),
                    span: decl.span,
                    spec: Spec::Adj,
                    body: SpecBody::Gen(SpecGen::Invert),
                });
            }

            if is_ctl && spec_decl.iter().all(|s| s.spec != Spec::Ctl) {
                spec_decl.push(SpecDecl {
                    id: NodeId::default(),
                    span: decl.span,
                    spec: Spec::Ctl,
                    body: SpecBody::Gen(SpecGen::Distribute),
                });
            }

            if is_ctladj && spec_decl.iter().all(|s| s.spec != Spec::CtlAdj) {
                let gen = if is_self_adjoint(&spec_decl) {
                    SpecGen::Slf
                } else {
                    SpecGen::Distribute
                };
                spec_decl.push(SpecDecl {
                    id: NodeId::default(),
                    span: decl.span,
                    spec: Spec::CtlAdj,
                    body: SpecBody::Gen(gen),
                });
            }

            decl.body = CallableBody::Specs(spec_decl);
        }
    }
}

fn collect_functors(func_kind: &FunctorExprKind, set: &mut HashSet<Functor>) {
    match func_kind {
        FunctorExprKind::BinOp(op, lhs, rhs) => match op {
            SetOp::Union => {
                collect_functors(&lhs.kind, set);
                collect_functors(&rhs.kind, set);
            }
            SetOp::Intersect => {
                let mut lhs_set = HashSet::new();
                let mut rhs_set = HashSet::new();
                collect_functors(&lhs.kind, &mut lhs_set);
                collect_functors(&rhs.kind, &mut rhs_set);
                set.extend(lhs_set.intersection(&rhs_set));
            }
        },
        FunctorExprKind::Lit(func) => {
            set.insert(*func);
        }
        FunctorExprKind::Paren(func) => collect_functors(&func.kind, set),
    }
}

fn is_self_adjoint(spec_decl: &[SpecDecl]) -> bool {
    spec_decl
        .iter()
        .any(|s| s.spec == Spec::Adj && s.body == SpecBody::Gen(SpecGen::Slf))
}

fn generate_spec_impls(unit: &mut CompileUnit) -> Vec<Error> {
    let mut pass = SpecImplPass {
        context: &mut unit.context,
        errors: Vec::new(),
    };
    pass.transform(&mut unit.package);
    pass.errors
}

struct SpecImplPass<'a> {
    context: &'a mut Context,
    errors: Vec<Error>,
}

impl<'a> SpecImplPass<'a> {
    fn transform(&mut self, package: &mut Package) {
        for ns in &mut package.namespaces {
            self.visit_namespace(ns);
        }
    }

    fn ctl_distrib(&mut self, input_ty: Ty, spec_decl: &mut SpecDecl, block: &Block) {
        // Clone the reference block and use the pass to update the calls inside.
        let ctls_id = self.context.assigner_mut().next_id();
        let mut ctl_block = block.clone();
        let mut distrib = CtlDistrib {
            ctls: Res::Internal(ctls_id),
            errors: Vec::new(),
        };
        distrib.visit_block(&mut ctl_block);
        self.errors
            .extend(distrib.errors.into_iter().map(Error::CtlGen));

        // Update the specialization body to reflect the generated block.
        spec_decl.body = SpecBody::Impl(
            Pat {
                id: NodeId::default(),
                span: spec_decl.span,
                ty: Ty::Tuple(vec![
                    Ty::Array(Box::new(Ty::Prim(PrimTy::Qubit))),
                    input_ty.clone(),
                ]),
                kind: PatKind::Tuple(vec![
                    Pat {
                        id: NodeId::default(),
                        span: spec_decl.span,
                        ty: Ty::Array(Box::new(Ty::Prim(PrimTy::Qubit))),
                        kind: PatKind::Bind(Ident {
                            id: ctls_id,
                            span: spec_decl.span,
                            name: "ctls".to_string(),
                        }),
                    },
                    Pat {
                        id: NodeId::default(),
                        span: spec_decl.span,
                        ty: input_ty,
                        kind: PatKind::Elided,
                    },
                ]),
            },
            ctl_block,
        );
    }
}

impl<'a> MutVisitor for SpecImplPass<'a> {
    fn visit_callable_decl(&mut self, decl: &mut CallableDecl) {
        if let CallableBody::Specs(spec_decls) = &mut decl.body {
            let (mut body, mut adj, mut ctl, mut ctladj) = (None, None, None, None);
            for spec_decl in spec_decls.drain(0..) {
                match spec_decl.spec {
                    Spec::Body => body = Some(spec_decl),
                    Spec::Adj => adj = Some(spec_decl),
                    Spec::Ctl => ctl = Some(spec_decl),
                    Spec::CtlAdj => ctladj = Some(spec_decl),
                }
            }

            let Some(body) = body else {
                    self.errors.push(Error::MissingBody(decl.span));
                    return;
                };
            let SpecBody::Impl(_, body_block) = &body.body else {
                    if body.body == SpecBody::Gen(SpecGen::Intrinsic) && [adj, ctl, ctladj].iter().any(Option::is_some) {
                        self.errors.push(Error::MissingBody(body.span));
                    } else {
                        spec_decls.push(body);
                    }
                    return;
                };

            if let Some(ctl) = ctl.as_mut() {
                if ctl.body == SpecBody::Gen(SpecGen::Distribute)
                    || ctl.body == SpecBody::Gen(SpecGen::Auto)
                {
                    self.ctl_distrib(decl.input.ty.clone(), ctl, body_block);
                }
            };

            if let Some(adj) = adj.as_mut() {
                if adj.body == SpecBody::Gen(SpecGen::Slf) {
                    adj.body = body.body.clone();
                }
            }

            if let (Some(ctladj), Some(adj), Some(ctl)) = (ctladj.as_mut(), &adj, &ctl) {
                match &ctladj.body {
                    SpecBody::Gen(SpecGen::Auto | SpecGen::Distribute) => {
                        if let SpecBody::Impl(_, adj_block) = &adj.body {
                            self.ctl_distrib(decl.input.ty.clone(), ctladj, adj_block);
                        }
                    }
                    SpecBody::Gen(SpecGen::Slf) => ctladj.body = ctl.body.clone(),
                    _ => {}
                }
            };

            *spec_decls = vec![body];
            adj.into_iter().for_each(|spec| spec_decls.push(spec));
            ctl.into_iter().for_each(|spec| spec_decls.push(spec));
            ctladj.into_iter().for_each(|spec| spec_decls.push(spec));
        }
    }
}
