// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// while we work through the conversion, allow dead code to avoid warnings
#![allow(dead_code)]

use indenter::{indented, Indented};
use num_bigint::BigInt;
use qsc_data_structures::span::{Span, WithSpan};
use std::{
    fmt::{self, Display, Formatter, Write},
    hash::Hash,
    rc::Rc,
};

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

// TODO: profile this with iai-callgrind in a large OpenQASM3
// sample to verify that is actually faster than using Vec<T>.
/// An alternative to `Vec<T>` that uses less stack space.
type List<T> = Box<[Box<T>]>;

#[derive(Clone, Debug)]
pub struct Program {
    pub span: Span,
    pub statements: List<Stmt>,
    pub version: Option<Version>,
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "Program {}:", self.span)?;
        indent = set_indentation(indent, 1);
        if let Some(version) = &self.version {
            write!(indent, "\nVersion {version}")?;
        }
        for stmt in &self.statements {
            write!(indent, "\n{stmt}")?;
        }
        Ok(())
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
        let mut indent = set_indentation(indented(f), 0);
        for annotation in &self.annotations {
            write!(indent, "\n{annotation}")?;
        }
        write!(indent, "Stmt {}", self.span)?;
        write!(indent, "\n{}", self.kind)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Annotation {
    pub span: Span,
    pub name: Box<PathKind>,
    pub value: Option<Rc<str>>,
}
impl Display for Annotation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(value) = &self.value {
            write!(f, "Annotation {}: {}, {}", self.span, self.name, value)
        } else {
            write!(f, "Annotation {}: {}", self.span, self.name)
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
        write!(f, "MeasureExpr {}: {}", self.span, self.operand)
    }
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
#[derive(Clone, Debug)]
pub enum UnOp {
    /// Negation: `-`.
    Neg,
    /// Bitwise NOT: `~`.
    NotB,
    /// Logical NOT: `!`.
    NotL,
}

impl Display for UnOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UnOp::Neg => write!(f, "Neg"),
            UnOp::NotB => write!(f, "NotB"),
            UnOp::NotL => write!(f, "NotL"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum GateOperand {
    Ident(Box<Identifier>),
    HardwareQubit(Box<HardwareQubit>),
}

impl Display for GateOperand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            GateOperand::Ident(ident) => write!(f, "GateOperand {ident}"),
            GateOperand::HardwareQubit(qubit) => write!(f, "GateOperand {qubit}"),
        }
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
pub struct Alias {
    pub ident: Box<Identifier>,
    pub expr: Box<List<Expr>>,
    pub span: Span,
}

impl Display for Alias {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "Alias {}: {}", self.span, self.ident)?;
        indent = set_indentation(indent, 1);
        for expr in &*self.expr {
            write!(indent, "\n{expr}")?;
        }
        Ok(())
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
    Alias(Alias),
    Assign(Assign),
    AssignOp(AssignOp),
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
    Pragma(Pragma),
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

impl Display for StmtKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "StmtKind: ")?;
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
            StmtKind::DelayStmt(delay) => write!(f, "{delay}"),
            StmtKind::Empty => write!(f, "Empty"),
            StmtKind::ExprStmt(expr) => write!(f, "{expr}"),
            StmtKind::ExternDecl(decl) => write!(f, "{decl}"),
            StmtKind::For(for_stmt) => write!(f, "{for_stmt}"),
            StmtKind::If(if_stmt) => write!(f, "{if_stmt}"),
            StmtKind::Include(include) => write!(f, "{include}"),
            StmtKind::IODeclaration(io) => write!(f, "{io}"),
            StmtKind::Measure(measure) => write!(f, "{measure}"),
            StmtKind::Pragma(pragma) => write!(f, "{pragma}"),
            StmtKind::QuantumGateDefinition(gate) => write!(f, "{gate}"),
            StmtKind::QuantumDecl(decl) => write!(f, "{decl}"),
            StmtKind::Quantum(quantum_stmt) => write!(f, "{quantum_stmt}"),
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
        write!(f, "CalibrationGrammarStmt {}: {}", self.span, self.name)
    }
}

#[derive(Clone, Debug)]
pub struct DefCalStmt {}

impl Display for DefCalStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DefCalStmt")
    }
}

#[derive(Clone, Debug)]
pub struct IfStmt {
    pub span: Span,
    pub condition: ExprStmt,
    pub if_block: List<Stmt>,
    pub else_block: Option<List<Stmt>>,
}

impl Display for IfStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "IfStmt {}: {}", self.span, self.condition)?;
        for stmt in &self.if_block {
            write!(indent, "\n{stmt}")?;
        }
        if let Some(else_block) = &self.else_block {
            write!(indent, "\nElse:")?;
            for stmt in else_block {
                write!(indent, "\n{stmt}")?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct DelayStmt {
    pub span: Span,
    pub duration: ExprStmt,
}

impl Display for DelayStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DelayStmt {}: {}", self.span, self.duration)
    }
}

#[derive(Clone, Debug)]
pub struct BarrierStmt {
    pub span: Span,
    pub qubits: List<GateOperand>,
}

impl Display for BarrierStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "Barrier {}: [", self.span)?;
        for qubit in &self.qubits {
            write!(indent, "\n{qubit}")?;
        }
        write!(indent, "]")
    }
}

#[derive(Clone, Debug)]
pub struct ResetStmt {
    pub span: Span,
    pub operand: Box<GateOperand>,
}

impl Display for ResetStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ResetStmt {}: {}", self.span, self.operand)
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
        if self.stmts.is_empty() {
            write!(f, "Block {}: <empty>", self.span)?;
        } else {
            let mut indent = set_indentation(indented(f), 0);
            write!(indent, "Block {}:", self.span)?;
            indent = set_indentation(indent, 1);
            for s in &self.stmts {
                write!(indent, "\n{s}")?;
            }
        }
        Ok(())
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
        write!(f, "IndexedIdent {}: {}[", self.span, self.name)?;

        for index in &self.indices {
            write!(f, "\n{index}")?;
        }
        write!(f, "]")
    }
}

#[derive(Clone, Debug)]
pub struct AliasStmt {
    pub span: Span,
    pub kind: Box<Expr>,
}

impl Display for AliasStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "AliasStmt {}: {}", self.span, self.kind)
    }
}

#[derive(Clone, Debug)]
pub struct ExprStmt {
    pub span: Span,
    pub expr: Box<Expr>,
}

impl Display for ExprStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ExprStmt {}: {}", self.span, self.expr)
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
    pub values: List<ExprStmt>,
}

impl Display for DiscreteSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "DiscreteSet {}:", self.span)?;
        indent = set_indentation(indent, 1);
        for value in &self.values {
            write!(indent, "\n{value}")?;
        }
        Ok(())
    }
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

impl Display for QuantumGateModifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "QuantumGateModifier {}: {}", self.span, self.qubit)
    }
}

#[derive(Clone, Debug)]
pub struct QuantumMeasurement {
    pub span: Span,
    pub qubit: Box<Identifier>,
}

impl Display for QuantumMeasurement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "QuantumMeasurement {}: {}", self.span, self.qubit)
    }
}

#[derive(Clone, Debug)]
pub struct ClassicalArgument {
    pub span: Span,
    pub r#type: ClassicalType,
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
pub struct ExternArgument {
    pub span: Span,
    pub r#type: ClassicalType,
    pub access: Option<AccessControl>,
}

impl Display for ExternArgument {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(access) = &self.access {
            write!(
                f,
                "ExternArgument {}: {}, {}",
                self.span, self.r#type, access
            )
        } else {
            write!(f, "ExternArgument {}: {}", self.span, self.r#type)
        }
    }
}

#[derive(Clone, Debug)]
pub struct ClassicalType {
    pub span: Span,
    pub kind: ClassicalTypeKind,
}

impl Display for ClassicalType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ClassicalType {}: {}", self.span, self.kind)
    }
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

impl Display for ClassicalTypeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ClassicalTypeKind::Int(int) => write!(f, "ClassicalTypeKind {int}"),
            ClassicalTypeKind::UInt(uint) => write!(f, "ClassicalTypeKind {uint}"),
            ClassicalTypeKind::Float(float) => write!(f, "ClassicalTypeKind {float}"),
            ClassicalTypeKind::Complex(complex) => write!(f, "ClassicalTypeKind {complex}"),
            ClassicalTypeKind::Angle(angle) => write!(f, "ClassicalTypeKind {angle}"),
            ClassicalTypeKind::Bit(bit) => write!(f, "ClassicalTypeKind {bit}"),
            ClassicalTypeKind::BoolType => write!(f, "ClassicalTypeKind BoolType"),
            ClassicalTypeKind::Array(array) => write!(f, "ClassicalTypeKind {array}"),
            ClassicalTypeKind::ArrayReference(array) => write!(f, "ClassicalTypeKind {array}"),
            ClassicalTypeKind::Duration => write!(f, "ClassicalTypeKind Duration"),
            ClassicalTypeKind::Stretch => write!(f, "ClassicalTypeKind Stretch"),
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
    Bit(BitType),
    BoolType,
}

impl Display for ArrayBaseTypeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ArrayBaseTypeKind::Int(int) => write!(f, "ArrayBaseTypeKind {int}"),
            ArrayBaseTypeKind::UInt(uint) => write!(f, "ArrayBaseTypeKind {uint}"),
            ArrayBaseTypeKind::Float(float) => write!(f, "ArrayBaseTypeKind {float}"),
            ArrayBaseTypeKind::Complex(complex) => write!(f, "ArrayBaseTypeKind {complex}"),
            ArrayBaseTypeKind::Angle(angle) => write!(f, "ArrayBaseTypeKind {angle}"),
            ArrayBaseTypeKind::Bit(bit) => write!(f, "ArrayBaseTypeKind {bit}"),
            ArrayBaseTypeKind::BoolType => write!(f, "ArrayBaseTypeKind BoolType"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct IntType {
    pub size: Option<ExprStmt>,
    pub span: Span,
}

impl Display for IntType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(size) = &self.size {
            write!(f, "IntType {}: {}", self.span, size)
        } else {
            write!(f, "IntType")
        }
    }
}

#[derive(Clone, Debug)]
pub struct UIntType {
    pub size: Option<ExprStmt>,
    pub span: Span,
}

impl Display for UIntType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(size) = &self.size {
            write!(f, "UIntType {}: {}", self.span, size)
        } else {
            write!(f, "UIntType")
        }
    }
}

#[derive(Clone, Debug)]
pub struct FloatType {
    pub size: Option<ExprStmt>,
    pub span: Span,
}

impl Display for FloatType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(size) = &self.size {
            write!(f, "FloatType {}: {}", self.span, size)
        } else {
            write!(f, "FloatType")
        }
    }
}

#[derive(Clone, Debug)]
pub struct ComplexType {
    pub base_size: Option<FloatType>,
    pub span: Span,
}

impl Display for ComplexType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(size) = &self.base_size {
            write!(f, "ComplexType {}: {}", self.span, size)
        } else {
            write!(f, "ComplexType")
        }
    }
}

#[derive(Clone, Debug)]
pub struct AngleType {
    pub size: Option<ExprStmt>,
    pub span: Span,
}

impl Display for AngleType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(size) = &self.size {
            write!(f, "AngleType {}: {}", self.span, size)
        } else {
            write!(f, "AngleType")
        }
    }
}

#[derive(Clone, Debug)]
pub struct BitType {
    pub size: Option<ExprStmt>,
    pub span: Span,
}

impl Display for BitType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(size) = &self.size {
            write!(f, "BitType {}: {}", self.span, size)
        } else {
            write!(f, "BitType")
        }
    }
}

#[derive(Clone, Debug)]
pub struct ArrayType {
    pub span: Span,
    pub base_type: ArrayBaseTypeKind,
    pub dimensions: List<ExprStmt>,
}

impl Display for ArrayType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "ArrayType {}: {}", self.span, self.base_type)?;
        for dimension in &self.dimensions {
            write!(indent, "\n{dimension}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ArrayReferenceType {
    pub span: Span,
    pub base_type: ArrayBaseTypeKind,
    pub dimensions: List<ExprStmt>,
}

impl Display for ArrayReferenceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(
            indent,
            "ArrayReferenceType {}: {}",
            self.span, self.base_type
        )?;
        for dimension in &self.dimensions {
            write!(indent, "\n{dimension}")?;
        }
        Ok(())
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
    pub expr: Option<ExprStmt>,
}

impl Display for QuantumArgument {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "QuantumArgument {}: {:?}", self.span, self.expr)
    }
}

#[derive(Clone, Debug)]
pub struct Pragma {
    pub span: Span,
    pub name: Box<PathKind>,
    pub value: Option<Rc<str>>,
}

impl Display for Pragma {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(value) = &self.value {
            write!(f, "Pragma {}: {}, {}", self.span, self.name, value)
        } else {
            write!(f, "Pragma {}: {}", self.span, self.name)
        }
    }
}

#[derive(Clone, Debug)]
pub struct CompoundStmt {
    pub span: Span,
    pub statements: List<Stmt>,
}

impl Display for CompoundStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "CompoundStmt {}:", self.span)?;
        for stmt in &self.statements {
            write!(indent, "\n{stmt}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct IncludeStmt {
    pub span: Span,
    pub filename: String,
}

impl Display for IncludeStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "IncludeStmt {}: {}", self.span, self.filename)
    }
}

#[derive(Clone, Debug)]
pub struct QubitDeclaration {
    pub span: Span,
    pub qubit: Ident,
    pub size: Option<ExprStmt>,
}

impl Display for QubitDeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(size) = &self.size {
            write!(
                f,
                "QubitDeclaration {}: {}, {}",
                self.span, self.qubit, size
            )
        } else {
            write!(f, "QubitDeclaration {}: {}", self.span, self.qubit)
        }
    }
}

#[derive(Clone, Debug)]
pub struct QuantumGateDefinition {
    pub span: Span,
    pub name: Identifier,
    pub arguments: Vec<Identifier>,
    pub qubits: Vec<Identifier>,
    pub body: Vec<Stmt>,
}

impl Display for QuantumGateDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "QuantumGateDefinition {}: {}", self.span, self.name)?;
        for arg in &self.arguments {
            write!(indent, "\n{arg}")?;
        }
        for qubit in &self.qubits {
            write!(indent, "\n{qubit}")?;
        }
        for stmt in &self.body {
            write!(indent, "\n{stmt}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ExternDecl {
    pub span: Span,
    pub name: Identifier,
    pub arguments: List<ExternArgument>,
    pub return_type: Option<ClassicalType>,
}

impl Display for ExternDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "ExternDecl {}: {}", self.span, self.name)?;
        for arg in &self.arguments {
            write!(indent, "\n{arg}")?;
        }
        if let Some(return_type) = &self.return_type {
            write!(indent, "\n{return_type}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct QuantumStmt {
    pub span: Span,
    pub kind: QuantumStmtKind,
}

impl Display for QuantumStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "QuantumStmt {}: {}", self.span, self.kind)
    }
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

impl Display for QuantumStmtKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            QuantumStmtKind::Gate(gate) => write!(f, "QuantumStmtKind {gate}"),
            QuantumStmtKind::Phase(phase) => write!(f, "QuantumStmtKind {phase}"),
            QuantumStmtKind::Barrier(barrier) => write!(f, "QuantumStmtKind {barrier:?}"),
            QuantumStmtKind::Reset(reset) => write!(f, "QuantumStmtKind {reset:?}"),
            QuantumStmtKind::DelayInstruction(delay) => write!(f, "QuantumStmtKind {delay:?}"),
            QuantumStmtKind::Box(box_stmt) => write!(f, "QuantumStmtKind {box_stmt:?}"),
        }
    }
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

impl Display for QuantumGate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "QuantumGate {}: {}", self.span, self.name)?;
        for arg in &self.args {
            write!(indent, "\n{arg}")?;
        }
        for qubit in &self.qubits {
            write!(indent, "\n{qubit}")?;
        }
        if let Some(duration) = &self.duration {
            write!(indent, "\n{duration}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct QuantumPhase {
    pub span: Span,
    pub modifiers: List<QuantumGateModifier>,
    pub arg: ExprStmt,
    pub qubits: List<Box<Identifier>>,
}

impl Display for QuantumPhase {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "QuantumPhase {}: {}", self.span, self.arg)?;
        for qubit in &self.qubits {
            write!(indent, "\n{qubit}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct DelayInstruction {
    span: Span,
    duration: ExprStmt,
    qubits: List<Box<Identifier>>,
}

impl Display for DelayInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "DelayInstruction {}: {}", self.span, self.duration)?;
        for qubit in &self.qubits {
            write!(indent, "\n{qubit}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct BoxStmt {
    pub span: Span,
    pub duration: Option<ExprStmt>,
    pub body: List<QuantumStmt>,
}

impl Display for BoxStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        if let Some(duration) = &self.duration {
            write!(indent, "BoxStmt {}: {}", self.span, duration)?;
        } else {
            write!(indent, "BoxStmt {}: <no duration>", self.span)?;
        }
        for stmt in &self.body {
            write!(indent, "\n{stmt}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct MeasureStmt {
    pub span: Span,
    pub measure: QuantumMeasurement,
    pub target: Option<Box<Identifier>>,
}

impl Display for MeasureStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(target) = &self.target {
            write!(f, "MeasureStmt {}: {}, {}", self.span, self.measure, target)
        } else {
            write!(f, "MeasureStmt {}: {}", self.span, self.measure)
        }
    }
}

#[derive(Clone, Debug)]
pub struct ClassicalDeclarationStmt {
    pub span: Span,
    pub r#type: ClassicalType,
    pub identifier: Identifier,
    pub init_expr: Option<Box<ValueExpression>>,
}

impl Display for ClassicalDeclarationStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(init_expr) = &self.init_expr {
            write!(
                f,
                "ClassicalDeclarationStmt {}: {}, {}, {}",
                self.span, self.r#type, self.identifier, init_expr
            )
        } else {
            write!(
                f,
                "ClassicalDeclarationStmt {}: {}, {}",
                self.span, self.r#type, self.identifier
            )
        }
    }
}

#[derive(Clone, Debug)]
pub enum ValueExpression {
    Expr(ExprStmt),
    Measurement(QuantumMeasurement),
}

impl Display for ValueExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ValueExpression::Expr(expr) => write!(f, "ValueExpression {expr}"),
            ValueExpression::Measurement(measure) => write!(f, "ValueExpression {measure}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct IODeclaration {
    pub span: Span,
    pub io_identifier: IOKeyword,
    pub r#type: ClassicalType,
    pub identifier: Identifier,
}

impl Display for IODeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "IODeclaration {}: {}, {}, {}",
            self.span, self.io_identifier, self.r#type, self.identifier
        )
    }
}

#[derive(Clone, Debug)]
pub struct ConstantDeclaration {
    span: Span,
    r#type: ClassicalType,
    identifier: Identifier,
    init_expr: ExprStmt,
}

impl Display for ConstantDeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ConstantDeclaration {}: {}, {}, {}",
            self.span, self.r#type, self.identifier, self.init_expr
        )
    }
}

#[derive(Clone, Debug)]
pub struct CalibrationGrammarDeclaration {
    span: Span,
    name: String,
}

impl Display for CalibrationGrammarDeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CalibrationGrammarDeclaration {}: {}",
            self.span, self.name
        )
    }
}

#[derive(Clone, Debug)]
pub struct CalibrationStmt {
    span: Span,
    body: String,
}

impl Display for CalibrationStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "CalibrationStmt {}: {}", self.span, self.body)
    }
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

impl Display for CalibrationDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "CalibrationDefinition {}: {}", self.span, self.name)?;
        for arg in &self.args {
            write!(indent, "\n{arg}")?;
        }
        for qubit in &self.qubits {
            write!(indent, "\n{qubit}")?;
        }
        if let Some(return_type) = &self.return_type {
            write!(indent, "\n{return_type}")?;
        }
        write!(indent, "\n{}", self.body)
    }
}

#[derive(Clone, Debug)]
pub enum CalibrationArgument {
    Classical(ClassicalArgument),
    Expr(ExprStmt),
}

impl Display for CalibrationArgument {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CalibrationArgument::Classical(arg) => write!(f, "CalibrationArgument {arg}"),
            CalibrationArgument::Expr(expr) => write!(f, "CalibrationArgument {expr}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DefStmt {
    span: Span,
    name: Identifier,
    args: List<Box<Operand>>,
    body: List<Box<Stmt>>,
    return_type: Option<ClassicalType>,
}

impl Display for DefStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "DefStmt {}: {}", self.span, self.name)?;
        for arg in &self.args {
            write!(indent, "\n{arg}")?;
        }
        for stmt in &self.body {
            write!(indent, "\n{stmt}")?;
        }
        if let Some(return_type) = &self.return_type {
            write!(indent, "\n{return_type}")?;
        }
        Ok(())
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
            Operand::Classical(arg) => write!(f, "Operand {arg}"),
            Operand::Quantum(arg) => write!(f, "Operand {arg}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ReturnStmt {
    span: Span,
    expr: Option<Box<ValueExpression>>,
}

impl Display for ReturnStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(expr) = &self.expr {
            write!(f, "ReturnStmt {}: {}", self.span, expr)
        } else {
            write!(f, "ReturnStmt {}: <no expr>", self.span)
        }
    }
}

#[derive(Clone, Debug)]
pub struct BranchingStmt {
    span: Span,
    condition: ExprStmt,
    if_block: List<Stmt>,
    else_block: List<Stmt>,
}

impl Display for BranchingStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "BranchingStmt {}: {}", self.span, self.condition)?;
        for stmt in &self.if_block {
            write!(indent, "\n{stmt}")?;
        }
        for stmt in &self.else_block {
            write!(indent, "\n{stmt}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct WhileLoop {
    span: Span,
    while_condition: ExprStmt,
    block: List<Stmt>,
}

impl Display for WhileLoop {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "WhileLoop {}: {}", self.span, self.while_condition)?;
        for stmt in &self.block {
            write!(indent, "\n{stmt}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ForStmt {
    span: Span,
    r#type: ClassicalType,
    identifier: Identifier,
    set_declaration: Box<EnumerableSet>,
    block: List<Stmt>,
}

impl Display for ForStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(
            indent,
            "ForStmt {}: {}, {}, {}",
            self.span, self.r#type, self.identifier, self.set_declaration
        )?;
        for stmt in &self.block {
            write!(indent, "\n{stmt}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum EnumerableSet {
    DiscreteSet(DiscreteSet),
    RangeDefinition(RangeDefinition),
    Expr(ExprStmt),
}

impl Display for EnumerableSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EnumerableSet::DiscreteSet(set) => write!(f, "{set}"),
            EnumerableSet::RangeDefinition(range) => {
                let indent = set_indentation(indented(f), 0);
                display_range(indent, range)
            }
            EnumerableSet::Expr(expr) => write!(f, "{expr}"),
        }
    }
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

impl Display for SwitchStmt {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        todo!("SwitchStmt display");
    }
}

#[derive(Clone, Debug)]
pub struct ClassicalAssignment {
    pub span: Span,
    pub lvalue: Identifier,
    pub op: AssignmentOp,
}

impl Display for ClassicalAssignment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClassicalAssignment {}: {}, {}",
            self.span, self.lvalue, self.op
        )
    }
}

#[derive(Clone, Debug, Default)]
pub enum ExprKind {
    /// An expression with invalid syntax that can't be parsed.
    #[default]
    Err,
    Ident(Ident),
    UnaryExpr(UnaryExpr),
    BinaryExpr(BinaryExpr),
    Lit(Lit),
    FunctionCall(FunctionCall),
    Cast(Cast),
    Concatenation(Concatenation),
    IndexExpr(IndexExpr),
}

impl Display for ExprKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let indent = set_indentation(indented(f), 0);
        match self {
            ExprKind::Err => write!(f, "Err"),
            ExprKind::Ident(id) => write!(f, "{id}"),
            ExprKind::UnaryExpr(expr) => write!(f, "{expr}"),
            ExprKind::BinaryExpr(expr) => display_bin_op(indent, expr),
            ExprKind::Lit(lit) => write!(f, "{lit}"),
            ExprKind::FunctionCall(call) => write!(f, "{call}"),
            ExprKind::Cast(cast) => write!(f, "{cast}"),
            ExprKind::Concatenation(concat) => write!(f, "{concat}"),
            ExprKind::IndexExpr(index) => write!(f, "{index}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UnaryExpr {
    pub span: Span,
    pub op: UnaryOp,
    pub expr: Box<Expr>,
}

impl Display for UnaryExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let indent = set_indentation(indented(f), 0);
        display_un_op(indent, self.op, &self.expr)
    }
}

#[derive(Clone, Debug)]
pub struct BinaryExpr {
    pub span: Span,
    pub op: BinaryOp,
    pub lhs: ExprStmt,
    pub rhs: ExprStmt,
}

impl Display for BinaryExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let indent = set_indentation(indented(f), 0);
        display_bin_op(indent, self)
    }
}

#[derive(Clone, Debug)]
pub struct FunctionCall {
    pub span: Span,
    pub name: Identifier,
    pub args: List<ExprStmt>,
}

impl Display for FunctionCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "FunctionCall {}: {}", self.span, self.name)?;
        indent = set_indentation(indent, 1);
        for arg in &self.args {
            write!(indent, "\n{arg}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Cast {
    pub span: Span,
    pub r#type: ClassicalType,
    pub arg: ExprStmt,
}

impl Display for Cast {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Cast {}: {}, {}", self.span, self.r#type, self.arg)
    }
}

#[derive(Clone, Debug)]
pub struct IndexExpr {
    pub span: Span,
    pub collection: ExprStmt,
    pub index: IndexElement,
}

impl Display for IndexExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "IndexExpr {}: {}, {}",
            self.span, self.collection, self.index
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub enum UnaryOp {
    NegB,
    NegL,
    NegN,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::NegB => write!(f, "NegB"),
            UnaryOp::NegL => write!(f, "NegL"),
            UnaryOp::NegN => write!(f, "NegN"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
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

impl Display for Lit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Lit: {}", self.kind)
    }
}

#[derive(Clone, Debug)]
pub enum LiteralKind {
    Array(List<ExprStmt>),
    Bitstring(BigInt, usize),
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
            LiteralKind::Array(exprs) => {
                let mut indent = set_indentation(indented(f), 0);
                write!(indent, "Array:")?;
                indent = set_indentation(indent, 1);
                for expr in exprs {
                    write!(indent, "\n{expr}")?;
                }
                Ok(())
            }
            LiteralKind::Bitstring(value, width) => {
                write!(f, "Bitstring({:0>width$})", value.to_str_radix(2))
            }
            LiteralKind::Bool(b) => write!(f, "Bool({b})"),
            LiteralKind::Duration { value, unit } => {
                write!(f, "Duration({value}, {unit})")
            }
            LiteralKind::Float(value) => write!(f, "Float({value})"),
            LiteralKind::Imaginary(value) => write!(f, "Imaginary({value})"),
            LiteralKind::Int(i) => write!(f, "Int({i})"),
            LiteralKind::BigInt(i) => write!(f, "BigInt({i})"),
            LiteralKind::String(s) => write!(f, "String({s})"),
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
pub struct Concatenation {
    lhs: ExprStmt,
    rhs: ExprStmt,
}

impl Display for Concatenation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "Concatenation:")?;
        indent = set_indentation(indent, 1);
        write!(indent, "\n{}", self.lhs)?;
        write!(indent, "\n{}", self.rhs)?;
        Ok(())
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
            IndexElement::DiscreteSet(set) => write!(f, "IndexElement {set}"),
            IndexElement::IndexSet(items) => {
                let mut indent = set_indentation(indented(f), 0);
                write!(indent, "IndexElement:")?;
                indent = set_indentation(indent, 1);
                for item in items {
                    write!(indent, "\n{item}")?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum IndexSetItem {
    RangeDefinition(RangeDefinition),
    Expr(ExprStmt),
}

impl Display for IndexSetItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let indent = set_indentation(indented(f), 0);
        match self {
            IndexSetItem::RangeDefinition(range) => display_range(indent, range),
            IndexSetItem::Expr(expr) => write!(f, "IndexSetItem {expr}"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum AssignmentOp {
    BinaryOp(BinaryOp),
    /// `OpenQASM3` has the `~=` assignment operator.
    /// This enum variant is meant to capture that.
    UnaryOp(UnaryOp),
    Assign,
}

impl Display for AssignmentOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AssignmentOp::BinaryOp(op) => write!(f, "AssignmentOp ({op:?})"),
            AssignmentOp::UnaryOp(op) => write!(f, "AssignmentOp ({op:?})"),
            AssignmentOp::Assign => write!(f, "AssignmentOp (Assign)"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum GateModifierName {
    Inv,
    Pow,
    Ctrl,
    NegCtrl,
}

impl Display for GateModifierName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            GateModifierName::Inv => write!(f, "inv"),
            GateModifierName::Pow => write!(f, "pow"),
            GateModifierName::Ctrl => write!(f, "ctrl"),
            GateModifierName::NegCtrl => write!(f, "negctrl"),
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
        write!(f, "Break {}", self.span)
    }
}

#[derive(Clone, Debug)]
pub struct ContinueStmt {
    pub span: Span,
}

impl Display for ContinueStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Continue {}", self.span)
    }
}

fn display_assign(mut indent: Indented<Formatter>, lhs: &Expr, rhs: &Expr) -> fmt::Result {
    write!(indent, "Assign:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{lhs}")?;
    write!(indent, "\n{rhs}")?;
    Ok(())
}

fn display_assign_op(
    mut indent: Indented<Formatter>,
    op: BinaryOp,
    lhs: &Expr,
    rhs: &Expr,
) -> fmt::Result {
    write!(indent, "AssignOp ({op:?}):")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{lhs}")?;
    write!(indent, "\n{rhs}")?;
    Ok(())
}

fn display_bin_op(mut indent: Indented<Formatter>, expr: &BinaryExpr) -> fmt::Result {
    write!(indent, "BinOp ({:?}):", expr.op)?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{}", expr.lhs)?;
    write!(indent, "\n{}", expr.rhs)?;
    Ok(())
}

fn display_un_op(mut indent: Indented<Formatter>, op: UnaryOp, expr: &Expr) -> fmt::Result {
    write!(indent, "UnOp ({op}):")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{expr}")?;
    Ok(())
}

fn display_while(mut indent: Indented<Formatter>, cond: &Expr, block: &Block) -> fmt::Result {
    write!(indent, "While:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{cond}")?;
    write!(indent, "\n{block}")?;
    Ok(())
}

fn display_range(mut indent: Indented<Formatter>, range: &RangeDefinition) -> fmt::Result {
    write!(indent, "Range: {}", range.span)?;
    indent = set_indentation(indent, 1);
    match &range.start {
        Some(e) => write!(indent, "\n{e}")?,
        None => write!(indent, "\n<no start>")?,
    }
    match &range.step {
        Some(e) => write!(indent, "\n{e}")?,
        None => write!(indent, "\n<no step>")?,
    }
    match &range.end {
        Some(e) => write!(indent, "\n{e}")?,
        None => write!(indent, "\n<no end>")?,
    }
    Ok(())
}
