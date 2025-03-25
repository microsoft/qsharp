// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::Diagnostic;
use qsc_data_structures::span::Span;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Eq, Error, PartialEq)]
#[error(transparent)]
#[diagnostic(transparent)]
pub struct Error(pub SemanticErrorKind);

impl Error {
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
pub enum SemanticErrorKind {
    #[error("Array literals are only allowed in classical declarations.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.ArrayLiteralInNonClassicalDecl"))]
    ArrayLiteralInNonClassicalDecl(#[label] Span),
    #[error("{0} must fit in a u32")]
    #[diagnostic(code("Qsc.Qasm3.Compile.ExprMustFitInU32"))]
    ExprMustFitInU32(String, #[label] Span),
    #[error("{0} must be a const expression")]
    #[diagnostic(code("Qsc.Qasm3.Compile.ExprMustBeConst"))]
    ExprMustBeConst(String, #[label] Span),
    #[error("Annotation missing target statement.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.AnnotationWithoutStatement"))]
    AnnotationWithoutStatement(#[label] Span),
    #[error("calibration statements are not supported: {0}")]
    #[diagnostic(code("Qsc.Qasm3.Compile.CalibrationsNotSupported"))]
    CalibrationsNotSupported(String, #[label] Span),
    #[error("Cannot alias type {0}. Only qubit and qubit[] can be aliased.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.CannotAliasType"))]
    CannotAliasType(String, #[label] Span),
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
    #[error("Cannot cast literal expression of type {0} to type {1}")]
    #[diagnostic(code("Qsc.Qasm3.Compile.CannotCastLiteral"))]
    CannotCastLiteral(String, String, #[label] Span),
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
    #[error("invalid classical statement in box")]
    #[diagnostic(code("Qsc.Qasm3.Compile.ClassicalStmtInBox"))]
    ClassicalStmtInBox(#[label] Span),
    #[error("Complex numbers in assignment binary expressions are not yet supported.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.ComplexBinaryAssignment"))]
    ComplexBinaryAssignment(#[label] Span),
    #[error("Designator must be a positive literal integer.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.DesignatorMustBePositiveIntLiteral"))]
    DesignatorMustBePositiveIntLiteral(#[label] Span),
    #[error("Type width must be a positive integer const expression.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.TypeWidthMustBePositiveIntConstExpr"))]
    TypeWidthMustBePositiveIntConstExpr(#[label] Span),
    #[error("Array size must be a non-negative integer const expression.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.ArraySizeMustBeNonNegativeConstExpr"))]
    ArraySizeMustBeNonNegativeConstExpr(#[label] Span),
    #[error("Designator is too large.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.DesignatorTooLarge"))]
    DesignatorTooLarge(#[label] Span),
    #[error("Failed to compile all expressions in expression list.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.FailedToCompileExpressionList"))]
    FailedToCompileExpressionList(#[label] Span),
    #[error("For iterable must have a set expression, range expression, or iterable expression.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.ForIterableInvalidExpression"))]
    ForIterableInvalidExpression(#[label] Span),
    #[error("For statements must have a body or statement.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.ForStatementsMustHaveABodyOrStatement"))]
    ForStatementsMustHaveABodyOrStatement(#[label] Span),
    #[error("Inconsisten types in alias expression: {0}.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.InconsistentTypesInAlias"))]
    InconsistentTypesInAlias(String, #[label] Span),
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
    InvalidAnnotationTarget(#[label] Span),
    #[error("Assigning {0} values to {1} must be in a range that be converted to {1}.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.InvalidCastValueRange"))]
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
    #[error("{0} are not supported.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.NotSupported"))]
    NotSupported(String, #[label] Span),
    #[error("{0} were introduced in version {1}")]
    #[diagnostic(code("Qsc.Qasm3.Compile.NotSupportedInThisVersion"))]
    NotSupportedInThisVersion(String, String, #[label] Span),
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
    #[error("Switch statement must have at least one non-default case.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.SwitchStatementMustHaveAtLeastOneCase"))]
    SwitchStatementMustHaveAtLeastOneCase(#[label] Span),
    #[error("Too many controls specified.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.TooManyControls"))]
    TooManyControls(#[label] Span),
    #[error("Too many indicies specified.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.TooManyIndices"))]
    TooManyIndices(#[label] Span),
    #[error("Bitwise not `~` is not allowed for instances of {0}.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.TypeDoesNotSupportBitwiseNot"))]
    TypeDoesNotSupportBitwiseNot(String, #[label] Span),
    #[error("Unary negation is not allowed for instances of {0}.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.TypeDoesNotSupportedUnaryNegation"))]
    TypeDoesNotSupportedUnaryNegation(String, #[label] Span),
    #[error("Types differ by dimensions and are incompatible.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.TypeRankError"))]
    TypeRankError(#[label] Span),
    #[error("Undefined symbol: {0}.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.UndefinedSymbol"))]
    UndefinedSymbol(String, #[label] Span),
    #[error("Unexpected parser error: {0}.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.UnexpectedParserError"))]
    UnexpectedParserError(String, #[label] Span),
    #[error("this statement is not yet handled during OpenQASM 3 import: {0}")]
    #[diagnostic(code("Qsc.Qasm3.Compile.Unimplemented"))]
    Unimplemented(String, #[label] Span),
    #[error("Unexpected annotation: {0}.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.UnknownAnnotation"))]
    UnknownAnnotation(String, #[label] Span),
    #[error("Unknown index operation kind.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.UnknownIndexedOperatorKind"))]
    UnknownIndexedOperatorKind(#[label] Span),
    #[error("Unsupported version: '{0}'.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.UnsupportedVersion"))]
    UnsupportedVersion(String, #[label] Span),
    #[error("While statement missing {0} expression.")]
    #[diagnostic(code("Qsc.Qasm3.Compile.WhileStmtMissingExpression"))]
    WhileStmtMissingExpression(String, #[label] Span),
}

impl SemanticErrorKind {
    /// The semantic errors are reported with the span of the syntax that caused the error.
    /// This offset is relative to the start of the file in which the error occurred.
    /// This method is used to adjust the span of the error to be relative to where the
    /// error was reported in the entire compilation unit as part of the source map.
    #[allow(clippy::too_many_lines)]
    fn with_offset(self, offset: u32) -> Self {
        match self {
            Self::ArrayLiteralInNonClassicalDecl(span) => {
                Self::ArrayLiteralInNonClassicalDecl(span + offset)
            }
            Self::ExprMustBeConst(name, span) => Self::ExprMustBeConst(name, span + offset),
            Self::ExprMustFitInU32(name, span) => Self::ExprMustFitInU32(name, span + offset),
            Self::AnnotationWithoutStatement(span) => {
                Self::AnnotationWithoutStatement(span + offset)
            }
            Self::CannotCast(lhs, rhs, span) => Self::CannotCast(lhs, rhs, span + offset),
            Self::CannotCastLiteral(lhs, rhs, span) => {
                Self::CannotCastLiteral(lhs, rhs, span + offset)
            }
            Self::CastWouldCauseTruncation(lhs, rhs, span) => {
                Self::CastWouldCauseTruncation(lhs, rhs, span + offset)
            }
            Self::ClassicalStmtInBox(span) => Self::ClassicalStmtInBox(span + offset),
            Self::CalibrationsNotSupported(name, span) => {
                Self::CalibrationsNotSupported(name, span + offset)
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
            Self::DesignatorMustBePositiveIntLiteral(span) => {
                Self::DesignatorMustBePositiveIntLiteral(span + offset)
            }
            Self::TypeWidthMustBePositiveIntConstExpr(span) => {
                Self::TypeWidthMustBePositiveIntConstExpr(span + offset)
            }
            Self::ArraySizeMustBeNonNegativeConstExpr(span) => {
                Self::ArraySizeMustBeNonNegativeConstExpr(span + offset)
            }
            Self::DesignatorTooLarge(span) => Self::DesignatorTooLarge(span + offset),
            Self::FailedToCompileExpressionList(span) => {
                Self::FailedToCompileExpressionList(span + offset)
            }
            Self::ForIterableInvalidExpression(span) => {
                Self::ForIterableInvalidExpression(span + offset)
            }
            Self::ForStatementsMustHaveABodyOrStatement(span) => {
                Self::ForStatementsMustHaveABodyOrStatement(span + offset)
            }
            Self::InconsistentTypesInAlias(name, span) => {
                Self::InconsistentTypesInAlias(name, span + offset)
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
            Self::NotSupported(name, span) => Self::NotSupported(name, span + offset),
            Self::NotSupportedInThisVersion(name, version, span) => {
                Self::NotSupportedInThisVersion(name, version, span + offset)
            }
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
            Self::SwitchStatementMustHaveAtLeastOneCase(span) => {
                Self::SwitchStatementMustHaveAtLeastOneCase(span + offset)
            }
            Self::TooManyControls(span) => Self::TooManyControls(span + offset),
            Self::TooManyIndices(span) => Self::TooManyIndices(span + offset),
            Self::TypeDoesNotSupportBitwiseNot(name, span) => {
                Self::TypeDoesNotSupportBitwiseNot(name, span + offset)
            }
            Self::TypeDoesNotSupportedUnaryNegation(name, span) => {
                Self::TypeDoesNotSupportedUnaryNegation(name, span + offset)
            }
            Self::TypeRankError(span) => Self::TypeRankError(span + offset),
            Self::UndefinedSymbol(name, span) => Self::UndefinedSymbol(name, span + offset),
            Self::UnexpectedParserError(error, span) => {
                Self::UnexpectedParserError(error, span + offset)
            }
            Self::Unimplemented(name, span) => Self::Unimplemented(name, span + offset),
            Self::UnknownAnnotation(name, span) => Self::UnknownAnnotation(name, span + offset),
            Self::UnknownIndexedOperatorKind(span) => {
                Self::UnknownIndexedOperatorKind(span + offset)
            }
            Self::UnsupportedVersion(version, span) => {
                Self::UnsupportedVersion(version, span + offset)
            }
            Self::WhileStmtMissingExpression(name, span) => {
                Self::WhileStmtMissingExpression(name, span + offset)
            }
        }
    }
}

impl From<Error> for crate::Error {
    fn from(val: Error) -> Self {
        crate::Error(crate::ErrorKind::Semantic(val))
    }
}
