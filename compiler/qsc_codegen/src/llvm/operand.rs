// Portions copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::types::TypeRef;
use super::{Constant, ConstantRef, Name};
use std::fmt::{self, Display};

#[derive(PartialEq, Clone, Debug)]
pub enum Operand {
    /// e.g., `i32 %foo`
    LocalOperand { name: Name, ty: TypeRef },
    /// includes [`GlobalReference`](../constant/enum.Constant.html#variant.GlobalReference) for things like `@foo`
    ConstantOperand(ConstantRef),
}

impl Operand {
    /// Get a reference to the `Constant`, if the operand is a constant;
    /// otherwise, returns `None`.
    ///
    /// This allows nested matching on `Operand`. Instead of the following code
    /// (which doesn't compile because you can't directly match on `ConstantRef`)
    /// ```ignore
    /// if let Operand::ConstantOperand(Constant::Float(Float::Double(val))) = op
    /// ```
    /// you can write this:
    /// ```ignore
    /// if let Some(Constant::Float(Float::Double(val))) = op.as_constant()
    /// ```
    #[must_use]
    pub fn as_constant(&self) -> Option<&Constant> {
        match self {
            Operand::ConstantOperand(cref) => Some(cref),
            Operand::LocalOperand { .. } => None,
        }
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operand::LocalOperand { name, ty } => write!(f, "{ty} {name}"),
            Operand::ConstantOperand(cref) => write!(f, "{}", &cref),
        }
    }
}
