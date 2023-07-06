// Copyright (c) 2019 Craig Disselkoen
// Portions copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::debugloc::{DebugLoc, HasDebugLoc};
use super::module::Linkage;
use super::types::TypeRef;
use super::{BasicBlock, Name};
use std::fmt::{Display, Formatter, Result};

/// See [LLVM 14 docs on Functions](https://releases.llvm.org/14.0.0/docs/LangRef.html#functions)
#[derive(PartialEq, Clone, Debug)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub is_var_arg: bool,
    pub return_type: TypeRef,
    pub basic_blocks: Vec<BasicBlock>,
    pub function_attributes: Vec<Attribute>,
    pub linkage: Linkage,
    pub debugloc: Option<DebugLoc>,
}

impl HasDebugLoc for Function {
    fn get_debug_loc(&self) -> &Option<DebugLoc> {
        &self.debugloc
    }
}

impl Function {
    /// Get the `BasicBlock` having the given `Name` (if any).
    #[must_use]
    pub fn get_bb_by_name(&self, name: &Name) -> Option<&BasicBlock> {
        self.basic_blocks.iter().find(|bb| &bb.name == name)
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "define {} {} @{}(",
            self.linkage, self.return_type, self.name
        )?;
        if let Some((last, most)) = self.parameters.split_last() {
            for param in most {
                write!(f, "{param}")?;
                write!(f, ", ")?;
            }
            write!(f, "{last}")?;
        }
        writeln!(f, ") {{")?;
        for bb in &self.basic_blocks {
            write!(f, "{bb}")?;
        }

        writeln!(f, "}}")
    }
}

/// See [LLVM 14 docs on Functions](https://releases.llvm.org/14.0.0/docs/LangRef.html#functions)
#[derive(PartialEq, Clone, Debug)]
pub struct Declaration {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub is_var_arg: bool,
    pub return_type: TypeRef,
    pub linkage: Linkage,
    pub debugloc: Option<DebugLoc>,
}

impl Display for Declaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "declare {} @{}(", self.return_type, self.name)?;
        if let Some((last, most)) = self.parameters.split_last() {
            for param in most {
                write!(f, "{param}")?;
                write!(f, ", ")?;
            }
            write!(f, "{last}")?;
        }
        writeln!(f, ")")
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Parameter {
    pub name: Option<Name>,
    pub ty: TypeRef,
}

impl Display for Parameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.ty)?;
        if let Some(name) = &self.name {
            write!(f, " {name}")?;
        }
        Ok(())
    }
}

/// See [LLVM 14 docs on Function Attributes](https://releases.llvm.org/14.0.0/docs/LangRef.html#fnattrs)
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Attribute {
    StringAttribute { kind: String, value: String },
}

pub type GroupID = usize;
