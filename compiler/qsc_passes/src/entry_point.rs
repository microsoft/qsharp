// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use super::Error as PassErr;
use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_hir::{
    hir::{CallableBody, CallableDecl, Expr, ExprKind, Item, ItemKind, NodeId, Package, PatKind},
    visit::Visitor,
};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("duplicate entry point callable `{0}`")]
    #[diagnostic(help("only one callable should be annotated with the entry point attribute"))]
    DuplicateEntryPoint(String, #[label] Span),

    #[error("entry point cannot have paramters")]
    EntryPointArgs(#[label] Span),

    #[error("entry point must have body implementation only")]
    EntryPointBody(#[label("cannot have specialization implementation")] Span),

    #[error("entry point not found")]
    #[diagnostic(help("a single callable with the `@EntryPoint()` attribute must be present if no entry expression is provided"))]
    EntryPointMissing,
}

/// Extracts a single entry point callable declaration, if found.
/// # Errors
/// Returns an error if a single entry point with no parameters cannot be found.
pub fn extract_entry(package: &Package) -> Result<Expr, Vec<super::Error>> {
    let mut entry_points = vec![];
    let mut visitor = EntryPointVisitor {
        entry_points: &mut entry_points,
    };
    visitor.visit_package(package);
    if entry_points.len() == 1 {
        let ep = entry_points[0];
        let arg_count = if let PatKind::Tuple(args) = &ep.input.kind {
            args.len()
        } else {
            1
        };
        if arg_count == 0 {
            if let CallableBody::Block(block) = &ep.body {
                Ok(Expr {
                    id: NodeId::default(),
                    span: Span::default(),
                    ty: ep.output.clone(),
                    kind: ExprKind::Block(block.clone()),
                })
            } else {
                Err(vec![PassErr::EntryPoint(Error::EntryPointBody(ep.span))])
            }
        } else {
            Err(vec![PassErr::EntryPoint(Error::EntryPointArgs(
                ep.input.span,
            ))])
        }
    } else if entry_points.is_empty() {
        Err(vec![PassErr::EntryPoint(Error::EntryPointMissing)])
    } else {
        Err(entry_points
            .into_iter()
            .map(|ep| {
                PassErr::EntryPoint(Error::DuplicateEntryPoint(
                    ep.name.name.to_string(),
                    ep.name.span,
                ))
            })
            .collect())
    }
}

struct EntryPointVisitor<'a, 'b> {
    entry_points: &'a mut Vec<&'b CallableDecl>,
}

impl<'a, 'b> Visitor<'b> for EntryPointVisitor<'a, 'b> {
    fn visit_item(&mut self, item: &'b Item) {
        if let ItemKind::Callable(decl) = &item.kind {
            if item
                .attrs
                .iter()
                .any(|attr| attr.name.name.as_ref() == "EntryPoint")
            {
                self.entry_points.push(decl);
            }
        }
    }
}
