// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

mod callable_limits;
mod common;
pub mod conjugate_invert;
pub mod entry_point;
mod invert_block;
mod logic_sep;
pub mod loop_unification;
pub mod replace_qubit_allocation;
pub mod spec_gen;

use callable_limits::CallableLimits;
use loop_unification::LoopUni;
use miette::Diagnostic;
use qsc_frontend::{compile::CompileUnit, incremental::Fragment};
use qsc_hir::{
    assigner::Assigner,
    global::{self, Table},
    hir::{Item, ItemKind},
    mut_visit::MutVisitor,
    visit::Visitor,
};
use replace_qubit_allocation::ReplaceQubitAllocation;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub enum Error {
    CallableLimits(callable_limits::Error),
    ConjInvert(conjugate_invert::Error),
    EntryPoint(entry_point::Error),
    SpecGen(spec_gen::Error),
}

/// Run the default set of passes required for evaluation.
pub fn run_default_passes(core: &Table, unit: &mut CompileUnit) -> Vec<Error> {
    let mut call_limits = CallableLimits::default();
    call_limits.visit_package(&unit.package);
    let callable_errors = call_limits.errors;
    let spec_errors = spec_gen::generate_specs(core, unit);
    let conjugate_errors = conjugate_invert::invert_conjugate_exprs(core, unit);

    LoopUni {
        core,
        assigner: &mut unit.assigner,
    }
    .visit_package(&mut unit.package);
    ReplaceQubitAllocation::new(core, &mut unit.assigner).visit_package(&mut unit.package);

    callable_errors
        .into_iter()
        .map(Error::CallableLimits)
        .chain(spec_errors.into_iter().map(Error::SpecGen))
        .chain(conjugate_errors.into_iter().map(Error::ConjInvert))
        .collect()
}

pub fn run_core_passes(core: &mut CompileUnit) {
    let table = global::iter_package(None, &core.package).collect();
    LoopUni {
        core: &table,
        assigner: &mut core.assigner,
    }
    .visit_package(&mut core.package);
    ReplaceQubitAllocation::new(&table, &mut core.assigner).visit_package(&mut core.package);
}

pub fn run_default_passes_for_fragment(
    core: &Table,
    assigner: &mut Assigner,
    fragment: &mut Fragment,
) -> Vec<Error> {
    let mut errors = Vec::new();

    match fragment {
        Fragment::Stmt(stmt) => {
            errors.extend(
                conjugate_invert::invert_conjugate_exprs_for_stmt(core, assigner, stmt)
                    .into_iter()
                    .map(Error::ConjInvert),
            );
            LoopUni { core, assigner }.visit_stmt(stmt);
            ReplaceQubitAllocation::new(core, assigner).visit_stmt(stmt);
        }
        Fragment::Item(Item {
            kind: ItemKind::Callable(decl),
            ..
        }) => {
            let mut call_limits = CallableLimits::default();
            call_limits.visit_callable_decl(decl);
            errors.extend(call_limits.errors.into_iter().map(Error::CallableLimits));
            errors.extend(
                spec_gen::generate_specs_for_callable(core, assigner, decl)
                    .into_iter()
                    .map(Error::SpecGen),
            );
            errors.extend(
                conjugate_invert::invert_conjugate_exprs_for_callable(core, assigner, decl)
                    .into_iter()
                    .map(Error::ConjInvert),
            );
            LoopUni { core, assigner }.visit_callable_decl(decl);
            ReplaceQubitAllocation::new(core, assigner).visit_callable_decl(decl);
        }
        Fragment::Item(_) | Fragment::Error(_) => {}
    }

    errors
}
