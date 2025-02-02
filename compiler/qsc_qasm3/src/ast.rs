// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use indenter::{indented, Indented};
use qsc_data_structures::span::Span;
use std::{
    fmt::{self, Display, Formatter, Write},
    hash::Hash,
    rc::Rc,
};

// TODO: profile this with iai-callgrind in a large OpenQASM3
// sample to verify that is actually faster than using Vec<T>.
/// An alternative to `Vec<T>` that uses less stack space.
type List<T> = Box<[Box<T>]>;

#[derive(Clone, Debug)]
pub struct BreakStmt {
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ContinueStmt {
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub span: Span,
    pub statements: List<Stmt>,
    pub version: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Stmt {
    pub span: Span,
    pub annotations: List<Annotation>,
    pub kind: Box<StmtKind>,
}

#[derive(Clone, Debug)]
pub struct Annotation {
    pub span: Span,
    pub name: Box<PathKind>,
    pub value: Option<Rc<str>>,
}

/// A path that may or may not have been successfully parsed.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum PathKind {
    /// A successfully parsed path.
    Ok(Box<Path>),

    /// An invalid path.
    Err(Option<Box<IncompletePath>>),
}

impl Default for PathKind {
    fn default() -> Self {
        PathKind::Err(None)
    }
}

/// A path that was successfully parsed up to a certain `.`,
/// but is missing its final identifier.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IncompletePath {
    /// The whole span of the incomplete path,
    /// including the final `.` and any whitespace or keyword
    /// that follows it.
    pub span: Span,
    /// Any segments that were successfully parsed before the final `.`.
    pub segments: Box<[Ident]>,
    /// Whether a keyword exists after the final `.`.
    /// This keyword can be presumed to be a partially typed identifier.
    pub keyword: bool,
}

impl Display for PathKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PathKind::Ok(path) => write!(f, "{path}")?,
            PathKind::Err(Some(incomplete_path)) => {
                let mut indent = set_indentation(indented(f), 0);
                write!(indent, "Err IncompletePath {}:", incomplete_path.span)?;
                indent = set_indentation(indent, 1);
                for part in &incomplete_path.segments {
                    write!(indent, "\n{part}")?;
                }
            }
            PathKind::Err(None) => write!(f, "Err",)?,
        }
        Ok(())
    }
}

/// A path to a declaration or a field access expression,
/// to be disambiguated during name resolution.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Path {
    /// The span.
    pub span: Span,
    /// The segments that make up the front of the path before the final `.`.
    pub segments: Option<Box<[Ident]>>,
    /// The declaration or field name.
    pub name: Box<Ident>,
}

fn set_indentation<'a, 'b>(
    indent: Indented<'a, Formatter<'b>>,
    level: usize,
) -> Indented<'a, Formatter<'b>> {
    match level {
        0 => indent.with_str(""),
        1 => indent.with_str("    "),
        2 => indent.with_str("        "),
        _ => unimplemented!("indentation level not supported"),
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.segments.is_none() {
            write!(f, "Path {} ({})", self.span, self.name)?;
        } else {
            let mut indent = set_indentation(indented(f), 0);
            write!(indent, "Path {}:", self.span)?;
            indent = set_indentation(indent, 1);
            if let Some(parts) = &self.segments {
                for part in parts {
                    write!(indent, "\n{part}")?;
                }
            }
            write!(indent, "\n{}", self.name)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum AssignmentExpr {
    Expr(Expr),
    Measurement(MeasureExpr),
}

#[derive(Clone, Debug)]
pub struct MeasureExpr {
    pub span: Span,
    pub operand: GateOperand,
}

/// A binary operator.
#[derive(Clone, Debug)]
pub enum BinOp {
    /// Addition: `+`.
    Add,
    /// Bitwise AND: `&`.
    AndB,
    /// Logical AND: `&&`.
    AndL,
    /// Division: `/`.
    Div,
    /// Equality: `==`.
    Eq,
    /// Exponentiation: `**`.
    Exp,
    /// Greater than: `>`.
    Gt,
    /// Greater than or equal: `>=`.
    Gte,
    /// Less than: `<`.
    Lt,
    /// Less than or equal: `<=`.
    Lte,
    /// Modulus: `%`.
    Mod,
    /// Multiplication: `*`.
    Mul,
    /// Inequality: `!=`.
    Neq,
    /// Bitwise OR: `|`.
    OrB,
    /// Logical OR: `||`.
    OrL,
    /// Shift left: `<<`.
    Shl,
    /// Shift right: `>>`.
    Shr,
    /// Subtraction: `-`.
    Sub,
    /// Bitwise XOR: `^`.
    XorB,
}

/// A unary operator.
#[derive(Clone, Debug)]
pub enum UnOp {
    /// Negation: `-`.
    Neg,
    /// Bitwise NOT: `~`.
    NotB,
    /// Logical NOT: `!`.
    NotL,
}

#[derive(Clone, Debug)]
pub enum GateOperand {
    Ident(Box<Identifier>),
    HardwareQubit(Box<HardwareQubit>),
}

#[derive(Clone, Debug)]
pub struct HardwareQubit {
    pub span: Span,
    pub name: Rc<str>,
}

/// A statement kind.
#[derive(Clone, Debug, Default)]
pub enum StmtKind {
    Alias(Box<Identifier>, Box<List<Expr>>),
    Assign(Box<Identifier>, Box<AssignmentExpr>),
    AssignOp(BinOp, Box<Identifier>, Box<AssignmentExpr>),
    Barrier(BarrierStmt),
    Box(BoxStmt),
    Break(BreakStmt),
    Block(Box<Block>),
    Cal(CalibrationStmt),
    CalibrationGrammar(CalibrationGrammarStmt),
    ClassicalDecl(ClassicalDeclarationStmt),
    ConstDecl(ConstantDeclaration),
    Continue(ContinueStmt),
    Def(DefStmt),
    DefCal(DefCalStmt),
    DelayStmt(DelayStmt),
    /// An empty statement.
    Empty,
    ExprStmt(ExprStmt),
    ExternDecl(ExternDecl),
    For(ForStmt),
    If(IfStmt),
    Include(IncludeStmt),
    IODeclaration(IODeclaration),
    Measure(MeasureStmt),
    QuantumGateDefinition(QuantumGateDefinition),
    QuantumDecl(QubitDeclaration),
    Quantum(QuantumStmt),
    Reset(ResetStmt),
    Return(ReturnStmt),
    Switch(SwitchStmt),
    WhileLoop(WhileLoop),
    /// An invalid statement.
    #[default]
    Err,
}

#[derive(Clone, Debug)]
pub struct CalibrationGrammarStmt {}

#[derive(Clone, Debug)]
pub struct DefCalStmt {}
#[derive(Clone, Debug)]
pub struct IfStmt {}
#[derive(Clone, Debug)]
pub struct DelayStmt {}

#[derive(Clone, Debug)]
pub struct BarrierStmt {
    pub span: Span,
    pub qubits: List<GateOperand>,
}

#[derive(Clone, Debug)]
struct ResetStmt {
    pub span: Span,
    pub operand: Box<GateOperand>,
}

/// A sequenced block of statements.
#[derive(Clone, Debug, Default)]
pub struct Block {
    /// The span.
    pub span: Span,
    /// The statements in the block.
    pub stmts: List<Stmt>,
}

#[derive(Clone, Debug)]
pub enum Identifier {
    Ident(Box<Ident>),
    IndexedIdent(Box<IndexedIdent>),
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Identifier::Ident(ident) => write!(f, "{}", ident),
            Identifier::IndexedIdent(ident) => write!(f, "{}", ident),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Ident {
    pub span: Span,
    pub name: Rc<str>,
}

impl Display for Ident {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Ident {} \"{}\"", self.span, self.name)
    }
}

#[derive(Clone, Debug)]
pub struct IndexedIdent {
    pub span: Span,
    pub name: Ident,
    pub indices: List<IndexElement>,
}

impl Display for IndexedIdent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "IndexedIdent {} \"{}\"", self.span, self.name)
    }
}

#[derive(Clone, Debug)]
pub struct AliasStmt {
    pub span: Span,
    pub kind: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct ExprStmt {
    pub span: Span,
    pub kind: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub span: Span,
    pub kind: Box<ExprKind>,
}

#[derive(Clone, Debug)]
pub struct DiscreteSet {
    pub span: Span,
    pub values: List<ExprStmt>,
}

#[derive(Clone, Debug)]
pub struct RangeDefinition {
    pub span: Span,
    pub start: Option<ExprStmt>,
    pub end: Option<ExprStmt>,
    pub step: Option<ExprStmt>,
}

#[derive(Clone, Debug)]
pub struct QuantumGateModifier {
    pub span: Span,
    pub qubit: Box<Identifier>,
}

#[derive(Clone, Debug)]
pub struct QuantumMeasurement {
    pub span: Span,
    pub qubit: Box<Identifier>,
}

#[derive(Clone, Debug)]
pub struct ClassicalArgument {
    pub span: Span,
    pub r#type: ClassicalType,
    pub name: Identifier,
    pub access: Option<AccessControl>,
}

#[derive(Clone, Debug)]
pub struct ExternArgument {
    pub span: Span,
    pub r#type: ClassicalType,
    pub access: Option<AccessControl>,
}

#[derive(Clone, Debug)]
pub struct ClassicalType {
    pub span: Span,
    pub kind: ClassicalTypeKind,
}

#[derive(Clone, Debug)]
pub enum ClassicalTypeKind {
    Int(IntType),
    UInt(UIntType),
    Float(FloatType),
    Complex(ComplexType),
    Angle(AngleType),
    Bit(BitType),
    BoolType,
    Array(ArrayType),
    ArrayReference(ArrayReferenceType),
    Duration,
    Stretch,
}

#[derive(Clone, Debug)]
pub enum ArrayBaseTypeKind {
    Int(IntType),
    UInt(UIntType),
    Float(FloatType),
    Complex(ComplexType),
    Angle(AngleType),
    Bit(BitType),
    BoolType,
}

#[derive(Clone, Debug)]
pub struct IntType {
    pub size: Option<ExprStmt>,
}

#[derive(Clone, Debug)]
pub struct UIntType {
    pub size: Option<ExprStmt>,
}

#[derive(Clone, Debug)]
pub struct FloatType {
    pub size: Option<ExprStmt>,
}

#[derive(Clone, Debug)]
pub struct ComplexType {
    pub base_size: Option<FloatType>,
}

#[derive(Clone, Debug)]
pub struct AngleType {
    pub size: Option<ExprStmt>,
}

#[derive(Clone, Debug)]
pub struct BitType {
    pub size: Option<ExprStmt>,
}

#[derive(Clone, Debug)]
pub struct ArrayType {
    pub span: Span,
    pub base_type: ArrayBaseTypeKind,
    pub dimensions: List<ExprStmt>,
}

#[derive(Clone, Debug)]
pub struct ArrayReferenceType {
    pub span: Span,
    pub base_type: ArrayBaseTypeKind,
    pub dimensions: List<ExprStmt>,
}

#[derive(Clone, Debug)]
pub enum AccessControl {
    ReadOnly,
    Mutable,
}

#[derive(Clone, Debug)]
pub struct QuantumArgument {
    pub span: Span,
    pub size: Option<ExprStmt>,
}

#[derive(Clone, Debug)]
pub struct Pragma {
    pub span: Span,
    pub name: Box<Path>,
    pub value: Option<Rc<str>>,
}

#[derive(Clone, Debug)]
pub struct CompoundStmt {
    pub span: Span,
    pub statements: List<Stmt>,
}

#[derive(Clone, Debug)]
pub struct IncludeStmt {
    pub span: Span,
    pub filename: String,
}

#[derive(Clone, Debug)]
pub struct QubitDeclaration {
    pub span: Span,
    pub qubit: Identifier,
    pub size: Option<ExprStmt>,
}

#[derive(Clone, Debug)]
pub struct QuantumGateDefinition {
    pub span: Span,
    pub name: Identifier,
    pub arguments: Vec<Identifier>,
    pub qubits: Vec<Identifier>,
    pub body: Vec<Box<Stmt>>,
}

#[derive(Clone, Debug)]
pub struct ExternDecl {
    pub span: Span,
    pub name: Identifier,
    pub arguments: List<ExternArgument>,
    pub return_type: Option<ClassicalType>,
}

#[derive(Clone, Debug)]
pub struct QuantumStmt {
    pub span: Span,
    pub kind: QuantumStmtKind,
}

#[derive(Clone, Debug)]
pub enum QuantumStmtKind {
    Gate(QuantumGate),
    Phase(QuantumPhase),
    Barrier(List<ExprStmt>),
    Reset(List<Box<Identifier>>),
    DelayInstruction(DelayInstruction),
    Box(BoxStmt),
}

#[derive(Clone, Debug)]
pub struct QuantumGate {
    pub span: Span,
    pub modifiers: List<QuantumGateModifier>,
    pub name: Identifier,
    pub args: List<ExprStmt>,
    pub qubits: List<Box<Identifier>>,
    pub duration: Option<ExprStmt>,
}

#[derive(Clone, Debug)]
pub struct QuantumPhase {
    pub span: Span,
    pub modifiers: List<QuantumGateModifier>,
    pub arg: ExprStmt,
    pub qubits: List<Box<Identifier>>,
}

#[derive(Clone, Debug)]
pub struct DelayInstruction {
    span: Span,
    duration: ExprStmt,
    qubits: List<Box<Identifier>>,
}

#[derive(Clone, Debug)]
pub struct BoxStmt {
    pub span: Span,
    pub duration: Option<ExprStmt>,
    pub body: List<QuantumStmt>,
}

#[derive(Clone, Debug)]
pub struct MeasureStmt {
    pub span: Span,
    pub measure: QuantumMeasurement,
    pub target: Option<Box<Identifier>>,
}

#[derive(Clone, Debug)]
pub struct ClassicalDeclarationStmt {
    pub span: Span,
    pub r#type: ClassicalType,
    pub identifier: Identifier,
    pub init_expr: Option<Box<ValueExpression>>,
}

#[derive(Clone, Debug)]
pub enum ValueExpression {
    Expr(ExprStmt),
    Measurement(QuantumMeasurement),
}

#[derive(Clone, Debug)]
pub struct IODeclaration {
    pub span: Span,
    pub io_identifier: IOKeyword,
    pub r#type: ClassicalType,
    pub identifier: Identifier,
}

#[derive(Clone, Debug)]
pub struct ConstantDeclaration {
    span: Span,
    r#type: ClassicalType,
    identifier: Identifier,
    init_expr: ExprStmt,
}

#[derive(Clone, Debug)]
pub struct CalibrationGrammarDeclaration {
    span: Span,
    name: String,
}

#[derive(Clone, Debug)]
pub struct CalibrationStmt {
    span: Span,
    body: String,
}

#[derive(Clone, Debug)]
pub struct CalibrationDefinition {
    span: Span,
    name: Identifier,
    args: List<CalibrationArgument>,
    qubits: List<Identifier>,
    return_type: Option<ClassicalType>,
    body: String,
}

#[derive(Clone, Debug)]
pub enum CalibrationArgument {
    Classical(ClassicalArgument),
    Expr(ExprStmt),
}

#[derive(Clone, Debug)]
pub struct DefStmt {
    span: Span,
    name: Identifier,
    args: List<Box<Operand>>,
    body: List<Box<Stmt>>,
    return_type: Option<ClassicalType>,
}

#[derive(Clone, Debug)]
pub enum Operand {
    Classical(ClassicalArgument),
    Quantum(QuantumArgument),
}

#[derive(Clone, Debug)]
pub struct ReturnStmt {
    span: Span,
    expr: Option<Box<ValueExpression>>,
}

#[derive(Clone, Debug)]
pub struct BranchingStmt {
    span: Span,
    condition: ExprStmt,
    if_block: List<Stmt>,
    else_block: List<Stmt>,
}

#[derive(Clone, Debug)]
pub struct WhileLoop {
    span: Span,
    while_condition: ExprStmt,
    block: List<Stmt>,
}

#[derive(Clone, Debug)]
pub struct ForStmt {
    span: Span,
    r#type: ClassicalType,
    identifier: Identifier,
    set_declaration: Box<EnumerableSet>,
    block: List<Stmt>,
}

#[derive(Clone, Debug)]
pub enum EnumerableSet {
    DiscreteSet(DiscreteSet),
    RangeDefinition(RangeDefinition),
    Expr(ExprStmt),
}

#[derive(Clone, Debug)]
pub struct SwitchStmt {
    pub span: Span,
    pub target: ExprStmt,
    pub cases: List<(List<ExprStmt>, CompoundStmt)>,
    /// Note that `None` is quite different to `[]` in this case; the latter is
    /// an explicitly empty body, whereas the absence of a default might mean
    /// that the switch is inexhaustive, and a linter might want to complain.
    pub default: Option<CompoundStmt>,
}

#[derive(Clone, Debug)]
pub struct ClassicalAssignment {
    pub span: Span,
    pub lvalue: Identifier,
    pub op: AssignmentOp,
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Ident(Ident),
    UnaryExpr(UnaryExpr),
    BinaryExpr(BinaryExpr),
    Literal(Lit),
    FunctionCall(FunctionCall),
    Cast(Cast),
    Concatenation(Concatenation),
    IndexExpr(IndexExpr),
}

#[derive(Clone, Debug)]
pub struct UnaryExpr {
    pub span: Span,
    pub op: UnaryOp,
    pub expr: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct BinaryExpr {
    pub span: Span,
    pub op: BinaryOp,
    pub lhs: ExprStmt,
    pub rhs: ExprStmt,
}
#[derive(Clone, Debug)]
pub struct FunctionCall {
    pub span: Span,
    pub name: Identifier,
    pub args: List<ExprStmt>,
}

#[derive(Clone, Debug)]
pub struct Cast {
    pub span: Span,
    pub r#type: ClassicalType,
    pub arg: ExprStmt,
}

#[derive(Clone, Debug)]
pub struct IndexExpr {
    pub span: Span,
    pub collection: ExprStmt,
    pub index: IndexElement,
}

#[derive(Clone, Debug)]
pub enum UnaryOp {
    NegB,
    NegL,
    NegN,
}

#[derive(Clone, Debug)]
pub enum BinaryOp {
    /// `>`
    Gt,
    /// `<`
    Lt,
    /// `>=`
    Gte,
    /// `<=`
    Lte,
    /// `==`
    Eq,
    /// `!=`
    Neq,
    /// `&&`
    AndL,
    /// `||`
    OrL,
    /// `|`
    OrB,
    /// `^`
    XorB,
    /// `&`
    AndB,
    /// `<<`
    ShL,
    /// `>>`
    ShR,
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `%`
    Mod,
    /// `**`
    Exp,
}

#[derive(Clone, Debug)]
pub struct Lit {
    pub span: Span,
    pub kind: LiteralKind,
}

#[derive(Clone, Debug)]
pub enum LiteralKind {
    Array(List<ExprStmt>),
    Bitstring { value: usize, width: u32 },
    Boolean(bool),
    Duration { value: f64, unit: TimeUnit },
    Float(f64),
    Imaginary(f64),
    Integer(i64),
    String(Rc<str>),
}

#[derive(Clone, Debug)]
pub struct Concatenation {
    lhs: ExprStmt,
    rhs: ExprStmt,
}

#[derive(Clone, Debug)]
pub enum IndexElement {
    DiscreteSet(DiscreteSet),
    IndexSet(List<IndexSetItem>),
}

#[derive(Clone, Debug)]
pub enum IndexSetItem {
    RangeDefinition(RangeDefinition),
    Expr(ExprStmt),
}

#[derive(Clone, Debug)]
pub enum AssignmentOp {
    BinaryOp(BinaryOp),
    /// OpenQASM3 has the `~=` assignment operator.
    /// This enum variant is meant to capture that.
    UnaryOp(UnaryOp),
    Assign,
}

#[derive(Clone, Debug)]
pub enum GateModifierName {
    Inv,
    Pow,
    Ctrl,
    NegCtrl,
}

#[derive(Clone, Debug)]
pub enum IOKeyword {
    Input,
    Output,
}

#[derive(Clone, Debug)]
pub enum TimeUnit {
    Dt,
    /// Nanoseconds.
    Ns,
    /// Microseconds.
    Us,
    /// Milliseconds.
    Ms,
    /// Seconds.
    S,
}
