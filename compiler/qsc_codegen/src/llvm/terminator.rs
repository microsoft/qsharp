// Copyright (c) 2019 Craig Disselkoen
// Portions copyright (c) Microsoft Corporation.
// Licensed under the MIT License.
use super::debugloc::{DebugLoc, HasDebugLoc};
use super::{ConstantRef, Name, Operand};
use std::convert::TryFrom;
use std::fmt::{self, Display};

/// Terminator instructions end a basic block.
/// See [LLVM 14 docs on Terminator Instructions](https://releases.llvm.org/14.0.0/docs/LangRef.html#terminator-instructions)
#[derive(PartialEq, Clone, Debug)]
pub enum Terminator {
    Ret(Ret),
    Br(Br),
    CondBr(CondBr),
    Switch(Switch),
    Unreachable(Unreachable),
}

impl HasDebugLoc for Terminator {
    fn get_debug_loc(&self) -> &Option<DebugLoc> {
        match self {
            Terminator::Ret(t) => t.get_debug_loc(),
            Terminator::Br(t) => t.get_debug_loc(),
            Terminator::CondBr(t) => t.get_debug_loc(),
            Terminator::Switch(t) => t.get_debug_loc(),
            Terminator::Unreachable(t) => t.get_debug_loc(),
        }
    }
}

impl Display for Terminator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Terminator::Ret(t) => write!(f, "{t}"),
            Terminator::Br(t) => write!(f, "{t}"),
            Terminator::CondBr(t) => write!(f, "{t}"),
            Terminator::Switch(t) => write!(f, "{t}"),
            Terminator::Unreachable(t) => write!(f, "{t}"),
        }
    }
}

macro_rules! impl_term {
    ($term:ty, $id:ident) => {
        impl From<$term> for Terminator {
            fn from(term: $term) -> Terminator {
                Terminator::$id(term)
            }
        }

        impl TryFrom<Terminator> for $term {
            type Error = &'static str;
            fn try_from(term: Terminator) -> Result<Self, Self::Error> {
                match term {
                    Terminator::$id(term) => Ok(term),
                    _ => Err("Terminator is not of requested type"),
                }
            }
        }

        impl HasDebugLoc for $term {
            fn get_debug_loc(&self) -> &Option<DebugLoc> {
                &self.debugloc
            }
        }
    };
}

/// See [LLVM 14 docs on the 'ret' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#ret-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Ret {
    /// The value being returned, or `None` if returning void.
    pub return_operand: Option<Operand>,
    pub debugloc: Option<DebugLoc>,
}

impl_term!(Ret, Ret);

impl Display for Ret {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ret {}",
            match &self.return_operand {
                None => "void".into(),
                Some(op) => format!("{op}"),
            },
        )?;
        // if self.debugloc.is_some() {
        // write!(f, " (with debugloc)")?;
        // }
        Ok(())
    }
}

/// See [LLVM 14 docs on the 'br' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#br-instruction).
/// The LLVM 'br' instruction has both conditional and unconditional variants, which we separate -- this is
/// the unconditional variant, while the conditional variant is [`CondBr`](struct.CondBr.html).
#[derive(PartialEq, Clone, Debug)]
pub struct Br {
    /// The [`Name`](../enum.Name.html) of the [`BasicBlock`](../struct.BasicBlock.html) destination.
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_term!(Br, Br);

impl Display for Br {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "br label {}", &self.dest)?;
        // if self.debugloc.is_some() {
        // write!(f, " (with debugloc)")?;
        // }
        Ok(())
    }
}

/// See [LLVM 14 docs on the 'br' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#br-instruction).
/// The LLVM 'br' instruction has both conditional and unconditional variants, which we separate -- this is
/// the conditional variant, while the unconditional variant is [`Br`](struct.Br.html).
#[derive(PartialEq, Clone, Debug)]
pub struct CondBr {
    /// The branch condition.
    pub condition: Operand,
    /// The [`Name`](../enum.Name.html) of the [`BasicBlock`](../struct.BasicBlock.html) destination if the `condition` is true.
    pub true_dest: Name,
    /// The [`Name`](../enum.Name.html) of the [`BasicBlock`](../struct.BasicBlock.html) destination if the `condition` is false.
    pub false_dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_term!(CondBr, CondBr);

impl Display for CondBr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "br {}, label {}, label {}",
            &self.condition, &self.true_dest, &self.false_dest,
        )?;
        // if self.debugloc.is_some() {
        // write!(f, " (with debugloc)")?;
        // }
        Ok(())
    }
}

/// See [LLVM 14 docs on the 'switch' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#switch-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Switch {
    pub operand: Operand,
    pub dests: Vec<(ConstantRef, Name)>,
    pub default_dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_term!(Switch, Switch);

impl Display for Switch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "switch {}, label {} [ ",
            &self.operand, &self.default_dest,
        )?;
        for (val, label) in &self.dests {
            write!(f, "{val}, label {label}; ")?;
        }
        write!(f, "]")?;
        // if self.debugloc.is_some() {
        // write!(f, " (with debugloc)")?;
        // }
        Ok(())
    }
}

/// See [LLVM 14 docs on the 'unreachable' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#unreachable-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Unreachable {
    pub debugloc: Option<DebugLoc>,
}

impl_term!(Unreachable, Unreachable);

impl Display for Unreachable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unreachable")?;
        // if self.debugloc.is_some() {
        // write!(f, " (with debugloc)")?;
        // }
        Ok(())
    }
}
