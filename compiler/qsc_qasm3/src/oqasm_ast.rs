use qsc::Span;
use std::convert::Infallible;

// TODO: profile this with iai-callgrind in a large OpenQASM3
// sample to verify that is actually faster than using Vec<T>.
/// An alternative to `Vec<T>` that uses less stack space.
type List<T> = Box<[Box<T>]>;
type Identifier = String;

enum Union<T1, T2, T3 = Infallible> {
    First(T1),
    Second(T2),
    Third(T3),
}

enum QASMNode {
    Program(Program),
    Annotation(Annotation),
    Stmt(Stmt),
    Expr(Expr),
    DiscreteSet(DiscreteSet),
    RangeDefinition(RangeDefinition),
    IndexedIdentifier(IndexedIdentifier),
    QuantumGateModifier(QuantumGateModifier),
    QuantumMeasurement(QuantumMeasurement),
    ClassicalArgument(ClassicalArgument),
    ExternArgument(ExternArgument),
    ClassicalType(ClassicalType),
    QuantumArgument(QuantumArgument),
    Pragma(Pragma),
}

struct Program {
    span: Span,
    statements: List<Union<Stmt, Pragma>>,
    version: Option<String>,
}

struct Annotation {
    span: Span,
    keyword: String,
    command: Option<String>,
}

struct Stmt {
    span: Span,
    annotations: List<Annotation>,
    kind: Box<StmtKind>,
}

struct Expr {
    span: Span,
    kind: Box<ExprKind>,
}

struct DiscreteSet {
    span: Span,
    values: List<Expr>,
}

struct RangeDefinition {
    start: Option<Expr>,
    end: Option<Expr>,
    step: Option<Expr>,
}

struct IndexedIdentifier {
    name: Identifier,
    indices: List<IndexElement>,
}

struct QuantumGateModifier {
    span: Span,
    qubit: Union<IndexedIdentifier, Identifier>,
}

struct QuantumMeasurement {
    span: Span,
    qubit: Union<IndexedIdentifier, Identifier>,
}

struct ClassicalArgument {
    span: Span,
    r#type: ClassicalType,
    name: Identifier,
    access: Option<AccessControl>,
}

struct ExternArgument {
    span: Span,
    r#type: ClassicalType,
    access: Option<AccessControl>,
}

enum ClassicalType {
    Int(IntType),
    UInt(UIntType),
    Float(FloatType),
    Complex(ComplexType),
    Angle(AngleType),
    Bit(BitType),
    BoolType,
    Array {
        base_type: ArrayBaseType,
        dimensions: List<Expr>,
    },
    ArrayReference {
        base_type: ArrayBaseType,
        dimensions: Union<Expr, List<Expr>>,
    },
    Duration,
    Stretch,
}

enum ArrayBaseType {
    Int(IntType),
    UInt(UIntType),
    Float(FloatType),
    Complex(ComplexType),
    Angle(AngleType),
    Bit(BitType),
    BoolType,
}

struct IntType {
    size: Option<Expr>,
}

struct UIntType {
    size: Option<Expr>,
}

struct FloatType {
    size: Option<Expr>,
}

struct ComplexType {
    base_size: Option<FloatType>,
}

struct AngleType {
    size: Option<Expr>,
}

struct BitType {
    size: Option<Expr>,
}

enum AccessControl {
    ReadOnly,
    Mutable,
}

struct QuantumArgument {
    span: Span,
    size: Option<Expr>,
}

struct Pragma {
    span: Span,
    command: String,
}

enum StmtKind {
    CompoundStmt(CompoundStmt),
    Include(String),
    ExpressionStatement(Expr),
    QubitDeclaration(Identifier, Option<Expr>),
    QuantumGateDefinition(QuantumGateDefinition),
    ExternDeclaration(ExternDeclaration),
    Quantum(QuantumStmt),
    Measurement(QuantumMeasurementStmt),
    ClassicalDeclaration(ClassicalDeclaration),
    IODeclaration(IODeclaration),
    ConstantDeclaration(ConstantDeclaration),
    CalibrationGrammarDeclaration { name: String },
    CalibrationStatement { body: String },
    CalibrationDefinition(CalibrationDefinition),
    SubroutineDefinition(SubroutineDefinition),
    Return(Option<Union<Expr, QuantumMeasurement>>),
    Break,
    Continue,
    Branching(BranchingStmt),
    WhileLoop(WhileLoop),
    ForInLoop(ForInLoop),
    Switch(SwitchStmt),
    ClassicalAssignment(ClassicalAssignment),
}

type CompoundStmt = List<Stmt>;

struct QuantumGateDefinition {
    name: Identifier,
    arguments: Vec<Identifier>,
    qubits: Vec<Identifier>,
    body: Vec<Stmt>,
}

struct ExternDeclaration {
    name: Identifier,
    arguments: List<ExternArgument>,
    return_type: Option<ClassicalType>,
}

enum QuantumStmt {
    Gate(QuantumGate),
    Phase(QuantumPhase),
    Barrier(List<Expr>),
    Reset(List<Union<IndexedIdentifier, Identifier>>),
    DelayInstruction(DelayInstruction),
    Box(BoxStmt),
}

struct QuantumGate {
    modifiers: List<QuantumGateModifier>,
    name: Identifier,
    args: List<Expr>,
    qubits: List<Union<IndexedIdentifier, Identifier>>,
    duration: Option<Expr>,
}

struct QuantumPhase {
    modifiers: List<QuantumGateModifier>,
    arg: Expr,
    qubits: List<Union<IndexedIdentifier, Identifier>>,
}

struct DelayInstruction {
    duration: Expr,
    qubits: List<Union<IndexedIdentifier, Identifier>>,
}

struct BoxStmt {
    duration: Option<Expr>,
    body: List<QuantumStmt>,
}

struct QuantumMeasurementStmt {
    measure: QuantumMeasurement,
    target: Option<Box<Union<Identifier, IndexedIdentifier>>>,
}

struct ClassicalDeclaration {
    r#type: ClassicalType,
    identifier: Identifier,
    init_expr: Option<Union<Expr, QuantumMeasurement>>,
}

struct IODeclaration {
    io_identifier: IOKeyword,
    r#type: ClassicalType,
    identifier: Identifier,
}

struct ConstantDeclaration {
    r#type: ClassicalType,
    identifier: Identifier,
    init_expr: Expr,
}

struct CalibrationDefinition {
    name: Identifier,
    args: List<Union<ClassicalArgument, Expr>>,
    qubits: List<Identifier>,
    return_type: Option<ClassicalType>,
    body: String,
}

struct SubroutineDefinition {
    name: Identifier,
    args: List<Union<ClassicalArgument, QuantumArgument>>,
    body: List<Stmt>,
    return_type: Option<ClassicalType>,
}

struct BranchingStmt {
    condition: Expr,
    if_block: List<Stmt>,
    else_block: List<Stmt>,
}

struct WhileLoop {
    while_condition: Expr,
    block: List<Stmt>,
}

struct ForInLoop {
    r#type: ClassicalType,
    identifier: Identifier,
    set_declaration: Union<RangeDefinition, DiscreteSet, Expr>,
    block: List<Stmt>,
}

struct SwitchStmt {
    target: Expr,
    cases: List<(List<Expr>, CompoundStmt)>,
    /// Note that `None` is quite different to `[]` in this case; the latter is
    /// an explicitly empty body, whereas the absence of a default might mean
    /// that the switch is inexhaustive, and a linter might want to complain.
    default: Option<CompoundStmt>,
}

struct ClassicalAssignment {
    lvalue: Union<Identifier, IndexedIdentifier>,
    op: AssignmentOp,
}

enum ExprKind {
    Identifier(Identifier),
    UnaryExpr(UnaryOp, Expr),
    BinaryExpr(BinaryOp, Expr, Expr),
    Literal(Literal),
    FunctionCall {
        name: Identifier,
        args: List<Expr>,
    },
    Cast {
        r#type: ClassicalType,
        arg: Expr,
    },
    Concatenation(Concatenation),
    IndexExpr {
        collection: Expr,
        index: IndexElement,
    },
    DurationOf {
        target: List<Stmt>,
    },
    SizeOf {
        target: Expr,
        value: Union<Identifier, Concatenation>,
    },
}

enum UnaryOp {
    NegB,
    NegL,
    NegN,
}

enum BinaryOp {
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

enum Literal {
    Integer(i64),
    Float(f64),
    Imaginary(f64),
    Boolean(bool),
    Bitstring { value: usize, width: u32 },
    Duration { value: f64, unit: TimeUnit },
    Array(List<Expr>),
}

struct Concatenation {
    lhs: Expr,
    rhs: Expr,
}

type IndexElement = Union<DiscreteSet, List<Union<Expr, RangeDefinition>>>;

enum AssignmentOp {
    BinaryOp(BinaryOp),
    /// OpenQASM3 has the `~=` assignment operator.
    /// This enum variant is meant to capture that.
    UnaryOp(UnaryOp),
    Assign,
}

enum GateModifierName {
    Inv,
    Pow,
    Ctrl,
    NegCtrl,
}

enum IOKeyword {
    Input,
    Output,
}

enum TimeUnit {
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
