// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use super::Error as PassErr;
use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_frontend::compile::CompileUnit;
use qsc_hir::{
    assigner::Assigner,
    hir::{
        Attr, CallableDecl, Expr, ExprKind, Item, ItemId, ItemKind, LocalItemId, Package, PatKind,
        Res,
    },
    ty::Ty,
    visit::Visitor,
};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("duplicate entry point callable `{0}`")]
    #[diagnostic(help("only one callable should be annotated with the entry point attribute"))]
    #[diagnostic(code("Qsc.EntryPoint.Duplicate"))]
    Duplicate(String, #[label] Span),

    #[error("entry point cannot have parameters")]
    #[diagnostic(code("Qsc.EntryPoint.Args"))]
    Args(#[label] Span),

    #[error("entry point must have body implementation only")]
    #[diagnostic(code("Qsc.EntryPoint.BodyMissing"))]
    BodyMissing(#[label("cannot have specialization implementation")] Span),

    #[error("entry point not found")]
    #[diagnostic(help("a single callable with the `@EntryPoint()` attribute must be present if no entry expression is provided"))]
    #[diagnostic(code("Qsc.EntryPoint.NotFound"))]
    NotFound,
}

// If no entry expression is provided, generate one from the entry point callable.
// Only one callable should be annotated with the entry point attribute.
pub(super) fn generate_entry_expr(unit: &mut CompileUnit) -> Vec<super::Error> {
    if unit.package.entry.is_some() {
        return vec![];
    }
    let callables = get_callables(&unit.package);

    match create_entry_from_callables(&mut unit.assigner, callables) {
        Ok(expr) => {
            unit.package.entry = Some(expr);
            vec![]
        }
        Err(errs) => errs,
    }
}

fn create_entry_from_callables(
    assigner: &mut Assigner,
    callables: Vec<(&CallableDecl, LocalItemId)>,
) -> Result<Expr, Vec<super::Error>> {
    if callables.len() == 1 {
        let ep = callables[0].0;
        let arg_count = if let PatKind::Tuple(args) = &ep.input.kind {
            args.len()
        } else {
            1
        };
        if arg_count == 0 {
            if ep.adj.is_some() || ep.ctl.is_some() || ep.ctl_adj.is_some() {
                Err(vec![PassErr::EntryPoint(Error::BodyMissing(ep.span))])
            } else {
                match &ep.body.body {
                    qsc_hir::hir::SpecBody::Gen(_) => {
                        Err(vec![PassErr::EntryPoint(Error::BodyMissing(ep.span))])
                    }
                    qsc_hir::hir::SpecBody::Impl(_, block) => {
                        let arg = Expr {
                            id: assigner.next_node(),
                            span: ep.span,
                            ty: Ty::UNIT,
                            kind: ExprKind::Tuple(Vec::new()),
                        };
                        let item = callables[0].1;
                        let item_id = ItemId {
                            package: None,
                            item,
                        };
                        let callee = Expr {
                            id: assigner.next_node(),
                            span: ep.span,
                            ty: block.ty.clone(),
                            kind: ExprKind::Var(Res::Item(item_id), Vec::new()),
                        };
                        let call = Expr {
                            id: assigner.next_node(),
                            span: ep.span,
                            ty: block.ty.clone(),
                            kind: ExprKind::Call(Box::new(callee), Box::new(arg)),
                        };
                        Ok(call)
                    }
                }
            }
        } else {
            Err(vec![PassErr::EntryPoint(Error::Args(ep.input.span))])
        }
    } else if callables.is_empty() {
        Err(vec![PassErr::EntryPoint(Error::NotFound)])
    } else {
        Err(callables
            .into_iter()
            .map(|ep| {
                PassErr::EntryPoint(Error::Duplicate(ep.0.name.name.to_string(), ep.0.name.span))
            })
            .collect())
    }
}

fn get_callables(package: &Package) -> Vec<(&CallableDecl, LocalItemId)> {
    let mut finder = EntryPointFinder {
        callables: Vec::new(),
    };
    finder.visit_package(package);
    finder.callables
}

struct EntryPointFinder<'a> {
    callables: Vec<(&'a CallableDecl, LocalItemId)>,
}

impl<'a> Visitor<'a> for EntryPointFinder<'a> {
    fn visit_item(&mut self, item: &'a Item) {
        if let ItemKind::Callable(callable) = &item.kind {
            if item.attrs.iter().any(|a| a == &Attr::EntryPoint) {
                self.callables.push((callable, item.id));
            }
        }
    }
}
