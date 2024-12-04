// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod angle;
mod ast_builder;
mod compile;
pub use compile::qasm_to_program;
pub mod io;
mod oqasm_helpers;
mod oqasm_types;
pub mod parse;
mod runtime;
mod symbols;
mod types;

#[cfg(test)]
pub(crate) mod tests;

use std::{fmt::Write, sync::Arc};

use miette::Diagnostic;
use qsc::Span;
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
pub enum ErrorKind {
    #[error("this statement is not yet handled during OpenQASM 3 import: {0}")]
    Unimplemented(String, #[label] Span),
    #[error("calibration statements are not supported: {0}")]
    CalibrationsNotSupported(String, #[label] Span),
    #[error("{0} are not supported.")]
    NotSupported(String, #[label] Span),
    #[error("QASM3 Parse Error: {0}")]
    Parse(String, #[label] Span),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Semantic(#[from] crate::SemanticError),
    #[error("QASM3 Parse Error: Not Found {0}")]
    NotFound(String),
    #[error("IO Error: {0}")]
    IO(String),
}

impl ErrorKind {
    fn with_offset(self, offset: u32) -> Self {
        match self {
            ErrorKind::Unimplemented(error, span) => Self::Unimplemented(error, span + offset),
            ErrorKind::CalibrationsNotSupported(error, span) => {
                Self::CalibrationsNotSupported(error, span + offset)
            }
            ErrorKind::NotSupported(error, span) => Self::NotSupported(error, span + offset),
            ErrorKind::Parse(error, span) => Self::Parse(error, span + offset),
            ErrorKind::Semantic(error) => ErrorKind::Semantic(error.with_offset(offset)),
            ErrorKind::NotFound(error) => Self::NotFound(error),
            ErrorKind::IO(error) => Self::IO(error),
        }
    }
}

#[derive(Clone, Debug, Diagnostic, Eq, Error, PartialEq)]
#[error(transparent)]
#[diagnostic(transparent)]
pub struct SemanticError(SemanticErrorKind);

impl SemanticError {
    #[must_use]
    pub fn with_offset(self, offset: u32) -> Self {
        Self(self.0.with_offset(offset))
    }
}

/// Represents the kind of semantic error that occurred during compilation of a QASM file(s).
/// For the most part, these errors are fatal and prevent compilation and are
/// safety checks to ensure that the QASM code is valid.
///
/// We can't use the semantics library for this:
///   - it is unsafe to use (heavy use of panic and unwrap)
///   - it is missing many language features
#[derive(Clone, Debug, Diagnostic, Eq, Error, PartialEq)]
enum SemanticErrorKind {
    #[error("Annotation missing target statement.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.AnnotationWithoutStatement"))]
    AnnotationWithoutStatement(#[label] Span),
    #[error("Cannot alias type {0}. Only qubit and qubit[] can be aliased.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.CannotAliasType"))]
    CannotAliasType(String, Span),
    #[error("Cannot apply operator {0} to types {1} and {2}.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.CannotApplyOperatorToTypes"))]
    CannotApplyOperatorToTypes(String, String, String, #[label] Span),
    #[error("Cannot assign a value of {0} type to a classical variable of {1} type.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.CannotAssignToType"))]
    CannotAssignToType(String, String, #[label] Span),
    #[error("Cannot call a gate that is not a gate.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.CannotCallNonGate"))]
    CannotCallNonGate(#[label] Span),
    #[error("Cannot cast expression of type {0} to type {1}")]
    #[diagnostic(code("Qsc.Qasm3.Compile.CannotCast"))]
    CannotCast(String, String, #[label] Span),
    #[error("Cannot index variables of type {0}")]
    #[diagnostic(code("Qsc.Qasm3.Compile.CannotIndexType"))]
    CannotIndexType(String, #[label] Span),
    #[error("Cannot update const variable {0}")]
    #[diagnostic(help("mutable variables must be declared without the keyword `const`."))]
    #[diagnostic(code("Qsc.Qasm3.Compile.CannotUpdateConstVariable"))]
    CannotUpdateConstVariable(String, #[label] Span),
    #[error("Cannot cast expression of type {0} to type {1} as it would cause truncation.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.CastWouldCauseTruncation"))]
    CastWouldCauseTruncation(String, String, #[label] Span),
    #[error("Complex numbers in assignment binary expressions are not yet supported.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.ComplexBinaryAssignment"))]
    ComplexBinaryAssignment(#[label] Span),
    #[error("Designator must be a literal integer.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.DesignatorMustBeIntLiteral"))]
    DesignatorMustBeIntLiteral(#[label] Span),
    #[error("Failed to compile all expressions in expression list.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.FailedToCompileExpressionList"))]
    FailedToCompileExpressionList(#[label] Span),
    #[error("For iterable must have a set expression, range expression, or iterable expression.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.ForIterableInvalidExpression"))]
    ForIterableInvalidExpression(#[label] Span),
    #[error("For statements must have a body or statement.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.ForStatementsMustHaveABodyOrStatement"))]
    ForStatementsMustHaveABodyOrStatement(#[label] Span),
    #[error("If statement missing {0} expression.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.IfStmtMissingExpression"))]
    IfStmtMissingExpression(String, #[label] Span),
    #[error("include {0} could not be found.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.IncludeNotFound"))]
    IncludeNotFound(String, #[label] Span),
    #[error("include {0} must be declared in global scope.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.IncludeNotInGlobalScope"))]
    IncludeNotInGlobalScope(String, #[label] Span),
    #[error("include {0} must be declared in global scope.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.IncludeStatementMissingPath"))]
    IncludeStatementMissingPath(#[label] Span),
    #[error("Indexed must be a single expression.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.IndexMustBeSingleExpr"))]
    IndexMustBeSingleExpr(#[label] Span),
    #[error("Annotations only valid on gate definitions.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.InvalidAnnotationTarget"))]
    InvalidAnnotationTarget(Span),
    #[error("Assigning {0} values to {1} must be in a range that be converted to {1}.")]
    InvalidCastValueRange(String, String, #[label] Span),
    #[error("Gate operands other than qubits or qubit arrays are not supported.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.InvalidGateOperand"))]
    InvalidGateOperand(#[label] Span),
    #[error("Control counts must be integer literals.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.InvalidControlCount"))]
    InvalidControlCount(#[label] Span),
    #[error("Gate operands other than qubit arrays are not supported.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.InvalidIndexedGateOperand"))]
    InvalidIndexedGateOperand(#[label] Span),
    #[error("Gate expects {0} classical arguments, but {1} were provided.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.InvalidNumberOfClassicalArgs"))]
    InvalidNumberOfClassicalArgs(usize, usize, #[label] Span),
    #[error("Gate expects {0} qubit arguments, but {1} were provided.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.InvalidNumberOfQubitArgs"))]
    InvalidNumberOfQubitArgs(usize, usize, #[label] Span),
    #[error("Measure statements must have a name.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.MeasureExpressionsMustHaveName"))]
    MeasureExpressionsMustHaveName(#[label] Span),
    #[error("Measure statements must have a gate operand name.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.MeasureExpressionsMustHaveGateOperand"))]
    MeasureExpressionsMustHaveGateOperand(#[label] Span),
    #[error("Control counts must be postitive integers.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.NegativeControlCount"))]
    NegativeControlCount(#[label] Span),
    #[error("The operator {0} is not valid with lhs {1} and rhs {2}.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.OperatorNotSupportedForTypes"))]
    OperatorNotSupportedForTypes(String, String, String, #[label] Span),
    #[error("Pow gate modifiers must have an exponent.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.PowModifierMustHaveExponent"))]
    PowModifierMustHaveExponent(#[label] Span),
    #[error("Qiskit circuits must have output registers.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.QiskitEntryPointMissingOutput"))]
    QiskitEntryPointMissingOutput(#[label] Span),
    #[error("Quantum declarations must be done in global scope.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.QuantumDeclarationInNonGlobalScope"))]
    QuantumDeclarationInNonGlobalScope(#[label] Span),
    #[error("Quantum typed values cannot be used in binary expressions.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.QuantumTypesInBinaryExpression"))]
    QuantumTypesInBinaryExpression(#[label] Span),
    #[error("Range expressions must have a start.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.RangeExpressionsMustHaveStart"))]
    RangeExpressionsMustHaveStart(#[label] Span),
    #[error("Range expressions must have a stop.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.RangeExpressionsMustHaveStop"))]
    RangeExpressionsMustHaveStop(#[label] Span),
    #[error("Redefined symbol: {0}.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.RedefinedSymbol"))]
    RedefinedSymbol(String, #[label] Span),
    #[error("Reset expression must have a gate operand.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.ResetExpressionMustHaveGateOperand"))]
    ResetExpressionMustHaveGateOperand(#[label] Span),
    #[error("Reset expression must have a name.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.ResetExpressionMustHaveName"))]
    ResetExpressionMustHaveName(#[label] Span),
    #[error("Return statements are only allowed within subroutines.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.ReturnNotInSubroutine"))]
    ReturnNotInSubroutine(#[label] Span),
    #[error("Too many controls specified.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.TooManyControls"))]
    TooManyControls(#[label] Span),
    #[error("Types differ by dimensions and are incompatible.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.TypeRankError"))]
    TypeRankError(#[label] Span),
    #[error("Undefined symbol: {0}.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.UndefinedSymbol"))]
    UndefinedSymbol(String, #[label] Span),
    #[error("Unexpected parser error: {0}.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.UnexpectedParserError"))]
    UnexpectedParserError(String, #[label] Span),
    #[error("Unexpected annotation: {0}.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.UnknownAnnotation"))]
    UnknownAnnotation(String, #[label] Span),
    #[error("Undefined symbol: {0}.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.UnknownIndexedOperatorKind"))]
    UnknownIndexedOperatorKind(#[label] Span),
    #[error("While statement missing {0} expression.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.WhileStmtMissingExpression"))]
    WhileStmtMissingExpression(String, Span),
}

impl SemanticErrorKind {
    /// The semantic errors are reported with the span of the syntax that caused the error.
    /// This offset is relative to the start of the file in which the error occurred.
    /// This method is used to adjust the span of the error to be relative to where the
    /// error was reported in the entire compilation unit as part of the source map.
    #[allow(clippy::too_many_lines)]
    fn with_offset(self, offset: u32) -> Self {
        match self {
            Self::AnnotationWithoutStatement(span) => {
                Self::AnnotationWithoutStatement(span + offset)
            }
            Self::CannotCast(lhs, rhs, span) => Self::CannotCast(lhs, rhs, span + offset),
            Self::CastWouldCauseTruncation(lhs, rhs, span) => {
                Self::CastWouldCauseTruncation(lhs, rhs, span + offset)
            }
            Self::CannotAliasType(name, span) => Self::CannotAliasType(name, span + offset),
            Self::CannotApplyOperatorToTypes(op, lhs, rhs, span) => {
                Self::CannotApplyOperatorToTypes(op, lhs, rhs, span + offset)
            }
            Self::CannotAssignToType(lhs, rhs, span) => {
                Self::CannotAssignToType(lhs, rhs, span + offset)
            }
            Self::CannotCallNonGate(span) => Self::CannotCallNonGate(span + offset),
            Self::CannotIndexType(name, span) => Self::CannotIndexType(name, span + offset),
            Self::CannotUpdateConstVariable(name, span) => {
                Self::CannotUpdateConstVariable(name, span + offset)
            }
            Self::ComplexBinaryAssignment(span) => Self::ComplexBinaryAssignment(span + offset),
            Self::DesignatorMustBeIntLiteral(span) => {
                Self::DesignatorMustBeIntLiteral(span + offset)
            }
            Self::FailedToCompileExpressionList(span) => {
                Self::FailedToCompileExpressionList(span + offset)
            }
            Self::ForIterableInvalidExpression(span) => {
                Self::ForIterableInvalidExpression(span + offset)
            }
            Self::ForStatementsMustHaveABodyOrStatement(span) => {
                Self::ForStatementsMustHaveABodyOrStatement(span + offset)
            }
            Self::IfStmtMissingExpression(name, span) => {
                Self::IfStmtMissingExpression(name, span + offset)
            }
            Self::IncludeNotFound(name, span) => Self::IncludeNotFound(name, span + offset),
            Self::IncludeNotInGlobalScope(name, span) => {
                Self::IncludeNotInGlobalScope(name, span + offset)
            }
            Self::IncludeStatementMissingPath(span) => {
                Self::IncludeStatementMissingPath(span + offset)
            }
            Self::IndexMustBeSingleExpr(span) => Self::IndexMustBeSingleExpr(span + offset),
            Self::InvalidAnnotationTarget(span) => Self::InvalidAnnotationTarget(span + offset),
            Self::InvalidControlCount(span) => Self::InvalidControlCount(span + offset),
            Self::InvalidNumberOfClassicalArgs(expected, actual, span) => {
                Self::InvalidNumberOfClassicalArgs(expected, actual, span + offset)
            }
            Self::InvalidNumberOfQubitArgs(expected, actual, span) => {
                Self::InvalidNumberOfQubitArgs(expected, actual, span + offset)
            }
            Self::InvalidCastValueRange(lhs, rhs, span) => {
                Self::InvalidCastValueRange(lhs, rhs, span + offset)
            }
            Self::InvalidGateOperand(span) => Self::InvalidGateOperand(span + offset),
            Self::InvalidIndexedGateOperand(span) => Self::InvalidIndexedGateOperand(span + offset),
            Self::MeasureExpressionsMustHaveGateOperand(span) => {
                Self::MeasureExpressionsMustHaveGateOperand(span + offset)
            }
            Self::MeasureExpressionsMustHaveName(span) => {
                Self::MeasureExpressionsMustHaveName(span + offset)
            }
            Self::NegativeControlCount(span) => Self::NegativeControlCount(span + offset),
            Self::OperatorNotSupportedForTypes(op, lhs, rhs, span) => {
                Self::OperatorNotSupportedForTypes(op, lhs, rhs, span + offset)
            }
            Self::PowModifierMustHaveExponent(span) => {
                Self::PowModifierMustHaveExponent(span + offset)
            }
            Self::QiskitEntryPointMissingOutput(span) => {
                Self::QiskitEntryPointMissingOutput(span + offset)
            }
            Self::QuantumDeclarationInNonGlobalScope(span) => {
                Self::QuantumDeclarationInNonGlobalScope(span + offset)
            }
            Self::QuantumTypesInBinaryExpression(span) => {
                Self::QuantumTypesInBinaryExpression(span + offset)
            }
            Self::RangeExpressionsMustHaveStart(span) => {
                Self::RangeExpressionsMustHaveStart(span + offset)
            }
            Self::RangeExpressionsMustHaveStop(span) => {
                Self::RangeExpressionsMustHaveStop(span + offset)
            }
            Self::RedefinedSymbol(name, span) => Self::RedefinedSymbol(name, span + offset),
            Self::ResetExpressionMustHaveGateOperand(span) => {
                Self::ResetExpressionMustHaveGateOperand(span + offset)
            }
            Self::ResetExpressionMustHaveName(span) => {
                Self::ResetExpressionMustHaveName(span + offset)
            }
            Self::ReturnNotInSubroutine(span) => Self::ReturnNotInSubroutine(span + offset),
            Self::TooManyControls(span) => Self::TooManyControls(span + offset),
            Self::TypeRankError(span) => Self::TypeRankError(span + offset),
            Self::UndefinedSymbol(name, span) => Self::UndefinedSymbol(name, span + offset),
            Self::UnexpectedParserError(error, span) => {
                Self::UnexpectedParserError(error, span + offset)
            }
            Self::UnknownAnnotation(name, span) => Self::UnknownAnnotation(name, span + offset),
            Self::UnknownIndexedOperatorKind(span) => {
                Self::UnknownIndexedOperatorKind(span + offset)
            }
            Self::WhileStmtMissingExpression(name, span) => {
                Self::WhileStmtMissingExpression(name, span + offset)
            }
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

use qsc::{ast::Package, error::WithSource, SourceMap};

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
    ///     variables described as outputs will be returned as output.
    ///     The spec make no mention of endianness or order of the output.
    /// - Otherwise, assume all of the declared variables are returned as output.
    OpenQasm,
    /// No output semantics are applied. The entry point returns `Unit`.
    ResourceEstimation,
}
