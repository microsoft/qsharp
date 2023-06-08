// Portions copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

pub mod basicblock;
pub use basicblock::BasicBlock;
pub mod constant;
pub use constant::{Constant, ConstantRef};
pub mod debugloc;
pub use debugloc::{DebugLoc, HasDebugLoc};
pub mod function;
pub use function::Function;
pub mod instruction;
pub use instruction::Instruction;
pub mod module;
pub use module::Module;
pub mod name;
pub use name::Name;
pub mod operand;
pub use operand::Operand;
pub mod predicates;
pub use predicates::{FPPredicate, IntPredicate};
pub mod terminator;
pub use terminator::Terminator;
pub mod types;
pub use types::{Type, TypeRef};
