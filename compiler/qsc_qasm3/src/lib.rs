// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// while we work through the conversion, allow dead code to avoid warnings
#![allow(dead_code)]

mod angle;
mod ast_builder;
mod compile;
pub use compile::qasm_to_program;
pub mod io;
mod keyword;
mod lex;
mod oqasm_helpers;
mod oqasm_types;
pub mod parse;
pub mod parser;
mod runtime;
pub mod semantic;
mod symbols;
mod types;

#[cfg(test)]
pub(crate) mod tests;

use std::{fmt::Write, sync::Arc};

use miette::Diagnostic;
use qsc_ast::ast::Package;
use qsc_data_structures::span::Span;
use qsc_frontend::{compile::SourceMap, error::WithSource};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Eq, Error, PartialEq)]
#[diagnostic(transparent)]
#[error(transparent)]
pub struct Error(pub ErrorKind);

impl Error {
    #[must_use]
    pub fn with_offset(self, offset: u32) -> Self {
        Self(self.0.with_offset(offset))
    }

    #[must_use]
    pub fn is_syntax_error(&self) -> bool {
        matches!(self.0, ErrorKind::Parse(_, _))
    }

    #[must_use]
    pub fn is_semantic_error(&self) -> bool {
        matches!(self.0, ErrorKind::Semantic(..))
    }
}

/// Represents the kind of error that occurred during compilation of a QASM file(s).
/// The errors fall into a few categories:
/// - Unimplemented features
/// - Not supported features
/// - Parsing errors (converted from the parser)
/// - Semantic errors
/// - IO errors
#[derive(Clone, Debug, Diagnostic, Eq, Error, PartialEq)]
#[error(transparent)]
pub enum ErrorKind {
    #[error("QASM3 Parse Error: {0}")]
    Parse(String, #[label] Span),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Parser(#[from] crate::parser::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Semantic(#[from] crate::semantic::Error),
    #[error("QASM3 Parse Error: Not Found {0}")]
    NotFound(String),
    #[error("IO Error: {0}")]
    IO(String),
}

impl ErrorKind {
    fn with_offset(self, offset: u32) -> Self {
        match self {
            ErrorKind::Parse(error, span) => Self::Parse(error, span + offset),
            ErrorKind::Parser(error) => Self::Parser(error.with_offset(offset)),
            ErrorKind::Semantic(error) => Self::Semantic(error.with_offset(offset)),
            ErrorKind::NotFound(error) => Self::NotFound(error),
            ErrorKind::IO(error) => Self::IO(error),
        }
    }
}

/// Qubit semantics differ between Q# and Qiskit. This enum is used to
/// specify which semantics to use when compiling QASM to Q#.
///
/// Q# requires qubits to be in the 0 state before and after use.
/// Qiskit makes no assumptions about the state of qubits before or after use.
///
/// During compliation, if Qiskit semantics are used, the compiler will insert
/// calls to create qubits instead of `use` bindings. This means that later
/// compiler passes won't generate the Q# code that would check the qubits.
///
/// If Q# semantics are used, the compiler will insert `use` bindings.
///
/// The Qiskit semantics can also be useful if we ever want to do state
/// vector simulation as it will allow us to get the simulator state at
/// the end of the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QubitSemantics {
    QSharp,
    Qiskit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerConfig {
    pub qubit_semantics: QubitSemantics,
    pub output_semantics: OutputSemantics,
    pub program_ty: ProgramType,
    operation_name: Option<Arc<str>>,
    namespace: Option<Arc<str>>,
}

impl CompilerConfig {
    #[must_use]
    pub fn new(
        qubit_semantics: QubitSemantics,
        output_semantics: OutputSemantics,
        program_ty: ProgramType,
        operation_name: Option<Arc<str>>,
        namespace: Option<Arc<str>>,
    ) -> Self {
        Self {
            qubit_semantics,
            output_semantics,
            program_ty,
            operation_name,
            namespace,
        }
    }

    fn operation_name(&self) -> Arc<str> {
        self.operation_name
            .clone()
            .unwrap_or_else(|| Arc::from("program"))
    }

    fn namespace(&self) -> Arc<str> {
        self.namespace
            .clone()
            .unwrap_or_else(|| Arc::from("qasm3_import"))
    }
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            qubit_semantics: QubitSemantics::Qiskit,
            output_semantics: OutputSemantics::Qiskit,
            program_ty: ProgramType::Fragments,
            operation_name: None,
            namespace: None,
        }
    }
}

/// Represents the type of compilation output to create
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgramType {
    /// Creates an operation in a namespace as if the program is a standalone
    /// file. Inputs are lifted to the operation params. Output are lifted to
    /// the operation return type. The operation is marked as `@EntryPoint`
    /// as long as there are no input parameters.
    File,
    /// Programs are compiled to a standalone function. Inputs are lifted to
    /// the operation params. Output are lifted to the operation return type.
    Operation,
    /// Creates a list of statements from the program. This is useful for
    /// interactive environments where the program is a list of statements
    /// imported into the current scope.
    /// This is also useful for testing individual statements compilation.
    Fragments,
}

/// Represents the signature of an operation.
/// This is used to create a function signature for the
/// operation that is created from the QASM source code.
/// This is the human readable form of the operation.
pub struct OperationSignature {
    pub name: String,
    pub ns: Option<String>,
    pub input: Vec<(String, String)>,
    pub output: String,
}

impl OperationSignature {
    /// Creates a human readable operation signature of the form:
    /// `ns.name(input)`
    /// which is required to call the operation from other code.
    #[must_use]
    pub fn create_entry_expr_from_params<S: AsRef<str>>(&self, params: S) -> String {
        let mut expr = String::new();
        if let Some(ns) = &self.ns {
            write!(expr, "{ns}.").unwrap();
        }
        write!(expr, "{}(", self.name).unwrap();
        write!(expr, "{}", params.as_ref()).unwrap();
        write!(expr, ")").unwrap();

        expr
    }

    /// Renders the input parameters as a string of comma separated
    /// <name: type> pairs.
    #[must_use]
    pub fn input_params(&self) -> String {
        let mut expr = String::new();
        for (i, (name, ty)) in self.input.iter().enumerate() {
            if i > 0 {
                write!(expr, ", ").unwrap();
            }
            write!(expr, "{name}: {ty}").unwrap();
        }
        expr
    }
}

impl std::fmt::Display for OperationSignature {
    /// Renders the operation signature as a human readable string.
    /// The signature is of the form:
    /// `ns.name(input) -> output`
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ns) = &self.ns {
            write!(f, "{ns}.")?;
        }
        write!(f, "{}(", self.name)?;
        for (i, (name, ty)) in self.input.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{name}: {ty}")?;
        }
        write!(f, ") -> {}", self.output)
    }
}

/// A unit of compilation for QASM source code.
/// This is the result of parsing and compiling a QASM source file.
pub struct QasmCompileUnit {
    /// Source map created from the accumulated source files,
    source_map: SourceMap,
    /// Semantic errors encountered during compilation.
    /// These are always fatal errors that prevent compilation.
    errors: Vec<WithSource<crate::Error>>,
    /// The compiled AST package, if compilation was successful.
    /// There is no guarantee that this package is valid unless
    /// there are no errors.
    package: Option<Package>,
    /// The signature of the operation created from the QASM source code.
    /// None if the program type is `ProgramType::Fragments`.
    signature: Option<OperationSignature>,
}

/// Represents a QASM compilation unit.
/// This is the result of parsing and compiling a QASM source file.
/// The result contains the source map, errors, and the compiled package.
/// The package is only valid if there are no errors.
impl QasmCompileUnit {
    #[must_use]
    pub fn new(
        source_map: SourceMap,
        errors: Vec<WithSource<crate::Error>>,
        package: Option<Package>,
        signature: Option<OperationSignature>,
    ) -> Self {
        Self {
            source_map,
            errors,
            package,
            signature,
        }
    }

    /// Returns true if there are errors in the compilation unit.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns a list of errors in the compilation unit.
    #[must_use]
    pub fn errors(&self) -> Vec<WithSource<crate::Error>> {
        self.errors.clone()
    }

    /// Deconstructs the compilation unit into its owned parts.
    #[must_use]
    pub fn into_tuple(
        self,
    ) -> (
        SourceMap,
        Vec<WithSource<crate::Error>>,
        Option<Package>,
        Option<OperationSignature>,
    ) {
        (self.source_map, self.errors, self.package, self.signature)
    }
}

/// Represents the output semantics of the compilation.
/// Each has implications on the output of the compilation
/// and the semantic checks that are performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputSemantics {
    /// The output is in Qiskit format meaning that the output
    /// is all of the classical registers, in reverse order
    /// in which they were added to the circuit with each
    /// bit within each register in reverse order.
    Qiskit,
    /// [OpenQASM 3 has two output modes](https://openqasm.com/language/directives.html#input-output)
    /// - If the programmer provides one or more `output` declarations, then
    ///   variables described as outputs will be returned as output.
    ///   The spec make no mention of endianness or order of the output.
    /// - Otherwise, assume all of the declared variables are returned as output.
    OpenQasm,
    /// No output semantics are applied. The entry point returns `Unit`.
    ResourceEstimation,
}
