// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod ctl_gen;

#[cfg(test)]
mod test;

use std::collections::HashSet;

use self::ctl_gen::CtlDistrib;
use qsc_frontend::{
    compile::{CompileUnit, Context},
    resolve::Res,
    typeck::ty::{Prim, Ty},
};
use qsc_hir::{
    hir::{
        Block, CallableBody, CallableDecl, Functor, FunctorExprKind, Ident, Package, Pat, PatKind,
        Path, SetOp, Spec, SpecBody, SpecDecl, SpecGen,
    },
    mut_visit::MutVisitor,
};

/// Generates specializations for the given compile unit, updating it in-place.
pub fn generate_specs(unit: &mut CompileUnit) {
    generate_placeholders(unit);

    generate_spec_impls(unit);
}

fn generate_placeholders(unit: &mut CompileUnit) {
    let mut pass = SpecPlacePass {
        context: &mut unit.context,
    };
    pass.transform(&mut unit.package);
}

struct SpecPlacePass<'a> {
    context: &'a mut Context,
}

impl<'a> SpecPlacePass<'a> {
    fn transform(&mut self, package: &mut Package) {
        for ns in &mut package.namespaces {
            self.visit_namespace(ns);
        }
    }
}

impl<'a> MutVisitor for SpecPlacePass<'a> {
    fn visit_callable_decl(&mut self, decl: &mut CallableDecl) {
        if let Some(functors) = &decl.functors {
            let mut func_set = HashSet::new();
            collect_functors(&functors.kind, &mut func_set);
            let is_adj = func_set.contains(&Functor::Adj);
            let is_ctl = func_set.contains(&Functor::Ctl);
            let is_ctladj = is_adj && is_ctl;

            let mut spec_decl = match &decl.body {
                CallableBody::Block(body) => vec![SpecDecl {
                    id: self.context.assigner_mut().next_id(),
                    span: body.span,
                    spec: Spec::Body,
                    body: SpecBody::Impl(
                        Pat {
                            id: self.context.assigner_mut().next_id(),
                            span: body.span,
                            kind: PatKind::Elided,
                        },
                        body.clone(),
                    ),
                }],
                CallableBody::Specs(spec_decl) => spec_decl.clone(),
            };

            if is_adj && !spec_decl.iter().any(|s| s.spec == Spec::Adj) {
                spec_decl.push(SpecDecl {
                    id: self.context.assigner_mut().next_id(),
                    span: decl.span,
                    spec: Spec::Adj,
                    body: SpecBody::Gen(SpecGen::Invert),
                });
            }

            if is_ctl && !spec_decl.iter().any(|s| s.spec == Spec::Ctl) {
                spec_decl.push(SpecDecl {
                    id: self.context.assigner_mut().next_id(),
                    span: decl.span,
                    spec: Spec::Ctl,
                    body: SpecBody::Gen(SpecGen::Distribute),
                });
            }

            if is_ctladj && !spec_decl.iter().any(|s| s.spec == Spec::CtlAdj) {
                let gen = if is_self_adjoint(&spec_decl) {
                    SpecGen::Slf
                } else {
                    SpecGen::Distribute
                };
                spec_decl.push(SpecDecl {
                    id: self.context.assigner_mut().next_id(),
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
    if let Some(adj_spec) = spec_decl.iter().find(|s| s.spec == Spec::Adj) {
        matches!(adj_spec.body, SpecBody::Gen(SpecGen::Slf))
    } else {
        false
    }
}

fn generate_spec_impls(unit: &mut CompileUnit) {
    let mut pass = SpecImplPass {
        context: &mut unit.context,
    };
    pass.transform(&mut unit.package);
}

struct SpecImplPass<'a> {
    context: &'a mut Context,
}

impl<'a> SpecImplPass<'a> {
    fn transform(&mut self, package: &mut Package) {
        for ns in &mut package.namespaces {
            self.visit_namespace(ns);
        }
    }

    fn ctl_distrib(&mut self, spec_decl: &mut SpecDecl, block: &Block) {
        // Create the Path that will be used when inserting controls into call args.
        let ctls_path_id = self.context.assigner_mut().next_id();
        let ctls_ident_id = self.context.assigner_mut().next_id();
        self.context
            .tys_mut()
            .insert(ctls_path_id, Ty::Array(Box::new(Ty::Prim(Prim::Qubit))));
        self.context
            .tys_mut()
            .insert(ctls_ident_id, Ty::Array(Box::new(Ty::Prim(Prim::Qubit))));
        let ctls = Path {
            id: ctls_path_id,
            span: spec_decl.span,
            namespace: None,
            name: Ident {
                id: ctls_ident_id,
                span: spec_decl.span,
                name: "ctls".to_string(),
            },
        };

        // Clone the reference block and use the pass to update the calls inside.
        let mut ctl_block = block.clone();
        let mut distrib = CtlDistrib {
            ctls: &ctls,
            context: self.context,
        };
        distrib.visit_block(&mut ctl_block);

        // Add both the Ident for the controls array and the Path to the resolutions.
        self.context
            .resolutions_mut()
            .insert(ctls.name.id, Res::Internal(ctls.name.id));
        self.context
            .resolutions_mut()
            .insert(ctls.id, Res::Internal(ctls.name.id));

        // Update the specialization body to reflect the generated block.
        spec_decl.body = SpecBody::Impl(
            Pat {
                id: self.context.assigner_mut().next_id(),
                span: spec_decl.span,
                kind: PatKind::Tuple(vec![
                    Pat {
                        id: self.context.assigner_mut().next_id(),
                        span: spec_decl.span,
                        kind: PatKind::Bind(ctls.name, None),
                    },
                    Pat {
                        id: self.context.assigner_mut().next_id(),
                        span: spec_decl.span,
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
        if decl.functors.is_some() {
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
                    todo!("controlled spec should be present");
                };
                let SpecBody::Impl(_, body_block) = &body.body else {
                    todo!("body spec should have impl");
                };

                if let Some(ctl) = ctl.as_mut() {
                    if matches!(ctl.body, SpecBody::Gen(SpecGen::Distribute))
                        || matches!(ctl.body, SpecBody::Gen(SpecGen::Auto))
                    {
                        self.ctl_distrib(ctl, body_block);
                    }
                };

                if let (Some(ctladj), Some(adj)) = (ctladj.as_mut(), &adj) {
                    if matches!(ctladj.body, SpecBody::Gen(SpecGen::Distribute))
                        || matches!(ctladj.body, SpecBody::Gen(SpecGen::Auto))
                    {
                        if let SpecBody::Impl(_, adj_block) = &adj.body {
                            self.ctl_distrib(ctladj, adj_block);
                        }
                    }
                };

                *spec_decls = vec![body];
                adj.into_iter().for_each(|spec| spec_decls.push(spec));
                ctl.into_iter().for_each(|spec| spec_decls.push(spec));
                ctladj.into_iter().for_each(|spec| spec_decls.push(spec));
            }
        }
    }
}
