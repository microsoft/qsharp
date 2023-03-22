// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//#[cfg(test)]
//mod tests;

//use miette::Diagnostic;
use qsc_ast::{
    ast::Block,
    mut_visit::{walk_block, MutVisitor},
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

struct ReplaceQubitAllocation {}

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
        walk_block(self, block);
    }
}
