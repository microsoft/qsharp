// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

mod common;
pub mod conjugate_invert;
pub mod entry_point;
mod invert_block;
mod logic_sep;
pub mod loop_unification;
pub mod replace_qubit_allocation;
pub mod spec_gen;

use miette::Diagnostic;
use qsc_frontend::{compile::CompileUnit, incremental::Fragment};
use qsc_hir::{
    assigner::Assigner,
    hir::{Item, ItemKind},
};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub enum Error {
    EntryPoint(entry_point::Error),
    SpecGen(spec_gen::Error),
    ConjInvert(conjugate_invert::Error),
}

/// Run the default set of passes required for evaluation.
pub fn run_default_passes(unit: &mut CompileUnit) -> Vec<Error> {
    let spec_errors = spec_gen::generate_specs(unit);
    let conjugate_errors = conjugate_invert::invert_conjugate_exprs(unit);

    spec_errors
        .into_iter()
        .map(Error::SpecGen)
        .chain(conjugate_errors.into_iter().map(Error::ConjInvert))
        .collect()
}

pub fn run_default_passes_for_fragment(
    assigner: &mut Assigner,
    fragment: &mut Fragment,
) -> Vec<Error> {
    let mut errors = Vec::new();

    match fragment {
        Fragment::Stmt(stmt) => {
            errors.extend(
                conjugate_invert::invert_conjugate_exprs_for_stmt(assigner, stmt)
                    .into_iter()
                    .map(Error::ConjInvert),
            );
        }
        Fragment::Item(Item {
            kind: ItemKind::Callable(decl),
            ..
        }) => {
            errors.extend(
                spec_gen::generate_specs_for_callable(assigner, decl)
                    .into_iter()
                    .map(Error::SpecGen),
            );
            errors.extend(
                conjugate_invert::invert_conjugate_exprs_for_callable(assigner, decl)
                    .into_iter()
                    .map(Error::ConjInvert),
            );
        }
        Fragment::Item(_) | Fragment::Error(_) => {}
    }

    errors
}
