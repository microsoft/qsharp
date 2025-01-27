use qsc::Span;
use std::convert::Infallible;

// TODO: profile this with iai-callgrind in a large OpenQASM3
// sample to verify that is actually faster than using Vec<T>.
/// An alternative to `Vec<T>` that uses less stack space.
type List<T> = Box<[Box<T>]>;

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
    span: Span,
    start: Option<Expr>,
    end: Option<Expr>,
    step: Option<Expr>,
}

struct Identifier {
    span: Span,
    name: String,
}

struct IndexedIdentifier {
    span: Span,
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

struct ClassicalType {
    span: Span,
    kind: ClassicalTypeKind,
}

enum ClassicalTypeKind {
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

enum ArrayBaseTypeKind {
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

struct ArrayType {
    span: Span,
    base_type: ArrayBaseTypeKind,
    dimensions: List<Expr>,
}

struct ArrayReferenceType {
    span: Span,
    base_type: ArrayBaseTypeKind,
    dimensions: Union<Expr, List<Expr>>,
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
    Include(IncludeStmt),
    ExpressionStmt(Expr),
    QubitDeclaration(QubitDeclaration),
    QuantumGateDefinition(QuantumGateDefinition),
    ExternDeclaration(ExternDeclaration),
    Quantum(QuantumStmt),
    Measurement(QuantumMeasurementStmt),
    ClassicalDeclaration(ClassicalDeclaration),
    IODeclaration(IODeclaration),
    ConstantDeclaration(ConstantDeclaration),
    CalibrationGrammarDeclaration(CalibrationGrammarDeclaration),
    CalibrationStmt(CalibrationStmt),
    CalibrationDefinition(CalibrationDefinition),
    SubroutineDefinition(SubroutineDefinition),
    Return(ReturnStmt),
    Break,
    Continue,
    Branching(BranchingStmt),
    WhileLoop(WhileLoop),
    ForInLoop(ForInLoop),
    Switch(SwitchStmt),
    ClassicalAssignment(ClassicalAssignment),
}

struct CompoundStmt {
    span: Span,
    statements: List<Stmt>,
}

struct IncludeStmt {
    span: Span,
    filename: String,
}

struct QubitDeclaration {
    span: Span,
    qubit: Identifier,
    size: Option<Expr>,
}

struct QuantumGateDefinition {
    span: Span,
    name: Identifier,
    arguments: Vec<Identifier>,
    qubits: Vec<Identifier>,
    body: Vec<Stmt>,
}

struct ExternDeclaration {
    span: Span,
    name: Identifier,
    arguments: List<ExternArgument>,
    return_type: Option<ClassicalType>,
}

struct QuantumStmt {
    span: Span,
    kind: QuantumStmtKind,
}

enum QuantumStmtKind {
    Gate(QuantumGate),
    Phase(QuantumPhase),
    Barrier(List<Expr>),
    Reset(List<Union<IndexedIdentifier, Identifier>>),
    DelayInstruction(DelayInstruction),
    Box(BoxStmt),
}

struct QuantumGate {
    span: Span,
    modifiers: List<QuantumGateModifier>,
    name: Identifier,
    args: List<Expr>,
    qubits: List<Union<IndexedIdentifier, Identifier>>,
    duration: Option<Expr>,
}

struct QuantumPhase {
    span: Span,
    modifiers: List<QuantumGateModifier>,
    arg: Expr,
    qubits: List<Union<IndexedIdentifier, Identifier>>,
}

struct DelayInstruction {
    span: Span,
    duration: Expr,
    qubits: List<Union<IndexedIdentifier, Identifier>>,
}

struct BoxStmt {
    span: Span,
    duration: Option<Expr>,
    body: List<QuantumStmt>,
}

struct QuantumMeasurementStmt {
    span: Span,
    measure: QuantumMeasurement,
    target: Option<Box<Union<Identifier, IndexedIdentifier>>>,
}

struct ClassicalDeclaration {
    span: Span,
    r#type: ClassicalType,
    identifier: Identifier,
    init_expr: Option<Union<Expr, QuantumMeasurement>>,
}

struct IODeclaration {
    span: Span,
    io_identifier: IOKeyword,
    r#type: ClassicalType,
    identifier: Identifier,
}

struct ConstantDeclaration {
    span: Span,
    r#type: ClassicalType,
    identifier: Identifier,
    init_expr: Expr,
}

struct CalibrationGrammarDeclaration {
    span: Span,
    name: String,
}

struct CalibrationStmt {
    span: Span,
    body: String,
}

struct CalibrationDefinition {
    span: Span,
    name: Identifier,
    args: List<Union<ClassicalArgument, Expr>>,
    qubits: List<Identifier>,
    return_type: Option<ClassicalType>,
    body: String,
}

struct SubroutineDefinition {
    span: Span,
    name: Identifier,
    args: List<Union<ClassicalArgument, QuantumArgument>>,
    body: List<Stmt>,
    return_type: Option<ClassicalType>,
}

struct ReturnStmt {
    span: Span,
    expr: Option<Union<Expr, QuantumMeasurement>>,
}

struct BranchingStmt {
    span: Span,
    condition: Expr,
    if_block: List<Stmt>,
    else_block: List<Stmt>,
}

struct WhileLoop {
    span: Span,
    while_condition: Expr,
    block: List<Stmt>,
}

struct ForInLoop {
    span: Span,
    r#type: ClassicalType,
    identifier: Identifier,
    set_declaration: Union<RangeDefinition, DiscreteSet, Expr>,
    block: List<Stmt>,
}

struct SwitchStmt {
    span: Span,
    target: Expr,
    cases: List<(List<Expr>, CompoundStmt)>,
    /// Note that `None` is quite different to `[]` in this case; the latter is
    /// an explicitly empty body, whereas the absence of a default might mean
    /// that the switch is inexhaustive, and a linter might want to complain.
    default: Option<CompoundStmt>,
}

struct ClassicalAssignment {
    span: Span,
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

struct Literal {
    span: Span,
    kind: LiteralKind,
}

enum LiteralKind {
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
