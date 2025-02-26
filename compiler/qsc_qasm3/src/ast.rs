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
        3 => indent.with_str("            "),
        _ => unimplemented!("indentation level not supported"),
    }
}

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
        write!(indent, "Stmt {}", self.span)?;
        indent = set_indentation(indent, 1);
        for annotation in &self.annotations {
            write!(indent, "\n{annotation}")?;
        }
        write!(indent, "\n{}", self.kind)?;
        Ok(())
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
    pub ident: Identifier,
    pub exprs: List<Expr>,
    pub span: Span,
}

impl Display for AliasDeclStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "Alias {}: {}", self.span, self.ident)?;
        indent = set_indentation(indent, 1);
        for expr in &*self.exprs {
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
    ConstDecl(ConstantDeclaration),
    Continue(ContinueStmt),
    Def(DefStmt),
    DefCal(DefCalStmt),
    DelayStmt(DelayStmt),
    /// An empty statement.
    Empty,
    End(EndStmt),
    ExprStmt(ExprStmt),
    ExternDecl(ExternDecl),
    For(ForStmt),
    If(IfStmt),
    GateCall(GateCall),
    GPhase(QuantumPhase),
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
        write!(f, "CalibrationGrammarStmt {}: {}", self.span, self.name)
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
pub struct ExprStmt {
    pub span: Span,
    pub expr: Expr,
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
    pub values: Box<[Expr]>,
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
    pub start: Option<Expr>,
    pub end: Option<Expr>,
    pub step: Option<Expr>,
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
        write!(f, "ClassicalType {}: {}", self.span, self.kind)
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
    pub size: Option<Expr>,
    pub span: Span,
}

impl Display for IntType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(size) = &self.size {
            write!(f, "IntType[{}]: {}", size, self.span)
        } else {
            write!(f, "IntType {}", self.span)
        }
    }
}

#[derive(Clone, Debug)]
pub struct UIntType {
    pub size: Option<Expr>,
    pub span: Span,
}

impl Display for UIntType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(size) = &self.size {
            write!(f, "UIntType[{}]: {}", size, self.span)
        } else {
            write!(f, "UIntType {}", self.span)
        }
    }
}

#[derive(Clone, Debug)]
pub struct FloatType {
    pub size: Option<Expr>,
    pub span: Span,
}

impl Display for FloatType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(size) = &self.size {
            write!(f, "FloatType[{}]: {}", size, self.span)
        } else {
            write!(f, "FloatType {}", self.span)
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
            write!(f, "ComplexType[float[{}]]: {}", size, self.span)
        } else {
            write!(f, "ComplexType {}", self.span)
        }
    }
}

#[derive(Clone, Debug)]
pub struct AngleType {
    pub size: Option<Expr>,
    pub span: Span,
}

impl Display for AngleType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(size) = &self.size {
            write!(f, "AngleType {}: {}", self.span, size)
        } else {
            write!(f, "AngleType {}", self.span)
        }
    }
}

#[derive(Clone, Debug)]
pub struct BitType {
    pub size: Option<Expr>,
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
pub enum TypeDef {
    Scalar(ScalarType),
    Array(ArrayType),
    ArrayReference(ArrayReferenceType),
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
    pub mutability: AccessControl,
    pub base_type: ArrayBaseTypeKind,
    pub dimensions: List<Expr>,
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
    pub expr: Option<Expr>,
}

impl Display for QuantumArgument {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "QuantumArgument {}: {:?}", self.span, self.expr)
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
        if let Some(value) = &self.value {
            write!(f, "Pragma {}: ({}, {})", self.span, self.identifier, value)
        } else {
            write!(f, "Pragma {}: ({})", self.span, self.identifier)
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
    pub qubit: Box<Ident>,
    pub size: Option<Expr>,
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
    pub ident: Box<Ident>,
    pub params: List<Ident>,
    pub qubits: List<Ident>,
    pub body: Box<Block>,
}

impl Display for QuantumGateDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "Gate {}: {}", self.span, self.ident)?;
        write!(indent, "(")?;
        if self.params.is_empty() {
            write!(indent, "<no params>")?;
        } else {
            let param_str = self
                .params
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            write!(indent, "{param_str}")?;
        }
        write!(indent, ") ")?;

        let qubit_str = self
            .qubits
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        write!(indent, "{qubit_str}")?;

        writeln!(indent)?;
        for stmt in &self.body.stmts {
            write!(indent, "\n{stmt}")?;
        }
        Ok(())
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
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "ExternDecl {}: {}", self.span, self.ident)?;
        for arg in &self.params {
            write!(indent, "\n{arg}")?;
        }
        if let Some(return_type) = &self.return_type {
            write!(indent, "\n{return_type}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct GateCall {
    pub span: Span,
    pub modifiers: List<QuantumGateModifier>,
    pub name: Identifier,
    pub args: List<Expr>,
    pub qubits: List<GateOperand>,
    pub duration: Option<Expr>,
}

impl Display for GateCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "GateCall {}: {}", self.span, self.name)?;
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
    pub arg: Expr,
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
pub struct DelayStmt {
    pub span: Span,
    pub duration: Expr,
    pub qubits: List<GateOperand>,
}

impl Display for DelayStmt {
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
    pub duration: Option<Expr>,
    pub body: List<Stmt>,
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
    pub measurement: MeasureExpr,
    pub target: Option<Box<IndexedIdent>>,
}

impl Display for MeasureStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(target) = &self.target {
            write!(
                f,
                "MeasureStmt {}: {}, {}",
                self.span, self.measurement, target
            )
        } else {
            write!(f, "MeasureStmt {}: {}", self.span, self.measurement)
        }
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
    Expr(Expr),
    Measurement(MeasureExpr),
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
    pub r#type: TypeDef,
    pub ident: Box<Ident>,
}

impl Display for IODeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "IODeclaration {}: {}, {}, {}",
            self.span, self.io_identifier, self.r#type, self.ident
        )
    }
}

#[derive(Clone, Debug)]
pub struct ConstantDeclaration {
    pub span: Span,
    pub r#type: TypeDef,
    pub identifier: Box<Ident>,
    pub init_expr: Expr,
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
    pub span: Span,
}

impl Display for CalibrationStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "CalibrationStmt {}", self.span)
    }
}

#[derive(Clone, Debug)]
pub struct CalibrationDefinition {
    span: Span,
    name: Identifier,
    args: List<CalibrationArgument>,
    qubits: List<Identifier>,
    return_type: Option<ScalarType>,
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
    Expr(Expr),
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
                write!(f, "{span} {ident}: {scalar}")
            }
            TypedParameter::Quantum(expr, ident, span) => {
                if let Some(expr) = expr {
                    write!(f, "{span} {ident}: qubit[{expr}]")
                } else {
                    write!(f, "{span} {ident}: qubit")
                }
            }
            TypedParameter::ArrayReference(array, ident, span) => {
                write!(f, "{span} {ident}: {array}")
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
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "DefStmt {}: {}", self.span, self.name)?;
        write!(indent, "(")?;
        if self.params.is_empty() {
            write!(indent, "<no params>")?;
        } else {
            let param_str = self
                .params
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            write!(indent, "{param_str}")?;
        }
        write!(indent, ") ")?;

        for stmt in &self.body.stmts {
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
    pub span: Span,
    pub expr: Option<Box<ValueExpression>>,
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
pub struct WhileLoop {
    pub span: Span,
    pub while_condition: Expr,
    pub block: List<Stmt>,
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
    pub span: Span,
    pub r#type: ScalarType,
    pub identifier: Identifier,
    pub set_declaration: Box<EnumerableSet>,
    pub block: List<Stmt>,
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
    Expr(Expr),
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
    pub target: Expr,
    pub cases: List<(List<Expr>, Block)>,
    /// Note that `None` is quite different to `[]` in this case; the latter is
    /// an explicitly empty body, whereas the absence of a default might mean
    /// that the switch is inexhaustive, and a linter might want to complain.
    pub default: Option<Block>,
}

impl Display for SwitchStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "SwitchStmt {}:", self.span)?;
        let mut indent = set_indentation(indented(f), 1);
        write!(indent, "\nTarget: {}", self.target)?;
        if self.cases.is_empty() {
            write!(indent, "\n<no cases>")?;
        } else {
            write!(indent, "\nCases:")?;
            for elt in &self.cases {
                let (labels, block) = &**elt;
                indent = display_switch_case(indent, labels, block)?;
            }
        }
        if let Some(default) = &self.default {
            write!(indent, "\nDefault Case:")?;
            indent = set_indentation(indent, 2);
            write!(indent, "\n{default}")?;
        } else {
            write!(indent, "\n<no default>")?;
        }
        Ok(())
    }
}

fn display_switch_case<'a, 'b>(
    mut indent: Indented<'a, Formatter<'b>>,
    labels: &List<Expr>,
    block: &Block,
) -> Result<Indented<'a, Formatter<'b>>, core::fmt::Error> {
    indent = set_indentation(indent, 2);
    if labels.is_empty() {
        write!(indent, "\n<no labels>")?;
    } else {
        write!(indent, "\nLabels:")?;
        indent = set_indentation(indent, 3);
        for label in labels {
            write!(indent, "\n{label}")?;
        }
    }
    indent = set_indentation(indent, 2);
    write!(indent, "\n{block}")?;
    Ok(indent)
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
        let indent = set_indentation(indented(f), 0);
        match self {
            ExprKind::Err => write!(f, "Err"),
            ExprKind::Ident(id) => write!(f, "{id}"),
            ExprKind::UnaryOp(expr) => write!(f, "{expr}"),
            ExprKind::BinaryOp(expr) => display_bin_op(indent, expr),
            ExprKind::Lit(lit) => write!(f, "{lit}"),
            ExprKind::FunctionCall(call) => write!(f, "{call}"),
            ExprKind::Cast(cast) => display_cast(indent, cast),
            ExprKind::IndexExpr(index) => write!(f, "{index}"),
            ExprKind::Assign(expr) => write!(f, "{expr}"),
            ExprKind::AssignOp(expr) => write!(f, "{expr}"),
            ExprKind::Paren(expr) => display_paren(indent, expr),
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
        let indent = set_indentation(indented(f), 0);
        display_assign(indent, &self.lhs, &self.rhs)
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
        let indent = set_indentation(indented(f), 0);
        display_assign_op(indent, self.op, &self.lhs, &self.rhs)
    }
}

#[derive(Clone, Debug)]
pub struct UnaryOpExpr {
    pub op: UnaryOp,
    pub expr: Expr,
}

impl Display for UnaryOpExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let indent = set_indentation(indented(f), 0);
        display_un_op(indent, self.op, &self.expr)
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
        let indent = set_indentation(indented(f), 0);
        display_bin_op(indent, self)
    }
}

#[derive(Clone, Debug)]
pub struct FunctionCall {
    pub span: Span,
    pub name: Identifier,
    pub args: List<Expr>,
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
    pub r#type: TypeDef,
    pub arg: Expr,
}

impl Display for Cast {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Cast {}: {}, {}", self.span, self.r#type, self.arg)
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
        write!(
            f,
            "IndexExpr {}: {}, {}",
            self.span, self.collection, self.index
        )
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
                    write!(indent, "\n{expr:?}")?;
                }
                Ok(())
            }
            LiteralKind::Bitstring(value, width) => {
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
        let indent = set_indentation(indented(f), 0);
        match self {
            IndexSetItem::RangeDefinition(range) => display_range(indent, range),
            IndexSetItem::Expr(expr) => write!(f, "IndexSetItem {expr}"),
            IndexSetItem::Err => write!(f, "Err"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum AssignmentOp {
    BinaryOp(BinOp),
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

#[derive(Clone, Debug)]
pub struct EndStmt {
    pub span: Span,
}

impl Display for EndStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "End {}", self.span)
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
    op: BinOp,
    lhs: &Expr,
    rhs: &Expr,
) -> fmt::Result {
    write!(indent, "AssignOp ({op:?}):")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{lhs}")?;
    write!(indent, "\n{rhs}")?;
    Ok(())
}

fn display_bin_op(mut indent: Indented<Formatter>, expr: &BinaryOpExpr) -> fmt::Result {
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

fn display_paren(mut indent: Indented<Formatter>, expr: &Expr) -> fmt::Result {
    write!(indent, "Paren:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{expr}")?;
    Ok(())
}
fn display_cast(mut indent: Indented<Formatter>, cast: &Cast) -> fmt::Result {
    let Cast { span, r#type, arg } = cast;
    write!(indent, "Cast {span}:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{type}\n{arg}")?;
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
        Some(e) => write!(indent, "\nstart: {e}")?,
        None => write!(indent, "\n<no start>")?,
    }
    match &range.step {
        Some(e) => write!(indent, "\nstep: {e}")?,
        None => write!(indent, "\n<no step>")?,
    }
    match &range.end {
        Some(e) => write!(indent, "\nend: {e}")?,
        None => write!(indent, "\n<no end>")?,
    }
    Ok(())
}
