// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

pub mod conjugate_invert;
pub mod entry_point;
mod invert_block;
mod logic_sep;
pub mod spec_gen;

use miette::Diagnostic;
use qsc_frontend::compile::CompileUnit;
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
    let mut errors = Vec::new();

    errors.extend(
        spec_gen::generate_specs(unit)
            .into_iter()
            .map(Error::SpecGen),
    );

    errors.extend(
        conjugate_invert::invert_conjugate_exprs(unit)
            .into_iter()
            .map(Error::ConjInvert),
    );

    errors
}
