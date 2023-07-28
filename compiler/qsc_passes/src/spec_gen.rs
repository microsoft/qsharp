// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub(crate) mod adj_gen;
mod ctl_gen;

#[cfg(test)]
mod tests;

use crate::{id_update::NodeIdRefresher, invert_block::adj_invert_block};

use self::{adj_gen::AdjDistrib, ctl_gen::CtlDistrib};
use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_frontend::compile::CompileUnit;
use qsc_hir::{
    assigner::Assigner,
    global::Table,
    hir::{
        Block, CallableDecl, CallableKind, Functor, Ident, NodeId, Package, Pat, PatKind, Res,
        SpecBody, SpecDecl, SpecGen,
    },
    mut_visit::MutVisitor,
    ty::{Prim, Ty},
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
    #[diagnostic(code("Qsc.SpecGen.MissingBody"))]
    MissingBody(#[label] Span),
}

/// Generates specializations for the given compile unit, updating it in-place.
pub(super) fn generate_specs(core: &Table, unit: &mut CompileUnit) -> Vec<Error> {
    generate_placeholders(&mut unit.package, &mut unit.assigner);
    generate_spec_impls(core, unit)
}

pub(super) fn generate_specs_for_callable(
    core: &Table,
    assigner: &mut Assigner,
    decl: &mut CallableDecl,
) -> Vec<Error> {
    generate_placeholders_for_callable(decl, assigner);
    generate_spec_impls_for_decl(core, assigner, decl)
}

fn generate_placeholders(package: &mut Package, assigner: &mut Assigner) {
    SpecPlacePass { assigner }.visit_package(package);
}

fn generate_placeholders_for_callable(decl: &mut CallableDecl, assigner: &mut Assigner) {
    SpecPlacePass { assigner }.visit_callable_decl(decl);
}

struct SpecPlacePass<'a> {
    assigner: &'a mut Assigner,
}

impl MutVisitor for SpecPlacePass<'_> {
    fn visit_callable_decl(&mut self, decl: &mut CallableDecl) {
        // Only applies to operations.
        if decl.kind == CallableKind::Function {
            return;
        }

        let is_adj = decl.functors.contains(&Functor::Adj);
        let is_ctl = decl.functors.contains(&Functor::Ctl);
        if !is_adj && !is_ctl {
            return;
        }

        if is_adj && decl.adj.is_none() {
            decl.adj = Some(SpecDecl {
                id: self.assigner.next_node(),
                span: decl.span,
                body: SpecBody::Gen(SpecGen::Invert),
            });
        }

        if is_ctl && decl.ctl.is_none() {
            decl.ctl = Some(SpecDecl {
                id: self.assigner.next_node(),
                span: decl.span,
                body: SpecBody::Gen(SpecGen::Distribute),
            });
        }

        let has_explicit_adj =
            matches!(&decl.adj, Some(s) if matches!(&s.body, SpecBody::Impl(..)));
        let has_explicit_ctl =
            matches!(&decl.ctl, Some(s) if matches!(&s.body, SpecBody::Impl(..)));
        let has_explicit_ctl_adj =
            matches!(&decl.ctl_adj, Some(s) if !matches!(&s.body, SpecBody::Gen(SpecGen::Auto)));

        if is_adj && is_ctl && !has_explicit_ctl_adj {
            let gen = if is_self_adjoint(decl) {
                SpecGen::Slf
            } else if has_explicit_ctl && !has_explicit_adj {
                SpecGen::Invert
            } else {
                SpecGen::Distribute
            };
            decl.ctl_adj = Some(SpecDecl {
                id: self.assigner.next_node(),
                span: decl.span,
                body: SpecBody::Gen(gen),
            });
        }
    }
}

fn is_self_adjoint(decl: &CallableDecl) -> bool {
    matches!(&decl.adj, Some(s) if matches!(&s.body, SpecBody::Gen(SpecGen::Slf)))
}

fn generate_spec_impls(core: &Table, unit: &mut CompileUnit) -> Vec<Error> {
    let mut pass = SpecImplPass {
        core,
        assigner: &mut unit.assigner,
        errors: Vec::new(),
    };
    pass.visit_package(&mut unit.package);
    pass.errors
}

fn generate_spec_impls_for_decl(
    core: &Table,
    assigner: &mut Assigner,
    decl: &mut CallableDecl,
) -> Vec<Error> {
    let mut pass = SpecImplPass {
        core,
        assigner,
        errors: Vec::new(),
    };
    pass.visit_callable_decl(decl);
    pass.errors
}

struct SpecImplPass<'a> {
    core: &'a Table,
    assigner: &'a mut Assigner,
    errors: Vec<Error>,
}

impl<'a> SpecImplPass<'a> {
    fn ctl_distrib(&mut self, spec_decl: &mut SpecDecl, block: &Block) {
        let ctls_id = self.assigner.next_node();

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
            Some(Pat {
                id: NodeId::default(),
                span: spec_decl.span,
                ty: Ty::Array(Box::new(Ty::Prim(Prim::Qubit))),
                kind: PatKind::Bind(Ident {
                    id: ctls_id,
                    span: spec_decl.span,
                    name: "ctls".into(),
                }),
            }),
            ctl_block,
        );
    }

    fn adj_invert(&mut self, spec_decl: &mut SpecDecl, block: &Block, ctls_pat: Option<Pat>) {
        // Clone the reference block and use the pass to update the calls inside.
        let mut adj_block = block.clone();
        if let Err(invert_errors) = adj_invert_block(self.core, self.assigner, &mut adj_block) {
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
        spec_decl.body = SpecBody::Impl(ctls_pat, adj_block);
    }
}

impl<'a> MutVisitor for SpecImplPass<'a> {
    fn visit_callable_decl(&mut self, decl: &mut CallableDecl) {
        // Only applies to operations.
        if decl.kind == CallableKind::Function {
            return;
        }

        let body = &decl.body;
        let adj = &mut decl.adj;
        let ctl = &mut decl.ctl;
        let ctl_adj = &mut decl.ctl_adj;

        let SpecBody::Impl(_, body_block) = &body.body else {
                if body.body == SpecBody::Gen(SpecGen::Intrinsic) && [adj, ctl, ctl_adj].into_iter().any(|x| Option::is_some(x)) {
                    self.errors.push(Error::MissingBody(body.span));
                }
                return;
            };

        if let Some(ctl) = ctl.as_mut() {
            if ctl.body == SpecBody::Gen(SpecGen::Distribute)
                || ctl.body == SpecBody::Gen(SpecGen::Auto)
            {
                self.ctl_distrib(ctl, body_block);
                NodeIdRefresher::new(self.assigner).visit_spec_decl(ctl);
            }
        };

        if let Some(adj) = adj.as_mut() {
            if adj.body == SpecBody::Gen(SpecGen::Slf) {
                adj.body = body.body.clone();
                NodeIdRefresher::new(self.assigner).visit_spec_decl(adj);
            } else if adj.body == SpecBody::Gen(SpecGen::Invert)
                || adj.body == SpecBody::Gen(SpecGen::Auto)
            {
                self.adj_invert(adj, body_block, None);
                NodeIdRefresher::new(self.assigner).visit_spec_decl(adj);
            }
        }

        if let (Some(ctl_adj), Some(adj), Some(ctl)) = (ctl_adj.as_mut(), &adj, &ctl) {
            match &ctl_adj.body {
                SpecBody::Gen(SpecGen::Auto | SpecGen::Distribute) => {
                    if let SpecBody::Impl(_, adj_block) = &adj.body {
                        self.ctl_distrib(ctl_adj, adj_block);
                        NodeIdRefresher::new(self.assigner).visit_spec_decl(ctl_adj);
                    }
                }
                SpecBody::Gen(SpecGen::Slf) => {
                    ctl_adj.body = ctl.body.clone();
                    NodeIdRefresher::new(self.assigner).visit_spec_decl(ctl_adj);
                }
                SpecBody::Gen(SpecGen::Invert) => {
                    if let SpecBody::Impl(pat, ctl_block) = &ctl.body {
                        self.adj_invert(ctl_adj, ctl_block, pat.clone());
                        NodeIdRefresher::new(self.assigner).visit_spec_decl(ctl_adj);
                    }
                }
                _ => {}
            }
        };
    }
}
