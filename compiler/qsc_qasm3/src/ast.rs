// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// while we work through the conversion, allow dead code to avoid warnings
#![allow(dead_code)]

mod display_utils;

use display_utils::{
    write_field, write_header, write_indented_list, write_list_field, write_opt_field,
    write_opt_list_field, writeln_field, writeln_header, writeln_list_field, writeln_opt_field,
};
use num_bigint::BigInt;
use qsc_data_structures::span::{Span, WithSpan};
use std::{
    fmt::{self, Display, Formatter},
    hash::Hash,
    rc::Rc,
};

// TODO: Profile this with iai-callgrind in a large OpenQASM3
//       sample to verify that is actually faster than using Vec<T>.
//       Even though Box<T> uses less stack space, it reduces cache
//       locality, because now you need to be jumping around in
//       memory to read contiguous elements of a list.
/// An alternative to `Vec<T>` that uses less stack space.
pub(crate) type List<T> = Box<[Box<T>]>;

pub(crate) fn list_from_iter<T>(vals: impl IntoIterator<Item = T>) -> List<T> {
    vals.into_iter().map(Box::new).collect()
}

#[derive(Clone, Debug)]
pub struct Program {
    pub span: Span,
    pub statements: List<Stmt>,
    pub version: Option<Version>,
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "Program", self.span)?;
        writeln_opt_field(f, "version", self.version.as_ref())?;
        write_list_field(f, "program stmts", &self.statements)
    }
}

#[derive(Clone, Debug)]
pub struct Stmt {
    pub span: Span,
    pub annotations: List<Annotation>,
    pub kind: Box<StmtKind>,
}

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "Stmt", self.span)?;
        writeln_list_field(f, "annotations", &self.annotations)?;
        write_field(f, "kind", &self.kind)
    }
}

#[derive(Clone, Debug)]
pub struct Annotation {
    pub span: Span,
    pub identifier: Rc<str>,
    pub value: Option<Rc<str>>,
}
impl Display for Annotation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(value) = &self.value {
            write!(
                f,
                "Annotation {}: ({}, {})",
                self.span, self.identifier, value
            )
        } else {
            write!(f, "Annotation {}: ({})", self.span, self.identifier)
        }
    }
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

impl Display for PathKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PathKind::Ok(path) => write!(f, "{path}"),
            PathKind::Err(Some(incomplete_path)) => {
                write!(f, "Err IncompletePath {}:", incomplete_path.span)?;
                write_list_field(f, "segments", &incomplete_path.segments)
            }
            PathKind::Err(None) => write!(f, "Err",),
        }
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

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln_header(f, "Path", self.span)?;
        writeln_field(f, "name", &self.name)?;
        write_opt_list_field(f, "segments", self.segments.as_ref())
    }
}

impl WithSpan for Path {
    fn with_span(self, span: Span) -> Self {
        Self { span, ..self }
    }
}

#[derive(Clone, Debug)]
pub enum AssignmentExpr {
    Expr(Expr),
    Measurement(MeasureExpr),
}

impl Display for AssignmentExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AssignmentExpr::Expr(expr) => write!(f, "AssignmentExpr {expr}"),
            AssignmentExpr::Measurement(measure) => write!(f, "AssignmentExpr {measure}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MeasureExpr {
    pub span: Span,
    pub operand: GateOperand,
}

impl Display for MeasureExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "MeasureExpr", self.span)?;
        write_field(f, "operand", &self.operand)
    }
}

/// A binary operator.
#[derive(Clone, Copy, Debug)]
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

impl Display for BinOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "Add"),
            BinOp::AndB => write!(f, "AndB"),
            BinOp::AndL => write!(f, "AndL"),
            BinOp::Div => write!(f, "Div"),
            BinOp::Eq => write!(f, "Eq"),
            BinOp::Exp => write!(f, "Exp"),
            BinOp::Gt => write!(f, "Gt"),
            BinOp::Gte => write!(f, "Gte"),
            BinOp::Lt => write!(f, "Lt"),
            BinOp::Lte => write!(f, "Lte"),
            BinOp::Mod => write!(f, "Mod"),
            BinOp::Mul => write!(f, "Mul"),
            BinOp::Neq => write!(f, "Neq"),
            BinOp::OrB => write!(f, "OrB"),
            BinOp::OrL => write!(f, "OrL"),
            BinOp::Shl => write!(f, "Shl"),
            BinOp::Shr => write!(f, "Shr"),
            BinOp::Sub => write!(f, "Sub"),
            BinOp::XorB => write!(f, "XorB"),
        }
    }
}

/// A unary operator.
#[derive(Clone, Copy, Debug)]
pub enum UnaryOp {
    /// Negation: `-`.
    Neg,
    /// Bitwise NOT: `~`.
    NotB,
    /// Logical NOT: `!`.
    NotL,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Neg => write!(f, "Neg"),
            UnaryOp::NotB => write!(f, "NotB"),
            UnaryOp::NotL => write!(f, "NotL"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum GateOperand {
    IndexedIdent(Box<IndexedIdent>),
    HardwareQubit(Box<HardwareQubit>),
    #[default]
    Err,
}

impl Display for GateOperand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            GateOperand::IndexedIdent(ident) => write!(f, "GateOperand {ident}"),
            GateOperand::HardwareQubit(qubit) => write!(f, "GateOperand {qubit}"),
            GateOperand::Err => write!(f, "Error"),
        }
    }
}

impl WithSpan for GateOperand {
    fn with_span(self, _span: Span) -> Self {
        self
    }
}

#[derive(Clone, Debug)]
pub struct HardwareQubit {
    pub span: Span,
    pub name: Rc<str>,
}

impl Display for HardwareQubit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "HardwareQubit {}: {}", self.span, self.name)
    }
}

#[derive(Clone, Debug)]
pub struct AliasDeclStmt {
    pub span: Span,
    pub ident: Identifier,
    pub exprs: List<Expr>,
}

impl Display for AliasDeclStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "AliasDeclStmt", self.span)?;
        writeln_field(f, "ident", &self.ident)?;
        write_list_field(f, "exprs", &self.exprs)
    }
}

#[derive(Clone, Debug)]
pub struct Assign {
    pub ident: Box<Identifier>,
    pub expr: Box<AssignmentExpr>,
    pub span: Span,
}

impl Display for Assign {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Assign {}: {}, {}", self.span, self.ident, self.expr)
    }
}

#[derive(Clone, Debug)]
pub struct AssignOp {
    pub op: BinOp,
    pub ident: Box<Identifier>,
    pub expr: Box<AssignmentExpr>,
    pub span: Span,
}

impl Display for AssignOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AssignOp {}: {}, {}, {}",
            self.span, self.op, self.ident, self.expr
        )
    }
}

/// A statement kind.
#[derive(Clone, Debug, Default)]
pub enum StmtKind {
    Alias(AliasDeclStmt),
    Assign(Assign),
    AssignOp(AssignOp),
    Barrier(BarrierStmt),
    Box(BoxStmt),
    Break(BreakStmt),
    Block(Box<Block>),
    Cal(CalibrationStmt),
    CalibrationGrammar(CalibrationGrammarStmt),
    ClassicalDecl(ClassicalDeclarationStmt),
    ConstDecl(ConstantDeclStmt),
    Continue(ContinueStmt),
    Def(DefStmt),
    DefCal(DefCalStmt),
    Delay(DelayStmt),
    /// An empty statement.
    Empty,
    End(EndStmt),
    ExprStmt(ExprStmt),
    ExternDecl(ExternDecl),
    For(ForStmt),
    If(IfStmt),
    GateCall(GateCall),
    GPhase(GPhase),
    Include(IncludeStmt),
    IODeclaration(IODeclaration),
    Measure(MeasureStmt),
    Pragma(Pragma),
    QuantumGateDefinition(QuantumGateDefinition),
    QuantumDecl(QubitDeclaration),
    Reset(ResetStmt),
    Return(ReturnStmt),
    Switch(SwitchStmt),
    WhileLoop(WhileLoop),
    /// An invalid statement.
    #[default]
    Err,
}

impl Display for StmtKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StmtKind::Alias(alias) => write!(f, "{alias}"),
            StmtKind::Assign(assign) => write!(f, "{assign}"),
            StmtKind::AssignOp(assign_op) => write!(f, "{assign_op}"),
            StmtKind::Barrier(barrier) => write!(f, "{barrier}"),
            StmtKind::Box(box_stmt) => write!(f, "{box_stmt}"),
            StmtKind::Break(break_stmt) => write!(f, "{break_stmt}"),
            StmtKind::Block(block) => write!(f, "{block}"),
            StmtKind::Cal(cal) => write!(f, "{cal}"),
            StmtKind::CalibrationGrammar(grammar) => write!(f, "{grammar}"),
            StmtKind::ClassicalDecl(decl) => write!(f, "{decl}"),
            StmtKind::ConstDecl(decl) => write!(f, "{decl}"),
            StmtKind::Continue(continue_stmt) => write!(f, "{continue_stmt}"),
            StmtKind::Def(def) => write!(f, "{def}"),
            StmtKind::DefCal(defcal) => write!(f, "{defcal}"),
            StmtKind::Delay(delay) => write!(f, "{delay}"),
            StmtKind::Empty => write!(f, "Empty"),
            StmtKind::End(end_stmt) => write!(f, "{end_stmt}"),
            StmtKind::ExprStmt(expr) => write!(f, "{expr}"),
            StmtKind::ExternDecl(decl) => write!(f, "{decl}"),
            StmtKind::For(for_stmt) => write!(f, "{for_stmt}"),
            StmtKind::GateCall(gate_call) => write!(f, "{gate_call}"),
            StmtKind::GPhase(gphase) => write!(f, "{gphase}"),
            StmtKind::If(if_stmt) => write!(f, "{if_stmt}"),
            StmtKind::Include(include) => write!(f, "{include}"),
            StmtKind::IODeclaration(io) => write!(f, "{io}"),
            StmtKind::Measure(measure) => write!(f, "{measure}"),
            StmtKind::Pragma(pragma) => write!(f, "{pragma}"),
            StmtKind::QuantumGateDefinition(gate) => write!(f, "{gate}"),
            StmtKind::QuantumDecl(decl) => write!(f, "{decl}"),
            StmtKind::Reset(reset_stmt) => write!(f, "{reset_stmt}"),
            StmtKind::Return(return_stmt) => write!(f, "{return_stmt}"),
            StmtKind::Switch(switch_stmt) => write!(f, "{switch_stmt}"),
            StmtKind::WhileLoop(while_loop) => write!(f, "{while_loop}"),
            StmtKind::Err => write!(f, "Err"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CalibrationGrammarStmt {
    pub span: Span,
    pub name: String,
}

impl Display for CalibrationGrammarStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "CalibrationGrammarStmt", self.span)?;
        write_field(f, "name", &self.name)
    }
}

#[derive(Clone, Debug)]
pub struct DefCalStmt {
    pub span: Span,
}

impl Display for DefCalStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DefCalStmt {}", self.span)
    }
}

#[derive(Clone, Debug)]
pub struct IfStmt {
    pub span: Span,
    pub condition: Expr,
    pub if_block: List<Stmt>,
    pub else_block: Option<List<Stmt>>,
}

impl Display for IfStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "IfStmt", self.span)?;
        writeln_field(f, "condition", &self.condition)?;
        writeln_list_field(f, "if_block", &self.if_block)?;
        write_opt_list_field(f, "else_block", self.else_block.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct BarrierStmt {
    pub span: Span,
    pub qubits: List<GateOperand>,
}

impl Display for BarrierStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "BarrierStmt", self.span)?;
        write_list_field(f, "operands", &self.qubits)
    }
}

#[derive(Clone, Debug)]
pub struct ResetStmt {
    pub span: Span,
    pub operand: Box<GateOperand>,
}

impl Display for ResetStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "ResetStmt", self.span)?;
        write_field(f, "operand", &self.operand)
    }
}

/// A sequenced block of statements.
#[derive(Clone, Debug, Default)]
pub struct Block {
    /// The span.
    pub span: Span,
    /// The statements in the block.
    pub stmts: List<Stmt>,
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_header(f, "Block", self.span)?;
        write_indented_list(f, &self.stmts)
    }
}

#[derive(Clone, Debug)]
pub enum Identifier {
    Ident(Box<Ident>),
    IndexedIdent(Box<IndexedIdent>),
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Identifier::Ident(ident) => write!(f, "{ident}"),
            Identifier::IndexedIdent(ident) => write!(f, "{ident}"),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Ident {
    pub span: Span,
    pub name: Rc<str>,
}

impl Default for Ident {
    fn default() -> Self {
        Ident {
            span: Span::default(),
            name: "".into(),
        }
    }
}

impl Display for Ident {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Ident {} \"{}\"", self.span, self.name)
    }
}

impl WithSpan for Ident {
    fn with_span(self, span: Span) -> Self {
        Self { span, ..self }
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
        writeln_header(f, "IndexedIdent", self.span)?;
        writeln_field(f, "name", &self.name)?;
        write_list_field(f, "indices", &self.indices)
    }
}

#[derive(Clone, Debug)]
pub struct ExprStmt {
    pub span: Span,
    pub expr: Expr,
}

impl Display for ExprStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "ExprStmt", self.span)?;
        write_field(f, "expr", &self.expr)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Expr {
    pub span: Span,
    pub kind: Box<ExprKind>,
}

impl WithSpan for Expr {
    fn with_span(self, span: Span) -> Self {
        Self { span, ..self }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Expr {}: {}", self.span, self.kind)
    }
}

#[derive(Clone, Debug)]
pub struct DiscreteSet {
    pub span: Span,
    pub values: List<Expr>,
}

impl Display for DiscreteSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "DiscreteSet", self.span)?;
        write_list_field(f, "values", &self.values)
    }
}

#[derive(Clone, Debug)]
pub struct RangeDefinition {
    pub span: Span,
    pub start: Option<Expr>,
    pub end: Option<Expr>,
    pub step: Option<Expr>,
}

impl Display for RangeDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "RangeDefinition", self.span)?;
        writeln_opt_field(f, "start", self.start.as_ref())?;
        writeln_opt_field(f, "step", self.step.as_ref())?;
        write_opt_field(f, "end", self.end.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct QuantumGateModifier {
    pub span: Span,
    pub kind: GateModifierKind,
}

impl Display for QuantumGateModifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "QuantumGateModifier {}: {}", self.span, self.kind)
    }
}

#[derive(Clone, Debug)]
pub enum GateModifierKind {
    Inv,
    Pow(Expr),
    Ctrl(Option<Expr>),
    NegCtrl(Option<Expr>),
}

impl Display for GateModifierKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            GateModifierKind::Inv => write!(f, "Inv"),
            GateModifierKind::Pow(expr) => write!(f, "Pow {expr}"),
            GateModifierKind::Ctrl(expr) => write!(f, "Ctrl {expr:?}"),
            GateModifierKind::NegCtrl(expr) => write!(f, "NegCtrl {expr:?}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ClassicalArgument {
    pub span: Span,
    pub r#type: ScalarType,
    pub name: Identifier,
    pub access: Option<AccessControl>,
}

impl Display for ClassicalArgument {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(access) = &self.access {
            write!(
                f,
                "ClassicalArgument {}: {}, {}, {}",
                self.span, self.r#type, self.name, access
            )
        } else {
            write!(
                f,
                "ClassicalArgument {}: {}, {}",
                self.span, self.r#type, self.name
            )
        }
    }
}

#[derive(Clone, Debug)]
pub enum ExternParameter {
    Scalar(ScalarType, Span),
    Quantum(Option<Expr>, Span),
    ArrayReference(ArrayReferenceType, Span),
}

impl Display for ExternParameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ExternParameter::Scalar(ty, span) => {
                write!(f, "{span}: {ty}")
            }
            ExternParameter::Quantum(expr, span) => {
                write!(f, "{span}: {expr:?}")
            }
            ExternParameter::ArrayReference(ty, span) => {
                write!(f, "{span}: {ty}")
            }
        }
    }
}

impl Default for ExternParameter {
    fn default() -> Self {
        ExternParameter::Scalar(ScalarType::default(), Span::default())
    }
}

impl WithSpan for ExternParameter {
    fn with_span(self, span: Span) -> Self {
        match self {
            ExternParameter::Scalar(ty, _) => ExternParameter::Scalar(ty, span),
            ExternParameter::Quantum(expr, _) => ExternParameter::Quantum(expr, span),
            ExternParameter::ArrayReference(ty, _) => ExternParameter::ArrayReference(ty, span),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ScalarType {
    pub span: Span,
    pub kind: ScalarTypeKind,
}

impl Display for ScalarType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ScalarType {}: {}", self.span, self.kind)
    }
}

#[derive(Clone, Debug, Default)]
pub enum ScalarTypeKind {
    Bit(BitType),
    Int(IntType),
    UInt(UIntType),
    Float(FloatType),
    Complex(ComplexType),
    Angle(AngleType),
    BoolType,
    Duration,
    Stretch,
    #[default]
    Err,
}

impl Display for ScalarTypeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ScalarTypeKind::Int(int) => write!(f, "{int}"),
            ScalarTypeKind::UInt(uint) => write!(f, "{uint}"),
            ScalarTypeKind::Float(float) => write!(f, "{float}"),
            ScalarTypeKind::Complex(complex) => write!(f, "{complex}"),
            ScalarTypeKind::Angle(angle) => write!(f, "{angle}"),
            ScalarTypeKind::Bit(bit) => write!(f, "{bit}"),
            ScalarTypeKind::BoolType => write!(f, "BoolType"),
            ScalarTypeKind::Duration => write!(f, "Duration"),
            ScalarTypeKind::Stretch => write!(f, "Stretch"),
            ScalarTypeKind::Err => write!(f, "Err"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ArrayBaseTypeKind {
    Int(IntType),
    UInt(UIntType),
    Float(FloatType),
    Complex(ComplexType),
    Angle(AngleType),
    BoolType,
    Duration,
}

impl Display for ArrayBaseTypeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ArrayBaseTypeKind::Int(int) => write!(f, "ArrayBaseTypeKind {int}"),
            ArrayBaseTypeKind::UInt(uint) => write!(f, "ArrayBaseTypeKind {uint}"),
            ArrayBaseTypeKind::Float(float) => write!(f, "ArrayBaseTypeKind {float}"),
            ArrayBaseTypeKind::Complex(complex) => write!(f, "ArrayBaseTypeKind {complex}"),
            ArrayBaseTypeKind::Angle(angle) => write!(f, "ArrayBaseTypeKind {angle}"),
            ArrayBaseTypeKind::Duration => write!(f, "ArrayBaseTypeKind DurationType"),
            ArrayBaseTypeKind::BoolType => write!(f, "ArrayBaseTypeKind BoolType"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct IntType {
    pub span: Span,
    pub size: Option<Expr>,
}

impl Display for IntType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "IntType", self.span)?;
        write_opt_field(f, "size", self.size.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct UIntType {
    pub span: Span,
    pub size: Option<Expr>,
}

impl Display for UIntType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "UIntType", self.span)?;
        write_opt_field(f, "size", self.size.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct FloatType {
    pub span: Span,
    pub size: Option<Expr>,
}

impl Display for FloatType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "FloatType", self.span)?;
        write_opt_field(f, "size", self.size.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct ComplexType {
    pub span: Span,
    pub base_size: Option<FloatType>,
}

impl Display for ComplexType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "ComplexType", self.span)?;
        write_opt_field(f, "base_size", self.base_size.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct AngleType {
    pub span: Span,
    pub size: Option<Expr>,
}

impl Display for AngleType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "AngleType", self.span)?;
        write_opt_field(f, "size", self.size.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct BitType {
    pub span: Span,
    pub size: Option<Expr>,
}

impl Display for BitType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "BitType", self.span)?;
        write_opt_field(f, "size", self.size.as_ref())
    }
}

#[derive(Clone, Debug)]
pub enum TypeDef {
    Scalar(ScalarType),
    Array(ArrayType),
    ArrayReference(ArrayReferenceType),
}

impl TypeDef {
    pub fn span(&self) -> Span {
        match self {
            TypeDef::Scalar(ident) => ident.span,
            TypeDef::Array(array) => array.span,
            TypeDef::ArrayReference(array) => array.span,
        }
    }
}

impl Display for TypeDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TypeDef::Scalar(scalar) => write!(f, "{scalar}"),
            TypeDef::Array(array) => write!(f, "{array}"),
            TypeDef::ArrayReference(array) => write!(f, "{array}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ArrayType {
    pub span: Span,
    pub base_type: ArrayBaseTypeKind,
    pub dimensions: List<Expr>,
}

impl Display for ArrayType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "ArrayType", self.span)?;
        writeln_field(f, "base_type", &self.base_type)?;
        write_list_field(f, "dimensions", &self.dimensions)
    }
}

#[derive(Clone, Debug)]
pub struct ArrayReferenceType {
    pub span: Span,
    pub mutability: AccessControl,
    pub base_type: ArrayBaseTypeKind,
    pub dimensions: List<Expr>,
}

impl Display for ArrayReferenceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "ArrayReferenceType", self.span)?;
        writeln_field(f, "mutability", &self.mutability)?;
        writeln_field(f, "base_type", &self.base_type)?;
        writeln_list_field(f, "dimensions", &self.dimensions)
    }
}

#[derive(Clone, Debug)]
pub enum AccessControl {
    ReadOnly,
    Mutable,
}

impl Display for AccessControl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AccessControl::ReadOnly => write!(f, "ReadOnly"),
            AccessControl::Mutable => write!(f, "Mutable"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct QuantumArgument {
    pub span: Span,
    pub expr: Option<Expr>,
}

impl Display for QuantumArgument {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "QuantumArgument", self.span)?;
        write_opt_field(f, "expr", self.expr.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct Pragma {
    pub span: Span,
    pub identifier: Rc<str>,
    pub value: Option<Rc<str>>,
}

impl Display for Pragma {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let identifier = format!("\"{}\"", self.identifier);
        let value = self.value.as_ref().map(|val| format!("\"{val}\""));
        writeln_header(f, "Pragma", self.span)?;
        writeln_field(f, "identifier", &identifier)?;
        write_opt_field(f, "value", value.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct IncludeStmt {
    pub span: Span,
    pub filename: String,
}

impl Display for IncludeStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "IncludeStmt", self.span)?;
        write_field(f, "filename", &self.filename)
    }
}

#[derive(Clone, Debug)]
pub struct QubitDeclaration {
    pub span: Span,
    pub qubit: Box<Ident>,
    pub size: Option<Expr>,
}

impl Display for QubitDeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "QubitDeclaration", self.span)?;
        writeln_field(f, "ident", &self.qubit)?;
        write_opt_field(f, "size", self.size.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct QuantumGateDefinition {
    pub span: Span,
    pub ident: Box<Ident>,
    pub params: List<Ident>,
    pub qubits: List<Ident>,
    pub body: Box<Block>,
}

impl Display for QuantumGateDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "Gate", self.span)?;
        writeln_field(f, "ident", &self.ident)?;
        writeln_list_field(f, "parameters", &self.params)?;
        writeln_list_field(f, "qubits", &self.qubits)?;
        write_field(f, "body", &self.body)
    }
}

#[derive(Clone, Debug)]
pub struct ExternDecl {
    pub span: Span,
    pub ident: Box<Ident>,
    pub params: List<ExternParameter>,
    pub return_type: Option<ScalarType>,
}

impl Display for ExternDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "ExternDecl", self.span)?;
        writeln_field(f, "ident", &self.ident)?;
        writeln_list_field(f, "parameters", &self.params)?;
        write_opt_field(f, "return_type", self.return_type.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct GateCall {
    pub span: Span,
    pub modifiers: List<QuantumGateModifier>,
    pub name: Ident,
    pub args: List<Expr>,
    pub qubits: List<GateOperand>,
    pub duration: Option<Expr>,
}

impl Display for GateCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "GateCall", self.span)?;
        writeln_list_field(f, "modifiers", &self.modifiers)?;
        writeln_field(f, "name", &self.name)?;
        writeln_list_field(f, "args", &self.args)?;
        writeln_opt_field(f, "duration", self.duration.as_ref())?;
        write_list_field(f, "qubits", &self.qubits)
    }
}

#[derive(Clone, Debug)]
pub struct GPhase {
    pub span: Span,
    pub modifiers: List<QuantumGateModifier>,
    pub args: List<Expr>,
    pub qubits: List<GateOperand>,
    pub duration: Option<Expr>,
}

impl Display for GPhase {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "GPhase", self.span)?;
        writeln_list_field(f, "modifiers", &self.modifiers)?;
        writeln_list_field(f, "args", &self.args)?;
        writeln_opt_field(f, "duration", self.duration.as_ref())?;
        write_list_field(f, "qubits", &self.qubits)
    }
}

#[derive(Clone, Debug)]
pub struct DelayStmt {
    pub span: Span,
    pub duration: Expr,
    pub qubits: List<GateOperand>,
}

impl Display for DelayStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "DelayStmt", self.span)?;
        writeln_field(f, "duration", &self.duration)?;
        write_list_field(f, "qubits", &self.qubits)
    }
}

#[derive(Clone, Debug)]
pub struct BoxStmt {
    pub span: Span,
    pub duration: Option<Expr>,
    pub body: List<Stmt>,
}

impl Display for BoxStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "BoxStmt", self.span)?;
        writeln_opt_field(f, "duration", self.duration.as_ref())?;
        write_list_field(f, "body", &self.body)
    }
}

#[derive(Clone, Debug)]
pub struct MeasureStmt {
    pub span: Span,
    pub measurement: MeasureExpr,
    pub target: Option<Box<IndexedIdent>>,
}

impl Display for MeasureStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "MeasureStmt", self.span)?;
        writeln_field(f, "measurement", &self.measurement)?;
        write_opt_field(f, "target", self.target.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct ClassicalDeclarationStmt {
    pub span: Span,
    pub r#type: TypeDef,
    pub identifier: Box<Ident>,
    pub init_expr: Option<Box<ValueExpression>>,
}

impl Display for ClassicalDeclarationStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "ClassicalDeclarationStmt", self.span)?;
        writeln_field(f, "type", &self.r#type)?;
        writeln_field(f, "ident", &self.identifier)?;
        write_opt_field(f, "init_expr", self.init_expr.as_ref())
    }
}

#[derive(Clone, Debug)]
pub enum ValueExpression {
    Expr(Expr),
    Measurement(MeasureExpr),
}

impl Display for ValueExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ValueExpression::Expr(expr) => write!(f, "{expr}"),
            ValueExpression::Measurement(measure) => write!(f, "{measure}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct IODeclaration {
    pub span: Span,
    pub io_identifier: IOKeyword,
    pub r#type: TypeDef,
    pub ident: Box<Ident>,
}

impl Display for IODeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "IODeclaration", self.span)?;
        writeln_field(f, "io_keyword", &self.io_identifier)?;
        writeln_field(f, "type", &self.r#type)?;
        write_field(f, "ident", &self.ident)
    }
}

#[derive(Clone, Debug)]
pub struct ConstantDeclStmt {
    pub span: Span,
    pub r#type: TypeDef,
    pub identifier: Box<Ident>,
    pub init_expr: Expr,
}

impl Display for ConstantDeclStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "ConstantDeclStmt", self.span)?;
        writeln_field(f, "type", &self.r#type)?;
        writeln_field(f, "ident", &self.identifier)?;
        write_field(f, "init_expr", &self.init_expr)
    }
}

#[derive(Clone, Debug)]
pub struct CalibrationGrammarDeclaration {
    span: Span,
    name: String,
}

impl Display for CalibrationGrammarDeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "CalibrationGrammarDeclaration", self.span)?;
        write_field(f, "name", &self.name)
    }
}

#[derive(Clone, Debug)]
pub struct CalibrationStmt {
    pub span: Span,
}

impl Display for CalibrationStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "CalibrationStmt {}", self.span)
    }
}

#[derive(Clone, Debug)]
pub enum TypedParameter {
    Scalar(ScalarType, Box<Ident>, Span),
    Quantum(Option<Expr>, Box<Ident>, Span),
    ArrayReference(ArrayReferenceType, Box<Ident>, Span),
}

impl WithSpan for TypedParameter {
    fn with_span(self, span: Span) -> Self {
        match self {
            TypedParameter::Scalar(scalar, ident, _) => TypedParameter::Scalar(scalar, ident, span),
            TypedParameter::Quantum(expr, ident, _) => TypedParameter::Quantum(expr, ident, span),
            TypedParameter::ArrayReference(array, ident, _) => {
                TypedParameter::ArrayReference(array, ident, span)
            }
        }
    }
}

impl Display for TypedParameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TypedParameter::Scalar(scalar, ident, span) => {
                writeln_header(f, "TypedParameter::Scalar", *span)?;
                writeln_field(f, "type", scalar)?;
                write_field(f, "ident", ident)
            }
            TypedParameter::Quantum(expr, ident, span) => {
                writeln_header(f, "TypedParameter::Quantum", *span)?;
                writeln_opt_field(f, "size", expr.as_ref())?;
                write_field(f, "ident", ident)
            }
            TypedParameter::ArrayReference(array, ident, span) => {
                writeln_header(f, "TypedParameter::ArrayReference", *span)?;
                writeln_field(f, "type", array)?;
                write_field(f, "ident", ident)
            }
        }
    }
}

impl Default for TypedParameter {
    fn default() -> Self {
        TypedParameter::Scalar(ScalarType::default(), Box::default(), Span::default())
    }
}

#[derive(Clone, Debug)]
pub struct DefStmt {
    pub span: Span,
    pub name: Box<Ident>,
    pub params: List<TypedParameter>,
    pub body: Box<Block>,
    pub return_type: Option<ScalarType>,
}

impl Display for DefStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "DefStmt", self.span)?;
        writeln_field(f, "ident", &self.name)?;
        writeln_list_field(f, "parameters", &self.params)?;
        writeln_opt_field(f, "return_type", self.return_type.as_ref())?;
        write_field(f, "body", &self.body)
    }
}

#[derive(Clone, Debug)]
pub enum Operand {
    Classical(ClassicalArgument),
    Quantum(QuantumArgument),
}

impl Display for Operand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Classical(arg) => write!(f, "{arg}"),
            Operand::Quantum(arg) => write!(f, "{arg}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ReturnStmt {
    pub span: Span,
    pub expr: Option<Box<ValueExpression>>,
}

impl Display for ReturnStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "ReturnStmt", self.span)?;
        write_opt_field(f, "expr", self.expr.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct WhileLoop {
    pub span: Span,
    pub while_condition: Expr,
    pub block: List<Stmt>,
}

impl Display for WhileLoop {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "WhileLoop", self.span)?;
        writeln_field(f, "condition", &self.while_condition)?;
        write_list_field(f, "block", &self.block)
    }
}

#[derive(Clone, Debug)]
pub struct ForStmt {
    pub span: Span,
    pub r#type: ScalarType,
    pub identifier: Identifier,
    pub set_declaration: Box<EnumerableSet>,
    pub block: List<Stmt>,
}

impl Display for ForStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "ForStmt", self.span)?;
        writeln_field(f, "variable_type", &self.r#type)?;
        writeln_field(f, "variable_name", &self.identifier)?;
        writeln_field(f, "iterable", &self.set_declaration)?;
        write_list_field(f, "block", &self.block)
    }
}

#[derive(Clone, Debug)]
pub enum EnumerableSet {
    DiscreteSet(DiscreteSet),
    RangeDefinition(RangeDefinition),
    Expr(Expr),
}

impl Display for EnumerableSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EnumerableSet::DiscreteSet(set) => write!(f, "{set}"),
            EnumerableSet::RangeDefinition(range) => write!(f, "{range}"),
            EnumerableSet::Expr(expr) => write!(f, "{expr}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SwitchStmt {
    pub span: Span,
    pub target: Expr,
    pub cases: List<SwitchCase>,
    /// Note that `None` is quite different to `[]` in this case; the latter is
    /// an explicitly empty body, whereas the absence of a default might mean
    /// that the switch is inexhaustive, and a linter might want to complain.
    pub default: Option<Block>,
}

impl Display for SwitchStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "SwitchStmt", self.span)?;
        writeln_field(f, "target", &self.target)?;
        writeln_list_field(f, "cases", &self.cases)?;
        write_opt_field(f, "default_case", self.default.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct SwitchCase {
    pub span: Span,
    pub labels: List<Expr>,
    pub block: Block,
}

impl Display for SwitchCase {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "SwitchCase", self.span)?;
        writeln_list_field(f, "labels", &self.labels)?;
        write_field(f, "block", &self.block)
    }
}

#[derive(Clone, Debug, Default)]
pub enum ExprKind {
    Assign(AssignExpr),
    AssignOp(AssignOpExpr),
    /// An expression with invalid syntax that can't be parsed.
    #[default]
    Err,
    Ident(Ident),
    UnaryOp(UnaryOpExpr),
    BinaryOp(BinaryOpExpr),
    Lit(Lit),
    FunctionCall(FunctionCall),
    Cast(Cast),
    IndexExpr(IndexExpr),
    Paren(Expr),
}

impl Display for ExprKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ExprKind::Err => write!(f, "Err"),
            ExprKind::Ident(id) => write!(f, "{id}"),
            ExprKind::UnaryOp(expr) => write!(f, "{expr}"),
            ExprKind::BinaryOp(expr) => write!(f, "{expr}"),
            ExprKind::Lit(lit) => write!(f, "{lit}"),
            ExprKind::FunctionCall(call) => write!(f, "{call}"),
            ExprKind::Cast(expr) => write!(f, "{expr}"),
            ExprKind::IndexExpr(expr) => write!(f, "{expr}"),
            ExprKind::Assign(expr) => write!(f, "{expr}"),
            ExprKind::AssignOp(expr) => write!(f, "{expr}"),
            ExprKind::Paren(expr) => write!(f, "Paren {expr}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AssignExpr {
    pub lhs: Expr,
    pub rhs: Expr,
}

impl Display for AssignExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "AssignExpr:")?;
        writeln_field(f, "lhs", &self.lhs)?;
        write_field(f, "rhs", &self.rhs)
    }
}

#[derive(Clone, Debug)]
pub struct AssignOpExpr {
    pub op: BinOp,
    pub lhs: Expr,
    pub rhs: Expr,
}

impl Display for AssignOpExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "AssignOpExpr:")?;
        writeln_field(f, "op", &self.op)?;
        writeln_field(f, "lhs", &self.lhs)?;
        write_field(f, "rhs", &self.rhs)
    }
}

#[derive(Clone, Debug)]
pub struct UnaryOpExpr {
    pub op: UnaryOp,
    pub expr: Expr,
}

impl Display for UnaryOpExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "UnaryOpExpr:")?;
        writeln_field(f, "op", &self.op)?;
        write_field(f, "expr", &self.expr)
    }
}

#[derive(Clone, Debug)]
pub struct BinaryOpExpr {
    pub op: BinOp,
    pub lhs: Expr,
    pub rhs: Expr,
}

impl Display for BinaryOpExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "BinaryOpExpr:")?;
        writeln_field(f, "op", &self.op)?;
        writeln_field(f, "lhs", &self.lhs)?;
        write_field(f, "rhs", &self.rhs)
    }
}

#[derive(Clone, Debug)]
pub struct FunctionCall {
    pub span: Span,
    pub name: Ident,
    pub args: List<Expr>,
}

impl Display for FunctionCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "FunctionCall", self.span)?;
        writeln_field(f, "name", &self.name)?;
        write_list_field(f, "args", &self.args)
    }
}

#[derive(Clone, Debug)]
pub struct Cast {
    pub span: Span,
    pub r#type: TypeDef,
    pub arg: Expr,
}

impl Display for Cast {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "Cast", self.span)?;
        writeln_field(f, "type", &self.r#type)?;
        write_field(f, "arg", &self.arg)
    }
}

#[derive(Clone, Debug)]
pub struct IndexExpr {
    pub span: Span,
    pub collection: Expr,
    pub index: IndexElement,
}

impl Display for IndexExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln_header(f, "IndexExpr", self.span)?;
        writeln_field(f, "collection", &self.collection)?;
        write_field(f, "index", &self.index)
    }
}

#[derive(Clone, Debug)]
pub struct Lit {
    pub span: Span,
    pub kind: LiteralKind,
}

impl Display for Lit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Lit: {}", self.kind)
    }
}

#[derive(Clone, Debug)]
pub enum LiteralKind {
    Array(List<Expr>),
    Bitstring { value: BigInt, width: usize },
    Bool(bool),
    Duration { value: f64, unit: TimeUnit },
    Float(f64),
    Imaginary(f64),
    Int(i64),
    BigInt(BigInt),
    String(Rc<str>),
}

impl Display for LiteralKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LiteralKind::Array(exprs) => write_list_field(f, "Array", exprs),
            LiteralKind::Bitstring { value, width } => {
                write!(f, "Bitstring(\"{:0>width$}\")", value.to_str_radix(2))
            }
            LiteralKind::Bool(b) => write!(f, "Bool({b:?})"),
            LiteralKind::Duration { value, unit } => {
                write!(f, "Duration({value:?}, {unit:?})")
            }
            LiteralKind::Float(value) => write!(f, "Float({value:?})"),
            LiteralKind::Imaginary(value) => write!(f, "Imaginary({value:?})"),
            LiteralKind::Int(i) => write!(f, "Int({i:?})"),
            LiteralKind::BigInt(i) => write!(f, "BigInt({i:?})"),
            LiteralKind::String(s) => write!(f, "String({s:?})"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Version {
    pub major: u32,
    pub minor: Option<u32>,
    pub span: Span,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.minor {
            Some(minor) => write!(f, "{}.{}", self.major, minor),
            None => write!(f, "{}", self.major),
        }
    }
}

#[derive(Clone, Debug)]
pub enum IndexElement {
    DiscreteSet(DiscreteSet),
    IndexSet(List<IndexSetItem>),
}

impl Display for IndexElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            IndexElement::DiscreteSet(set) => write!(f, "{set}"),
            IndexElement::IndexSet(items) => {
                write!(f, "IndexSet:")?;
                write_indented_list(f, items)
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum IndexSetItem {
    RangeDefinition(RangeDefinition),
    Expr(Expr),
    #[default]
    Err,
}

/// This is needed to able to use `IndexSetItem` in the `seq` combinator.
impl WithSpan for IndexSetItem {
    fn with_span(self, _span: Span) -> Self {
        self
    }
}

impl Display for IndexSetItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            IndexSetItem::RangeDefinition(range) => write!(f, "{range}"),
            IndexSetItem::Expr(expr) => write!(f, "{expr}"),
            IndexSetItem::Err => write!(f, "Err"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum IOKeyword {
    Input,
    Output,
}

impl Display for IOKeyword {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            IOKeyword::Input => write!(f, "input"),
            IOKeyword::Output => write!(f, "output"),
        }
    }
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

impl Display for TimeUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TimeUnit::Dt => write!(f, "dt"),
            TimeUnit::Ns => write!(f, "ns"),
            TimeUnit::Us => write!(f, "us"),
            TimeUnit::Ms => write!(f, "ms"),
            TimeUnit::S => write!(f, "s"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BreakStmt {
    pub span: Span,
}

impl Display for BreakStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "BreakStmt {}", self.span)
    }
}

#[derive(Clone, Debug)]
pub struct ContinueStmt {
    pub span: Span,
}

impl Display for ContinueStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ContinueStmt {}", self.span)
    }
}

#[derive(Clone, Debug)]
pub struct EndStmt {
    pub span: Span,
}

impl Display for EndStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "EndStmt {}", self.span)
    }
}
