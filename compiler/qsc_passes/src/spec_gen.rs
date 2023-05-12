// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub(crate) mod adj_gen;
mod ctl_gen;

#[cfg(test)]
mod tests;

use crate::invert_block::adj_invert_block;

use self::{adj_gen::AdjDistrib, ctl_gen::CtlDistrib};
use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_frontend::compile::CompileUnit;
use qsc_hir::{
    assigner::Assigner,
    hir::{
        Block, CallableBody, CallableDecl, Functor, Ident, NodeId, Pat, PatKind, PrimTy, Res, Spec,
        SpecBody, SpecDecl, SpecGen, Ty,
    },
    mut_visit::MutVisitor,
};
use std::option::Option;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[diagnostic(transparent)]
    #[error(transparent)]
    CtlGen(ctl_gen::Error),

    #[diagnostic(transparent)]
    #[error(transparent)]
    AdjGen(adj_gen::Error),

    #[error("specialization generation missing required body implementation")]
    MissingBody(#[label] Span),
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

pub fn generate_specs_for_callable(assigner: &mut Assigner, decl: &mut CallableDecl) -> Vec<Error> {
    generate_placeholders_for_callable(decl);
    generate_spec_impls_for_decl(assigner, decl)
}

fn generate_placeholders(unit: &mut CompileUnit) {
    SpecPlacePass.visit_package(&mut unit.package);
}

fn generate_placeholders_for_callable(decl: &mut CallableDecl) {
    SpecPlacePass.visit_callable_decl(decl);
}

struct SpecPlacePass;

impl MutVisitor for SpecPlacePass {
    fn visit_callable_decl(&mut self, decl: &mut CallableDecl) {
        if decl.functors.is_empty() {
            return;
        }

        let is_adj = decl.functors.contains(&Functor::Adj);
        let is_ctl = decl.functors.contains(&Functor::Ctl);
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

        let has_explicit_adj = spec_decl
            .iter()
            .any(|s| s.spec == Spec::Adj && matches!(s.body, SpecBody::Impl(..)));
        let has_explicit_ctl = spec_decl
            .iter()
            .any(|s| s.spec == Spec::Ctl && matches!(s.body, SpecBody::Impl(..)));

        if is_ctladj && spec_decl.iter().all(|s| s.spec != Spec::CtlAdj) {
            let gen = if is_self_adjoint(&spec_decl) {
                SpecGen::Slf
            } else if has_explicit_ctl && !has_explicit_adj {
                SpecGen::Invert
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

fn is_self_adjoint(spec_decl: &[SpecDecl]) -> bool {
    spec_decl
        .iter()
        .any(|s| s.spec == Spec::Adj && s.body == SpecBody::Gen(SpecGen::Slf))
}

fn generate_spec_impls(unit: &mut CompileUnit) -> Vec<Error> {
    let mut pass = SpecImplPass {
        assigner: &mut unit.assigner,
        errors: Vec::new(),
    };
    pass.visit_package(&mut unit.package);
    pass.errors
}

fn generate_spec_impls_for_decl(assigner: &mut Assigner, decl: &mut CallableDecl) -> Vec<Error> {
    let mut pass = SpecImplPass {
        assigner,
        errors: Vec::new(),
    };
    pass.visit_callable_decl(decl);
    pass.errors
}

struct SpecImplPass<'a> {
    assigner: &'a mut Assigner,
    errors: Vec<Error>,
}

impl<'a> SpecImplPass<'a> {
    fn ctl_distrib(&mut self, input_ty: Ty, spec_decl: &mut SpecDecl, block: &Block) {
        let ctls_id = self.assigner.next_id();

        // Clone the reference block and use the pass to update the calls inside.
        let mut ctl_block = block.clone();
        let mut distrib = CtlDistrib {
            ctls: Res::Local(ctls_id),
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
                            name: "ctls".into(),
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

    fn adj_invert(
        &mut self,
        input_ty: Ty,
        spec_decl: &mut SpecDecl,
        block: &Block,
        ctls_pat: Option<&Pat>,
    ) {
        // Clone the reference block and use the pass to update the calls inside.
        let mut adj_block = block.clone();
        if let Err(invert_errors) = adj_invert_block(self.assigner, &mut adj_block) {
            self.errors.extend(
                invert_errors
                    .into_iter()
                    .map(adj_gen::Error::LogicSep)
                    .map(Error::AdjGen),
            );
            return;
        }
        let mut distrib = AdjDistrib { errors: Vec::new() };
        distrib.visit_block(&mut adj_block);
        self.errors
            .extend(distrib.errors.into_iter().map(Error::AdjGen));

        // Update the specialization body to reflect the generated block.
        spec_decl.body = SpecBody::Impl(
            if let Some(pat) = ctls_pat {
                pat.clone()
            } else {
                Pat {
                    id: NodeId::default(),
                    ty: input_ty,
                    span: spec_decl.span,
                    kind: PatKind::Elided,
                }
            },
            adj_block,
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
                } else if adj.body == SpecBody::Gen(SpecGen::Invert) {
                    self.adj_invert(decl.input.ty.clone(), adj, body_block, None);
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
                    SpecBody::Gen(SpecGen::Invert) => {
                        if let SpecBody::Impl(pat, ctl_block) = &ctl.body {
                            self.adj_invert(decl.input.ty.clone(), ctladj, ctl_block, Some(pat));
                        }
                    }
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
