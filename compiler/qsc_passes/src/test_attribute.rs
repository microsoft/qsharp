// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_hir::{hir::Attr, visit::Visitor};
use thiserror::Error;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum TestAttributeError {
    #[error("test callables cannot take arguments")]
    CallableHasParameters(#[label] Span),
    #[error("test callables cannot have type parameters")]
    CallableHasTypeParameters(#[label] Span),
}

pub(crate) fn validate_test_attributes(
    package: &mut qsc_hir::hir::Package,
) -> Vec<TestAttributeError> {
    let mut validator = TestAttributeValidator { errors: Vec::new() };
    validator.visit_package(package);
    validator.errors
}

struct TestAttributeValidator {
    errors: Vec<TestAttributeError>,
}

impl<'a> Visitor<'a> for TestAttributeValidator {
    fn visit_callable_decl(&mut self, decl: &'a qsc_hir::hir::CallableDecl) {
        if decl.attrs.iter().any(|attr| matches!(attr, Attr::Test)) {
            if !decl.generics.is_empty() {
                self.errors
                    .push(TestAttributeError::CallableHasTypeParameters(decl.name.span));
            }
            if decl.input.ty != qsc_hir::ty::Ty::UNIT {
                self.errors
                    .push(TestAttributeError::CallableHasParameters(decl.name.span));
            }
        }
    }
}
