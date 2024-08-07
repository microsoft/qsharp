// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::{
    ast::{Stmt, TopLevelNode},
    LanguageFeatures,
};

const POW: &str = "
operation __Pow__<'T>(N: Int, op: ('T => Unit is Adj), target : 'T) : Unit is Adj {
    let op = if N > 0 { () => op(target) } else { () => Adjoint op(target) };
    for _ in 1..Microsoft.Quantum.Math.AbsI(N) {
        op()
    }
}
";

const BARRIER: &str = "
@SimulatableIntrinsic()
operation __quantum__qis__barrier__body() : Unit {}
";

use bitflags::bitflags;

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
    let r = qsc::parse::top_level_nodes(POW, LanguageFeatures::default());
    assert!(r.1.is_empty(), "Failed to parse POW: {:?}", r.1);
    assert!(
        r.0.len() == 1,
        "Expected one top-level node, found {:?}",
        r.0.len()
    );
    match r.0.into_iter().next().expect("no top-level nodes found") {
        TopLevelNode::Namespace(..) => {
            panic!("Expected operation, got Namespace")
        }
        TopLevelNode::Stmt(stmt) => *stmt,
    }
}

pub(crate) fn get_barrier_decl() -> Stmt {
    let r = qsc::parse::top_level_nodes(BARRIER, LanguageFeatures::default());
    assert!(r.1.is_empty(), "Failed to parse POW: {:?}", r.1);
    assert!(
        r.0.len() == 1,
        "Expected one top-level node, found {:?}",
        r.0.len()
    );
    match r.0.into_iter().next().expect("no top-level nodes found") {
        TopLevelNode::Namespace(..) => {
            panic!("Expected operation, got Namespace")
        }
        TopLevelNode::Stmt(stmt) => *stmt,
    }
}
