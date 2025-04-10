// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.
use miette::Diagnostic;
use qsc_data_structures::span::Span;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Eq, Error, PartialEq)]
#[error(transparent)]
#[diagnostic(transparent)]
pub struct Error(pub CompilerErrorKind);

/// Represents the kind of semantic error that occurred during compilation of a QASM file(s).
/// For the most part, these errors are fatal and prevent compilation and are
/// safety checks to ensure that the QASM code is valid.
#[derive(Clone, Debug, Diagnostic, Eq, Error, PartialEq)]
pub enum CompilerErrorKind {
    #[error("{0} are not supported.")]
    #[diagnostic(code("Qsc.Qasm3.Compiler.NotSupported"))]
    NotSupported(String, #[label] Span),
    #[error("Qiskit circuits must have output registers.")]
    #[diagnostic(code("Qsc.Qasm3.Compiler.QiskitEntryPointMissingOutput"))]
    QiskitEntryPointMissingOutput(#[label] Span),
    #[error("Annotations only valid on def and gate statements.")]
    #[diagnostic(code("Qsc.Qasm3.Compiler.InvalidAnnotationTarget"))]
    InvalidAnnotationTarget(#[label] Span),
    #[error("Gate expects {0} qubit arguments, but {1} were provided.")]
    #[diagnostic(code("Qsc.Qasm3.Compiler.InvalidNumberOfQubitArgs"))]
    InvalidNumberOfQubitArgs(usize, usize, #[label] Span),
    #[error("Unexpected annotation: {0}.")]
    #[diagnostic(code("Qsc.Qasm3.Compiler.UnknownAnnotation"))]
    UnknownAnnotation(String, #[label] Span),
    #[error("this statement is not yet handled during OpenQASM 3 import: {0}")]
    #[diagnostic(code("Qsc.Qasm3.Compiler.Unimplemented"))]
    Unimplemented(String, #[label] Span),
}

impl From<Error> for crate::Error {
    fn from(val: Error) -> Self {
        crate::Error(crate::ErrorKind::Compiler(val))
    }
}
