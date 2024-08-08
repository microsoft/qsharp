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

/// The BARRIER function is used to implement the `barrier` statement in QASM3.
/// The `@SimulatableIntrinsic` attribute is used to mark the operation for QIR
/// generation.
/// Q# doesn't support barriers, so this is a no-op. We need to figure out what
/// barriers mean in the context of QIR in the future for better support.
const BARRIER: &str = "
@SimulatableIntrinsic()
operation __quantum__qis__barrier__body() : Unit {}
";

/// Runtime functions that are used in the generated AST.
/// Once compilation is complete, we can use this to determine
/// which runtime functions need to be included in the AST.
#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Copy)]
pub struct RuntimeFunctions(u8);

bitflags! {
    impl RuntimeFunctions: u8 {
        const Pow = 0b1;
        const Barrier = 0b10;
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
