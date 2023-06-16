// Copyright (c) 2019 Craig Disselkoen
// Portions copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::fmt::{Display, Formatter, Result};

use super::instruction::Instruction;
use super::name::Name;
use super::terminator::Terminator;

/// A `BasicBlock` is a sequence of zero or more non-terminator instructions
/// followed by a single terminator instruction which ends the block.
/// Basic blocks are discussed in the [LLVM 14 docs on Functions](https://releases.llvm.org/14.0.0/docs/LangRef.html#functionstructure)
#[derive(PartialEq, Clone, Debug)]
pub struct BasicBlock {
    pub name: Name,
    pub instrs: Vec<Instruction>,
    pub term: Terminator,
}

impl BasicBlock {
    /// A `BasicBlock` instance with no instructions and an `Unreachable` terminator
    #[must_use]
    pub fn new(name: Name) -> Self {
        use super::terminator::Unreachable;
        Self {
            name,
            instrs: vec![],
            term: Terminator::Unreachable(Unreachable { debugloc: None }),
        }
    }
}

impl Display for BasicBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(
            f,
            "{}:",
            match &self.name {
                Name::Name(name) => name.to_string(),
                Name::Number(num) => num.to_string(),
            }
        )?;
        for i in &self.instrs {
            writeln!(f, "  {i}")?;
        }
        writeln!(f, "  {}", self.term)
    }
}
