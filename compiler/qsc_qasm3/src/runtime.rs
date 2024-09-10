// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use bitflags::bitflags;

use qsc::{
    ast::{Stmt, TopLevelNode},
    LanguageFeatures,
};

/// Runtime functions that are used in the generated AST.
/// These functions are not part of the QASM3 standard, but are used to implement
/// utility fuctions that would be cumbersome to implement building the AST
/// directly.

/// The POW function is used to implement the `pow` modifier in QASM3 for integers.
const POW: &str = "
operation __Pow__<'T>(N: Int, op: ('T => Unit is Adj), target : 'T) : Unit is Adj {
    let op = if N > 0 { () => op(target) } else { () => Adjoint op(target) };
    for _ in 1..Microsoft.Quantum.Math.AbsI(N) {
        op()
    }
}
";

/// The ``BARRIER`` function is used to implement the `barrier` statement in QASM3.
/// The `@SimulatableIntrinsic` attribute is used to mark the operation for QIR
/// generation.
/// Q# doesn't support barriers, so this is a no-op. We need to figure out what
/// barriers mean in the context of QIR in the future for better support.
const BARRIER: &str = "
@SimulatableIntrinsic()
operation __quantum__qis__barrier__body() : Unit {}
";

/// The ``BOOL_AS_RESULT`` function is used to implement the cast expr in QASM3 for bool to bit.
/// This already exists in the Q# library, but is defined as a marker for casts from QASM3.
const BOOL_AS_RESULT: &str = "
function __BoolAsResult__(input: Bool) : Result {
    Microsoft.Quantum.Convert.BoolAsResult(input)
}
";

/// The ``BOOL_AS_INT`` function is used to implement the cast expr in QASM3 for bool to int.
const BOOL_AS_INT: &str = "
function __BoolAsInt__(value: Bool) : Int {
    if value {
        1
    } else {
        0
    }
}
";

/// The ``BOOL_AS_BIGINT`` function is used to implement the cast expr in QASM3 for bool to big int.
const BOOL_AS_BIGINT: &str = "
function __BoolAsBigInt__(value: Bool) : BigInt {
    if value {
        1L
    } else {
        0L
    }
}
";

/// The ``BOOL_AS_DOUBLE`` function is used to implement the cast expr in QASM3 for bool to int.
const BOOL_AS_DOUBLE: &str = "
function __BoolAsDouble__(value: Bool) : Double {
    if value {
        1.
    } else {
        0.
    }
}
";

/// The ``RESULT_AS_BOOL`` function is used to implement the cast expr in QASM3 for bit to bool.
/// This already exists in the Q# library, but is defined as a marker for casts from QASM3.
const RESULT_AS_BOOL: &str = "
function __ResultAsBool__(input: Result) : Bool {
    Microsoft.Quantum.Convert.ResultAsBool(input)
}
";

/// The ``RESULT_AS_INT`` function is used to implement the cast expr in QASM3 for bit to bool.
const RESULT_AS_INT: &str = "
function __ResultAsInt__(input: Result) : Int {
    if Microsoft.Quantum.Convert.ResultAsBool(input) {
        1
    } else {
        0
    }
}
";

/// The ``RESULT_AS_BIGINT`` function is used to implement the cast expr in QASM3 for bit to bool.
const RESULT_AS_BIGINT: &str = "
function __ResultAsBigInt__(input: Result) : BigInt {
    if Microsoft.Quantum.Convert.ResultAsBool(input) {
        1L
    } else {
        0L
    }
}
";

/// The ``INT_AS_RESULT_ARRAY_BE`` function is used to implement the cast expr in QASM3 for int to bit[].
/// with big-endian order. This is needed for round-trip conversion for bin ops.
const INT_AS_RESULT_ARRAY_BE: &str = "
function __IntAsResultArrayBE__(number : Int, bits : Int) : Result[] {
    mutable runningValue = number;
    mutable result = [];
    for _ in 1..bits {
        set result += [__BoolAsResult__((runningValue &&& 1) != 0)];
        set runningValue >>>= 1;
    }
    Microsoft.Quantum.Arrays.Reversed(result)
}
";

/// The ``RESULT_ARRAY_AS_INT_BE`` function is used to implement the cast expr in QASM3 for bit[] to uint.
/// with big-endian order. This is needed for round-trip conversion for bin ops.
const RESULT_ARRAY_AS_INT_BE: &str = "
function __ResultArrayAsIntBE__(results : Result[]) : Int {
     Microsoft.Quantum.Convert.ResultArrayAsInt(Microsoft.Quantum.Arrays.Reversed(results))
}
";

/// Runtime functions that are used in the generated AST.
/// Once compilation is complete, we can use this to determine
/// which runtime functions need to be included in the AST.
#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Copy)]
pub struct RuntimeFunctions(u16);

bitflags! {
    impl RuntimeFunctions: u16 {
        const Pow = 0b1;
        const Barrier = 0b10;
        const BoolAsResult = 0b100;
        const BoolAsInt = 0b1_000;
        const BoolAsBigInt = 0b10_000;
        const BoolAsDouble = 0b100_000;
        const ResultAsBool = 0b1_000_000;
        const ResultAsInt = 0b10_000_000;
        const ResultAsBigInt = 0b100_000_000;
        /// IntAsResultArray requires BoolAsResult to be included.
        const IntAsResultArrayBE = 0b1_000_000_000 | 0b100;
        const ResultArrayAsIntBE = 0b10_000_000_000;
    }
}

impl Default for RuntimeFunctions {
    fn default() -> Self {
        RuntimeFunctions::empty()
    }
}

pub(crate) fn get_pow_decl() -> Stmt {
    parse_stmt(POW)
}

pub(crate) fn get_barrier_decl() -> Stmt {
    parse_stmt(BARRIER)
}

pub(crate) fn get_bool_as_result_decl() -> Stmt {
    parse_stmt(BOOL_AS_RESULT)
}

pub(crate) fn get_bool_as_int_decl() -> Stmt {
    parse_stmt(BOOL_AS_INT)
}

pub(crate) fn get_bool_as_bigint_decl() -> Stmt {
    parse_stmt(BOOL_AS_BIGINT)
}

pub(crate) fn get_bool_as_double_decl() -> Stmt {
    parse_stmt(BOOL_AS_DOUBLE)
}

pub(crate) fn get_int_as_result_array_be_decl() -> Stmt {
    parse_stmt(INT_AS_RESULT_ARRAY_BE)
}

pub(crate) fn get_result_as_bool_decl() -> Stmt {
    parse_stmt(RESULT_AS_BOOL)
}

pub(crate) fn get_result_as_bigint_decl() -> Stmt {
    parse_stmt(RESULT_AS_BIGINT)
}

pub(crate) fn get_result_as_int_decl() -> Stmt {
    parse_stmt(RESULT_AS_INT)
}

pub(crate) fn get_result_array_as_int_be_decl() -> Stmt {
    parse_stmt(RESULT_ARRAY_AS_INT_BE)
}

fn parse_stmt(name: &str) -> Stmt {
    let (nodes, errors) = qsc::parse::top_level_nodes(name, LanguageFeatures::default());
    assert!(errors.is_empty(), "Failed to parse POW: {errors:?}");
    assert!(
        nodes.len() == 1,
        "Expected one top-level node, found {:?}",
        nodes.len()
    );
    match nodes.into_iter().next().expect("no top-level nodes found") {
        TopLevelNode::Namespace(..) => {
            panic!("Expected operation, got Namespace")
        }
        TopLevelNode::Stmt(stmt) => *stmt,
    }
}

pub(crate) fn get_runtime_function_decls(runtime: RuntimeFunctions) -> Vec<Stmt> {
    let mut stmts = vec![];
    if runtime.contains(RuntimeFunctions::Pow) {
        let stmt = crate::runtime::get_pow_decl();
        stmts.push(stmt);
    }
    if runtime.contains(RuntimeFunctions::Barrier) {
        let stmt = crate::runtime::get_barrier_decl();
        stmts.push(stmt);
    }
    if runtime.contains(RuntimeFunctions::BoolAsBigInt) {
        let stmt = crate::runtime::get_bool_as_bigint_decl();
        stmts.push(stmt);
    }
    if runtime.contains(RuntimeFunctions::BoolAsDouble) {
        let stmt = crate::runtime::get_bool_as_double_decl();
        stmts.push(stmt);
    }
    if runtime.contains(RuntimeFunctions::BoolAsInt) {
        let stmt = crate::runtime::get_bool_as_int_decl();
        stmts.push(stmt);
    }
    if runtime.contains(RuntimeFunctions::BoolAsResult) {
        let stmt = crate::runtime::get_bool_as_result_decl();
        stmts.push(stmt);
    }
    if runtime.contains(RuntimeFunctions::IntAsResultArrayBE) {
        let stmt = crate::runtime::get_int_as_result_array_be_decl();
        stmts.push(stmt);
    }
    if runtime.contains(RuntimeFunctions::ResultAsBool) {
        let stmt = crate::runtime::get_result_as_bool_decl();
        stmts.push(stmt);
    }
    if runtime.contains(RuntimeFunctions::ResultAsBigInt) {
        let stmt = crate::runtime::get_result_as_bigint_decl();
        stmts.push(stmt);
    }
    if runtime.contains(RuntimeFunctions::ResultAsInt) {
        let stmt = crate::runtime::get_result_as_int_decl();
        stmts.push(stmt);
    }
    if runtime.contains(RuntimeFunctions::ResultArrayAsIntBE) {
        let stmt = crate::runtime::get_result_array_as_int_be_decl();
        stmts.push(stmt);
    }
    stmts
}
