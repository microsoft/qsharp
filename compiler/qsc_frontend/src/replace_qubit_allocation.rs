// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//#[cfg(test)]
//mod tests;

use std::mem::{swap, take};

//use miette::Diagnostic;
use qsc_ast::{
    ast::{Block, Ident, Pat, PatKind, QubitInit, QubitInitKind, Stmt, StmtKind},
    mut_visit::{walk_block, walk_stmt, MutVisitor},
};
//use thiserror::Error;

// #[derive(Clone, Debug, Diagnostic, Error)]
// pub(super) enum Error {
//     #[error("adjointable/controllable operation `{0}` must return Unit")]
//     NonUnitReturn(String, #[label("must return Unit")] Span),

//     #[error("callable parameter `{0}` must be type annotated")]
//     ParameterNotTyped(String, #[label("missing type annotation")] Span),

//     #[error("{0} are not currently supported")]
//     NotCurrentlySupported(&'static str, #[label("not currently supported")] Span),
// }

// pub(super) fn validate(package: &Package) -> Vec<Error> {
//     let mut validator = Validator {
//         validation_errors: Vec::new(),
//     };
//     validator.visit_package(package);
//     validator.validation_errors
// }

pub struct ReplaceQubitAllocation {
    //qubits_per_blocks_stack: Vec<Option<Vec<Ident>>>,
    qubits_curr_block: Vec<Ident>,
    prefix_qubits: Vec<Ident>,
}

impl ReplaceQubitAllocation {
    pub fn new() -> ReplaceQubitAllocation {
        ReplaceQubitAllocation {
            //qubits_per_blocks_stack: Vec::new(),
            qubits_curr_block: Vec::new(),
            prefix_qubits: Vec::new(),
        }
    }
}

// impl ReplaceQubitAllocation {
//     fn validate_params(&mut self, params: &Pat) {
//         match &params.kind {
//             qsc_ast::ast::PatKind::Bind(id, ty) => match &ty {
//                 None => self
//                     .validation_errors
//                     .push(Error::ParameterNotTyped(id.name.clone(), params.span)),
//                 Some(t) => self.validate_type(t, params.span),
//             },
//             qsc_ast::ast::PatKind::Paren(item) => self.validate_params(item),
//             qsc_ast::ast::PatKind::Tuple(items) => {
//                 items.iter().for_each(|i| self.validate_params(i));
//             }
//             _ => {}
//         }
//     }

//     fn validate_type(&mut self, ty: &Ty, span: Span) {
//         match &ty.kind {
//             TyKind::App(ty, tys) => {
//                 self.validate_type(ty, span);
//                 tys.iter().for_each(|t| self.validate_type(t, span));
//             }
//             TyKind::Arrow(_, _, _, _) => self.validation_errors.push(Error::NotCurrentlySupported(
//                 "callables as parameters",
//                 span,
//             )),
//             TyKind::Paren(ty) => self.validate_type(ty, span),
//             TyKind::Tuple(tys) => tys.iter().for_each(|t| self.validate_type(t, span)),
//             _ => {}
//         }
//     }
// }

impl MutVisitor for ReplaceQubitAllocation {
    fn visit_block(&mut self, block: &mut Block) {
        let qubits_super_block = take(&mut self.qubits_curr_block);

        while let Some(qubit) = self.prefix_qubits.pop() {
            self.qubits_curr_block.push(qubit);
            //todo!("add call to `__quantum__rt__qubit_allocate()`");
        }

        walk_block(self, block);

        for qubit in &self.qubits_curr_block {
            ();
            //todo!("add call to `__quantum__rt__qubit__release()`");
        }
        self.qubits_curr_block = qubits_super_block;
        ();
    }

    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        if let StmtKind::Qubit(_, pat, init, block) = &mut stmt.kind {
            let qs_w_ty = assign_qubit_type(pat, init);

            // assume single qubit ids
            match block {
                Some(_) => {
                    for q in &qs_w_ty {
                        self.prefix_qubits.push(q.0.clone());
                    }
                }
                None => {
                    for q in &qs_w_ty {
                        self.qubits_curr_block.push(q.0.clone());
                        //todo!("add call to `__quantum__rt__qubit_allocate()`");
                    }
                }
            }
        }

        walk_stmt(self, stmt);
    }
}

// This function is a temporary workaround until we have full type information on identifiers
fn assign_qubit_type<'a>(pat: &'a Pat, init: &'a QubitInit) -> Vec<(&'a Ident, &'a QubitInit)> {
    let init_no_parens = remove_extra_init_parens(init);

    match &pat.kind {
        PatKind::Bind(name, _) => vec![(name, init_no_parens)],
        PatKind::Discard(_) => vec![],
        PatKind::Elided => todo!("error state for `use` statements"),
        PatKind::Paren(pat) => assign_qubit_type(pat, init_no_parens),
        PatKind::Tuple(tup) => {
            if let QubitInitKind::Tuple(init_tup) = &init_no_parens.kind {
                assert!(
                    tup.len() == init_tup.len(),
                    "qubit tuple initializer length doesn't match identifier tuple length"
                );
                tup.iter()
                    .zip(init_tup.iter())
                    .flat_map(|(pat, init)| assign_qubit_type(pat, init))
                    .collect()
            } else {
                panic!("cannot initialize an identifier tuple with non-tuple qubit initializer");
            }
        }
    }
}

fn remove_extra_init_parens(init: &QubitInit) -> &QubitInit {
    if let QubitInitKind::Paren(p) = &init.kind {
        return remove_extra_init_parens(p);
    }
    init
}
