// Portions copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::fmt::{self, Display};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum IntPredicate {
    EQ,
    NE,
    UGT,
    UGE,
    ULT,
    ULE,
    SGT,
    SGE,
    SLT,
    SLE,
}

impl Display for IntPredicate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IntPredicate::EQ => write!(f, "eq"),
            IntPredicate::NE => write!(f, "ne"),
            IntPredicate::UGT => write!(f, "ugt"),
            IntPredicate::UGE => write!(f, "uge"),
            IntPredicate::ULT => write!(f, "ult"),
            IntPredicate::ULE => write!(f, "ule"),
            IntPredicate::SGT => write!(f, "sgt"),
            IntPredicate::SGE => write!(f, "sge"),
            IntPredicate::SLT => write!(f, "slt"),
            IntPredicate::SLE => write!(f, "sle"),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum FPPredicate {
    False,
    OEQ,
    OGT,
    OGE,
    OLT,
    OLE,
    ONE,
    ORD,
    UNO,
    UEQ,
    UGT,
    UGE,
    ULT,
    ULE,
    UNE,
    True,
}

impl Display for FPPredicate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FPPredicate::False => write!(f, "false"),
            FPPredicate::OEQ => write!(f, "oeq"),
            FPPredicate::OGT => write!(f, "ogt"),
            FPPredicate::OGE => write!(f, "oge"),
            FPPredicate::OLT => write!(f, "olt"),
            FPPredicate::OLE => write!(f, "ole"),
            FPPredicate::ONE => write!(f, "one"),
            FPPredicate::ORD => write!(f, "ord"),
            FPPredicate::UNO => write!(f, "uno"),
            FPPredicate::UEQ => write!(f, "ueq"),
            FPPredicate::UGT => write!(f, "ugt"),
            FPPredicate::UGE => write!(f, "uge"),
            FPPredicate::ULT => write!(f, "ult"),
            FPPredicate::ULE => write!(f, "ule"),
            FPPredicate::UNE => write!(f, "une"),
            FPPredicate::True => write!(f, "true"),
        }
    }
}
