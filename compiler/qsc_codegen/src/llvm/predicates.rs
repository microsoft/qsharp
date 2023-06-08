// Portions copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::fmt::{self, Display};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum IntPredicate {
    Eq,
    Ne,
    Ugt,
    Uge,
    Ult,
    Ule,
    Sgt,
    Sge,
    Slt,
    Sle,
}

impl Display for IntPredicate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IntPredicate::Eq => write!(f, "eq"),
            IntPredicate::Ne => write!(f, "ne"),
            IntPredicate::Ugt => write!(f, "ugt"),
            IntPredicate::Uge => write!(f, "uge"),
            IntPredicate::Ult => write!(f, "ult"),
            IntPredicate::Ule => write!(f, "ule"),
            IntPredicate::Sgt => write!(f, "sgt"),
            IntPredicate::Sge => write!(f, "sge"),
            IntPredicate::Slt => write!(f, "slt"),
            IntPredicate::Sle => write!(f, "sle"),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum FPPredicate {
    False,
    Oeq,
    Ogt,
    Oge,
    Olt,
    Ole,
    One,
    Ord,
    Uno,
    Ueq,
    Ugt,
    Uge,
    Ult,
    Ule,
    Une,
    True,
}

impl Display for FPPredicate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FPPredicate::False => write!(f, "false"),
            FPPredicate::Oeq => write!(f, "oeq"),
            FPPredicate::Ogt => write!(f, "ogt"),
            FPPredicate::Oge => write!(f, "oge"),
            FPPredicate::Olt => write!(f, "olt"),
            FPPredicate::Ole => write!(f, "ole"),
            FPPredicate::One => write!(f, "one"),
            FPPredicate::Ord => write!(f, "ord"),
            FPPredicate::Uno => write!(f, "uno"),
            FPPredicate::Ueq => write!(f, "ueq"),
            FPPredicate::Ugt => write!(f, "ugt"),
            FPPredicate::Uge => write!(f, "uge"),
            FPPredicate::Ult => write!(f, "ult"),
            FPPredicate::Ule => write!(f, "ule"),
            FPPredicate::Une => write!(f, "une"),
            FPPredicate::True => write!(f, "true"),
        }
    }
}
