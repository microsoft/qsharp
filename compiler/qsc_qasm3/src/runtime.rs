// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Q# doesn't support gphase and U gates which are used in QASM3.
//! We provide the implementation of these gates here so that QASM3
//! users can still define custom gates in terms of these operations.
//!
//! We also provide runtime functions that are used in the generated AST.
//! These functions are not part of the QASM3 standard, but are used to implement
//! utility fuctions that would be cumbersome to implement building the AST
//! directly.
//!
//! Finally, we provide QASM3 runtime functions mapped to their Q# counterparts.

use bitflags::bitflags;

/// Runtime functions that are used in the generated AST.
/// Once compilation is complete, we can use this to determine
/// which runtime functions need to be included in the AST.
#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Copy)]
pub struct RuntimeFunctions(u16);

bitflags! {
    impl RuntimeFunctions: u16 {
        const Pow                =                     0b1;
        const Barrier            =                    0b10;
        const BoolAsResult       =                   0b100;
        const BoolAsInt          =                  0b1000;
        const BoolAsBigInt       =                0b1_0000;
        const BoolAsDouble       =               0b10_0000;
        const ResultAsBool       =              0b100_0000;
        const ResultAsInt        =             0b1000_0000;
        const ResultAsBigInt     =           0b1_0000_0000;
        /// IntAsResultArray requires BoolAsResult to be included.
        const IntAsResultArrayBE =          0b10_0000_0000 | 0b100;
        const ResultArrayAsIntBE =         0b100_0000_0000;
        const GATES              =        0b1000_0000_0000;
    }
}

impl Default for RuntimeFunctions {
    fn default() -> Self {
        RuntimeFunctions::empty()
    }
}
