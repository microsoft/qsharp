// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

mod borrowck;
mod callable_limits;
mod common;
mod conjugate_invert;
pub mod entry_point;
mod id_update;
mod invert_block;
mod logic_sep;
mod loop_unification;
mod replace_qubit_allocation;
mod spec_gen;

use callable_limits::CallableLimits;
use entry_point::generate_entry_expr;
use loop_unification::LoopUni;
use miette::Diagnostic;
use qsc_frontend::{compile::CompileUnit, incremental::Fragment};
use qsc_hir::{
    assigner::Assigner,
    global::{self, Table},
    hir::{Item, ItemKind},
    mut_visit::MutVisitor,
    validate::Validator,
    visit::Visitor,
};
use replace_qubit_allocation::ReplaceQubitAllocation;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub enum Error {
    BorrowCk(borrowck::Error),
    CallableLimits(callable_limits::Error),
    ConjInvert(conjugate_invert::Error),
    EntryPoint(entry_point::Error),
    SpecGen(spec_gen::Error),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PackageType {
    Exe,
    Lib,
}

#[derive(Default)]
pub struct PassContext {
    borrow_check: borrowck::Checker,
}

/// Run the default set of passes required for evaluation.
pub fn run_default_passes(
    core: &Table,
    unit: &mut CompileUnit,
    package_type: PackageType,
) -> Vec<Error> {
    let mut call_limits = CallableLimits::default();
    call_limits.visit_package(&unit.package);
    let callable_errors = call_limits.errors;

    let mut borrow_check = borrowck::Checker::default();
    borrow_check.visit_package(&unit.package);
    let borrow_errors = borrow_check.errors;

    let spec_errors = spec_gen::generate_specs(core, unit);
    Validator::default().visit_package(&unit.package);

    let conjugate_errors = conjugate_invert::invert_conjugate_exprs(core, unit);
    Validator::default().visit_package(&unit.package);

    let entry_point_errors = if package_type == PackageType::Exe {
        let entry_point_errors = generate_entry_expr(unit);
        Validator::default().visit_package(&unit.package);
        entry_point_errors
    } else {
        Vec::new()
    };

    LoopUni {
        core,
        assigner: &mut unit.assigner,
    }
    .visit_package(&mut unit.package);
    Validator::default().visit_package(&unit.package);

    ReplaceQubitAllocation::new(core, &mut unit.assigner).visit_package(&mut unit.package);
    Validator::default().visit_package(&unit.package);

    callable_errors
        .into_iter()
        .map(Error::CallableLimits)
        .chain(borrow_errors.into_iter().map(Error::BorrowCk))
        .chain(spec_errors.into_iter().map(Error::SpecGen))
        .chain(conjugate_errors.into_iter().map(Error::ConjInvert))
        .chain(entry_point_errors.into_iter())
        .collect()
}

pub fn run_core_passes(core: &mut CompileUnit) -> Vec<Error> {
    let mut borrow_check = borrowck::Checker::default();
    borrow_check.visit_package(&core.package);
    let borrow_errors = borrow_check.errors;

    let table = global::iter_package(None, &core.package).collect();
    LoopUni {
        core: &table,
        assigner: &mut core.assigner,
    }
    .visit_package(&mut core.package);
    Validator::default().visit_package(&core.package);

    ReplaceQubitAllocation::new(&table, &mut core.assigner).visit_package(&mut core.package);
    Validator::default().visit_package(&core.package);

    borrow_errors.into_iter().map(Error::BorrowCk).collect()
}

impl PassContext {
    pub fn run(
        &mut self,
        core: &Table,
        assigner: &mut Assigner,
        fragment: &mut Fragment,
    ) -> Vec<Error> {
        let mut errors = Vec::new();

        match fragment {
            Fragment::Stmt(stmt) => {
                self.borrow_check.visit_stmt(stmt);
                errors.extend(self.borrow_check.errors.drain(..).map(Error::BorrowCk));

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

                let mut borrow_check = borrowck::Checker::default();
                borrow_check.visit_callable_decl(decl);
                errors.extend(borrow_check.errors.into_iter().map(Error::BorrowCk));

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
            Fragment::Item(_) => {}
        }

        errors
    }
}
