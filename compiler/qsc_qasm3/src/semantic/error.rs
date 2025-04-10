// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::Diagnostic;
use qsc_data_structures::span::Span;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Eq, Error, PartialEq)]
#[error(transparent)]
#[diagnostic(transparent)]
pub struct Error(pub SemanticErrorKind);

/// Represents the kind of semantic error that occurred during lowering of a QASM file(s).
/// For the most part, these errors are fatal and prevent compilation and are
/// safety checks to ensure that the QASM code is valid.
///
/// We can't use the semantics library for this:
///   - it is unsafe to use (heavy use of panic and unwrap)
///   - it is missing many language features
#[derive(Clone, Debug, Diagnostic, Eq, Error, PartialEq)]
pub enum SemanticErrorKind {
    #[error("annotation missing target statement")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.AnnotationWithoutStatement"))]
    AnnotationWithoutStatement(#[label] Span),
    #[error("array literals are only allowed in classical declarations")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.ArrayLiteralInNonClassicalDecl"))]
    ArrayLiteralInNonClassicalDecl(#[label] Span),
    #[error("array size must be a non-negative integer const expression")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.ArraySizeMustBeNonNegativeConstExpr"))]
    ArraySizeMustBeNonNegativeConstExpr(#[label] Span),
    #[error("calibration statements are not supported: {0}")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.CalibrationsNotSupported"))]
    CalibrationsNotSupported(String, #[label] Span),
    #[error("cannot alias type {0}. Only qubit and qubit[] can be aliased")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.CannotAliasType"))]
    CannotAliasType(String, #[label] Span),
    #[error("cannot apply operator {0} to types {1} and {2}")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.CannotApplyOperatorToTypes"))]
    CannotApplyOperatorToTypes(String, String, String, #[label] Span),
    #[error("cannot assign a value of {0} type to a classical variable of {1} type")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.CannotAssignToType"))]
    CannotAssignToType(String, String, #[label] Span),
    #[error("cannot call an expression that is not a function")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.CannotCallNonFunction"))]
    CannotCallNonFunction(#[label] Span),
    #[error("cannot call a gate that is not a gate")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.CannotCallNonGate"))]
    CannotCallNonGate(#[label] Span),
    #[error("cannot cast expression of type {0} to type {1}")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.CannotCast"))]
    CannotCast(String, String, #[label] Span),
    #[error("cannot cast literal expression of type {0} to type {1}")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.CannotCastLiteral"))]
    CannotCastLiteral(String, String, #[label] Span),
    #[error("cannot index variables of type {0}")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.CannotIndexType"))]
    CannotIndexType(String, #[label] Span),
    #[error("cannot update const variable {0}")]
    #[diagnostic(help("mutable variables must be declared without the keyword `const`"))]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.CannotUpdateConstVariable"))]
    CannotUpdateConstVariable(String, #[label] Span),
    #[error("cannot cast expression of type {0} to type {1} as it would cause truncation")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.CastWouldCauseTruncation"))]
    CastWouldCauseTruncation(String, String, #[label] Span),
    #[error("invalid classical statement in box")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.ClassicalStmtInBox"))]
    ClassicalStmtInBox(#[label] Span),
    #[error("complex numbers in assignment binary expressions are not yet supported")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.ComplexBinaryAssignment"))]
    ComplexBinaryAssignment(#[label] Span),
    #[error("designator must be a positive literal integer")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.DesignatorMustBePositiveIntLiteral"))]
    DesignatorMustBePositiveIntLiteral(#[label] Span),
    #[error("def declarations must be done in global scope")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.DefDeclarationInNonGlobalScope"))]
    DefDeclarationInNonGlobalScope(#[label] Span),
    #[error("designator is too large")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.DesignatorTooLarge"))]
    DesignatorTooLarge(#[label] Span),
    #[error("{0} must be a const expression")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.ExprMustBeConst"))]
    ExprMustBeConst(String, #[label] Span),
    #[error("{0} must fit in a u32")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.ExprMustFitInU32"))]
    ExprMustFitInU32(String, #[label] Span),
    #[error("extern declarations must be done in global scope")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.DefDeclarationInNonGlobalScope"))]
    ExternDeclarationInNonGlobalScope(#[label] Span),
    #[error("failed to compile all expressions in expression list")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.FailedToCompileExpressionList"))]
    FailedToCompileExpressionList(#[label] Span),
    #[error("for iterable must have a set expression, range expression, or iterable expression")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.ForIterableInvalidExpression"))]
    ForIterableInvalidExpression(#[label] Span),
    #[error("for statements must have a body or statement")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.ForStatementsMustHaveABodyOrStatement"))]
    ForStatementsMustHaveABodyOrStatement(#[label] Span),
    #[error("if statement missing {0} expression")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.IfStmtMissingExpression"))]
    IfStmtMissingExpression(String, #[label] Span),
    #[error("include {0} could not be found")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.IncludeNotFound"))]
    IncludeNotFound(String, #[label] Span),
    #[error("include {0} must be declared in global scope")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.IncludeNotInGlobalScope"))]
    IncludeNotInGlobalScope(String, #[label] Span),
    #[error("include {0} must be declared in global scope")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.IncludeStatementMissingPath"))]
    IncludeStatementMissingPath(#[label] Span),
    #[error("inconsistent types in alias expression: {0}")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.InconsistentTypesInAlias"))]
    InconsistentTypesInAlias(String, #[label] Span),
    #[error("indexed must be a single expression")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.IndexMustBeSingleExpr"))]
    IndexMustBeSingleExpr(#[label] Span),
    #[error("assigning {0} values to {1} must be in a range that be converted to {1}")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.InvalidCastValueRange"))]
    InvalidCastValueRange(String, String, #[label] Span),
    #[error("gate operands other than qubits or qubit arrays are not supported")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.InvalidGateOperand"))]
    InvalidGateOperand(#[label] Span),
    #[error("control counts must be integer literals")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.InvalidControlCount"))]
    InvalidControlCount(#[label] Span),
    #[error("gate operands other than qubit arrays are not supported")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.InvalidIndexedGateOperand"))]
    InvalidIndexedGateOperand(#[label] Span),
    #[error("gate expects {0} classical arguments, but {1} were provided")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.InvalidNumberOfClassicalArgs"))]
    InvalidNumberOfClassicalArgs(usize, usize, #[label] Span),
    #[error("gate expects {0} qubit arguments, but {1} were provided")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.InvalidNumberOfQubitArgs"))]
    InvalidNumberOfQubitArgs(usize, usize, #[label] Span),
    #[error("{0} can only appear in {1} scopes")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.InvalidScope"))]
    InvalidScope(String, String, #[label] Span),
    #[error("measure statements must have a name")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.MeasureExpressionsMustHaveName"))]
    MeasureExpressionsMustHaveName(#[label] Span),
    #[error("measure statements must have a gate operand name")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.MeasureExpressionsMustHaveGateOperand"))]
    MeasureExpressionsMustHaveGateOperand(#[label] Span),
    #[error("return statements on a non-void subroutine should have a target expression")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.MissingTargetExpressionInReturnStmt"))]
    MissingTargetExpressionInReturnStmt(#[label] Span),
    #[error("control counts must be postitive integers")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.NegativeControlCount"))]
    NegativeControlCount(#[label] Span),
    #[error("{0} are not supported")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.NotSupported"))]
    NotSupported(String, #[label] Span),
    #[error("{0} were introduced in version {1}")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.NotSupportedInThisVersion"))]
    NotSupportedInThisVersion(String, String, #[label] Span),
    #[error("the operator {0} is not valid with lhs {1} and rhs {2}")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.OperatorNotSupportedForTypes"))]
    OperatorNotSupportedForTypes(String, String, String, #[label] Span),
    #[error("pow gate modifiers must have an exponent")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.PowModifierMustHaveExponent"))]
    PowModifierMustHaveExponent(#[label] Span),
    #[error("quantum declarations must be done in global scope")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.QuantumDeclarationInNonGlobalScope"))]
    QuantumDeclarationInNonGlobalScope(#[label] Span),
    #[error("quantum typed values cannot be used in binary expressions")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.QuantumTypesInBinaryExpression"))]
    QuantumTypesInBinaryExpression(#[label] Span),
    #[error("range expressions must have a start")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.RangeExpressionsMustHaveStart"))]
    RangeExpressionsMustHaveStart(#[label] Span),
    #[error("range expressions must have a stop")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.RangeExpressionsMustHaveStop"))]
    RangeExpressionsMustHaveStop(#[label] Span),
    #[error("redefined symbol: {0}")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.RedefinedSymbol"))]
    RedefinedSymbol(String, #[label] Span),
    #[error("reset expression must have a gate operand")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.ResetExpressionMustHaveGateOperand"))]
    ResetExpressionMustHaveGateOperand(#[label] Span),
    #[error("reset expression must have a name")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.ResetExpressionMustHaveName"))]
    ResetExpressionMustHaveName(#[label] Span),
    #[error("cannot return an expression from a void subroutine")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.ReturningExpressionFromVoidSubroutine"))]
    ReturningExpressionFromVoidSubroutine(#[label] Span),
    #[error("return statements are only allowed within subroutines")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.ReturnNotInSubroutine"))]
    ReturnNotInSubroutine(#[label] Span),
    #[error("switch statement must have at least one non-default case")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.SwitchStatementMustHaveAtLeastOneCase"))]
    SwitchStatementMustHaveAtLeastOneCase(#[label] Span),
    #[error("too many controls specified")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.TooManyControls"))]
    TooManyControls(#[label] Span),
    #[error("too many indicies specified")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.TooManyIndices"))]
    TooManyIndices(#[label] Span),
    #[error("bitwise not `~` is not allowed for instances of {0}")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.TypeDoesNotSupportBitwiseNot"))]
    TypeDoesNotSupportBitwiseNot(String, #[label] Span),
    #[error("unary negation is not allowed for instances of {0}")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.TypeDoesNotSupportedUnaryNegation"))]
    TypeDoesNotSupportedUnaryNegation(String, #[label] Span),
    #[error("{0} max width is {1} but {2} was provided")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.TypeMaxWidthExceeded"))]
    TypeMaxWidthExceeded(String, usize, usize, #[label] Span),
    #[error("types differ by dimensions and are incompatible")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.TypeRankError"))]
    TypeRankError(#[label] Span),
    #[error("type width must be a positive integer const expression")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.TypeWidthMustBePositiveIntConstExpr"))]
    TypeWidthMustBePositiveIntConstExpr(#[label] Span),
    #[error("undefined symbol: {0}")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.UndefinedSymbol"))]
    UndefinedSymbol(String, #[label] Span),
    #[error("unexpected parser error: {0}")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.UnexpectedParserError"))]
    UnexpectedParserError(String, #[label] Span),
    #[error("this statement is not yet handled during OpenQASM 3 import: {0}")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.Unimplemented"))]
    Unimplemented(String, #[label] Span),
    #[error("unknown index operation kind")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.UnknownIndexedOperatorKind"))]
    UnknownIndexedOperatorKind(#[label] Span),
    #[error("unsupported version: '{0}'")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.UnsupportedVersion"))]
    UnsupportedVersion(String, #[label] Span),
    #[error("while statement missing {0} expression")]
    #[diagnostic(code("Qsc.Qasm3.Lowerer.WhileStmtMissingExpression"))]
    WhileStmtMissingExpression(String, #[label] Span),
}

impl From<Error> for crate::Error {
    fn from(val: Error) -> Self {
        crate::Error(crate::ErrorKind::Semantic(val))
    }
}
