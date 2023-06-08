// Portions copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{HasResult, Instruction, Name, Operand, TypeRef, Typed};
use crate::types::Types;
use std::convert::TryFrom;

/// Just the `BinaryOps`.  This ends up being better than a `&dyn `[`BinaryOp`](../trait.BinaryOp.html) for various reasons.
#[derive(PartialEq, Clone, Debug)]
pub enum BinaryOp {
    // Integer binary ops
    Add(super::Add),
    Sub(super::Sub),
    Mul(super::Mul),
    UDiv(super::UDiv),
    SDiv(super::SDiv),
    URem(super::URem),
    SRem(super::SRem),

    // Bitwise binary ops
    And(super::And),
    Or(super::Or),
    Xor(super::Xor),
    Shl(super::Shl),
    LShr(super::LShr),
    AShr(super::AShr),

    // Floating-point binary ops
    FAdd(super::FAdd),
    FSub(super::FSub),
    FMul(super::FMul),
    FDiv(super::FDiv),
    FRem(super::FRem),
}

/// Just the `UnaryOps`.  This ends up being better than a `&dyn `[`UnaryOp`](../trait.UnaryOp.html) for various reasons.
#[derive(PartialEq, Clone, Debug)]
pub enum UnaryOp {
    // listed alphabetically
    AddrSpaceCast(super::AddrSpaceCast),
    BitCast(super::BitCast),
    FNeg(super::FNeg),
    FPExt(super::FPExt),
    FPToSI(super::FPToSI),
    FPToUI(super::FPToUI),
    FPTrunc(super::FPTrunc),
    Freeze(super::Freeze),
    IntToPtr(super::IntToPtr),
    PtrToInt(super::PtrToInt),
    SExt(super::SExt),
    SIToFP(super::SIToFP),
    Trunc(super::Trunc),
    UIToFP(super::UIToFP),
    ZExt(super::ZExt),
}

impl From<BinaryOp> for Instruction {
    fn from(bo: BinaryOp) -> Instruction {
        match bo {
            BinaryOp::Add(i) => i.into(),
            BinaryOp::Sub(i) => i.into(),
            BinaryOp::Mul(i) => i.into(),
            BinaryOp::UDiv(i) => i.into(),
            BinaryOp::SDiv(i) => i.into(),
            BinaryOp::URem(i) => i.into(),
            BinaryOp::SRem(i) => i.into(),
            BinaryOp::And(i) => i.into(),
            BinaryOp::Or(i) => i.into(),
            BinaryOp::Xor(i) => i.into(),
            BinaryOp::Shl(i) => i.into(),
            BinaryOp::LShr(i) => i.into(),
            BinaryOp::AShr(i) => i.into(),
            BinaryOp::FAdd(i) => i.into(),
            BinaryOp::FSub(i) => i.into(),
            BinaryOp::FMul(i) => i.into(),
            BinaryOp::FDiv(i) => i.into(),
            BinaryOp::FRem(i) => i.into(),
        }
    }
}

impl From<UnaryOp> for Instruction {
    fn from(uo: UnaryOp) -> Instruction {
        match uo {
            UnaryOp::AddrSpaceCast(i) => i.into(),
            UnaryOp::BitCast(i) => i.into(),
            UnaryOp::FNeg(i) => i.into(),
            UnaryOp::FPExt(i) => i.into(),
            UnaryOp::FPToSI(i) => i.into(),
            UnaryOp::FPToUI(i) => i.into(),
            UnaryOp::FPTrunc(i) => i.into(),
            UnaryOp::Freeze(i) => i.into(),
            UnaryOp::IntToPtr(i) => i.into(),
            UnaryOp::PtrToInt(i) => i.into(),
            UnaryOp::SExt(i) => i.into(),
            UnaryOp::SIToFP(i) => i.into(),
            UnaryOp::Trunc(i) => i.into(),
            UnaryOp::UIToFP(i) => i.into(),
            UnaryOp::ZExt(i) => i.into(),
        }
    }
}

impl TryFrom<Instruction> for BinaryOp {
    type Error = &'static str;
    fn try_from(inst: Instruction) -> Result<Self, Self::Error> {
        match inst {
            Instruction::Add(i) => Ok(BinaryOp::Add(i)),
            Instruction::Sub(i) => Ok(BinaryOp::Sub(i)),
            Instruction::Mul(i) => Ok(BinaryOp::Mul(i)),
            Instruction::UDiv(i) => Ok(BinaryOp::UDiv(i)),
            Instruction::SDiv(i) => Ok(BinaryOp::SDiv(i)),
            Instruction::URem(i) => Ok(BinaryOp::URem(i)),
            Instruction::SRem(i) => Ok(BinaryOp::SRem(i)),
            Instruction::And(i) => Ok(BinaryOp::And(i)),
            Instruction::Or(i) => Ok(BinaryOp::Or(i)),
            Instruction::Xor(i) => Ok(BinaryOp::Xor(i)),
            Instruction::Shl(i) => Ok(BinaryOp::Shl(i)),
            Instruction::LShr(i) => Ok(BinaryOp::LShr(i)),
            Instruction::AShr(i) => Ok(BinaryOp::AShr(i)),
            Instruction::FAdd(i) => Ok(BinaryOp::FAdd(i)),
            Instruction::FSub(i) => Ok(BinaryOp::FSub(i)),
            Instruction::FMul(i) => Ok(BinaryOp::FMul(i)),
            Instruction::FDiv(i) => Ok(BinaryOp::FDiv(i)),
            Instruction::FRem(i) => Ok(BinaryOp::FRem(i)),
            _ => Err("Not a binary op"),
        }
    }
}

impl TryFrom<Instruction> for UnaryOp {
    type Error = &'static str;
    fn try_from(inst: Instruction) -> Result<Self, Self::Error> {
        match inst {
            Instruction::AddrSpaceCast(i) => Ok(UnaryOp::AddrSpaceCast(i)),
            Instruction::BitCast(i) => Ok(UnaryOp::BitCast(i)),
            Instruction::FNeg(i) => Ok(UnaryOp::FNeg(i)),
            Instruction::FPExt(i) => Ok(UnaryOp::FPExt(i)),
            Instruction::FPToSI(i) => Ok(UnaryOp::FPToSI(i)),
            Instruction::FPToUI(i) => Ok(UnaryOp::FPToUI(i)),
            Instruction::FPTrunc(i) => Ok(UnaryOp::FPTrunc(i)),
            Instruction::Freeze(i) => Ok(UnaryOp::Freeze(i)),
            Instruction::IntToPtr(i) => Ok(UnaryOp::IntToPtr(i)),
            Instruction::PtrToInt(i) => Ok(UnaryOp::PtrToInt(i)),
            Instruction::SExt(i) => Ok(UnaryOp::SExt(i)),
            Instruction::SIToFP(i) => Ok(UnaryOp::SIToFP(i)),
            Instruction::Trunc(i) => Ok(UnaryOp::Trunc(i)),
            Instruction::UIToFP(i) => Ok(UnaryOp::UIToFP(i)),
            Instruction::ZExt(i) => Ok(UnaryOp::ZExt(i)),
            _ => Err("Not a unary op"),
        }
    }
}

impl Typed for BinaryOp {
    fn get_type(&self, types: &Types) -> TypeRef {
        match self {
            BinaryOp::Add(i) => types.type_of(i),
            BinaryOp::Sub(i) => types.type_of(i),
            BinaryOp::Mul(i) => types.type_of(i),
            BinaryOp::UDiv(i) => types.type_of(i),
            BinaryOp::SDiv(i) => types.type_of(i),
            BinaryOp::URem(i) => types.type_of(i),
            BinaryOp::SRem(i) => types.type_of(i),
            BinaryOp::And(i) => types.type_of(i),
            BinaryOp::Or(i) => types.type_of(i),
            BinaryOp::Xor(i) => types.type_of(i),
            BinaryOp::Shl(i) => types.type_of(i),
            BinaryOp::LShr(i) => types.type_of(i),
            BinaryOp::AShr(i) => types.type_of(i),
            BinaryOp::FAdd(i) => types.type_of(i),
            BinaryOp::FSub(i) => types.type_of(i),
            BinaryOp::FMul(i) => types.type_of(i),
            BinaryOp::FDiv(i) => types.type_of(i),
            BinaryOp::FRem(i) => types.type_of(i),
        }
    }
}

impl Typed for UnaryOp {
    fn get_type(&self, types: &Types) -> TypeRef {
        match self {
            UnaryOp::AddrSpaceCast(i) => types.type_of(i),
            UnaryOp::BitCast(i) => types.type_of(i),
            UnaryOp::FNeg(i) => types.type_of(i),
            UnaryOp::FPExt(i) => types.type_of(i),
            UnaryOp::FPToSI(i) => types.type_of(i),
            UnaryOp::FPToUI(i) => types.type_of(i),
            UnaryOp::FPTrunc(i) => types.type_of(i),
            UnaryOp::Freeze(i) => types.type_of(i),
            UnaryOp::IntToPtr(i) => types.type_of(i),
            UnaryOp::PtrToInt(i) => types.type_of(i),
            UnaryOp::SExt(i) => types.type_of(i),
            UnaryOp::SIToFP(i) => types.type_of(i),
            UnaryOp::Trunc(i) => types.type_of(i),
            UnaryOp::UIToFP(i) => types.type_of(i),
            UnaryOp::ZExt(i) => types.type_of(i),
        }
    }
}

/* --TODO not yet implemented: metadata
impl HasMetadata for BinaryOp {
    fn get_metadata(&self) -> &InstructionMetadata {
        match self {
            BinaryOp::Add(i) => i.get_metadata(),
            BinaryOp::Sub(i) => i.get_metadata(),
            BinaryOp::Mul(i) => i.get_metadata(),
            BinaryOp::UDiv(i) => i.get_metadata(),
            BinaryOp::SDiv(i) => i.get_metadata(),
            BinaryOp::URem(i) => i.get_metadata(),
            BinaryOp::SRem(i) => i.get_metadata(),
            BinaryOp::And(i) => i.get_metadata(),
            BinaryOp::Or(i) => i.get_metadata(),
            BinaryOp::Xor(i) => i.get_metadata(),
            BinaryOp::Shl(i) => i.get_metadata(),
            BinaryOp::LShr(i) => i.get_metadata(),
            BinaryOp::AShr(i) => i.get_metadata(),
            BinaryOp::FAdd(i) => i.get_metadata(),
            BinaryOp::FSub(i) => i.get_metadata(),
            BinaryOp::FMul(i) => i.get_metadata(),
            BinaryOp::FDiv(i) => i.get_metadata(),
            BinaryOp::FRem(i) => i.get_metadata(),
        }
    }
}

impl HasMetadata for UnaryOp {
    fn get_metadata(&self) -> &InstructionMetadata {
        match self {
            UnaryOp::AddrSpaceCast(i) => i.get_metadata(),
            UnaryOp::BitCast(i) => i.get_metadata(),
            UnaryOp::FNeg(i) => i.get_metadata(),
            UnaryOp::FPExt(i) => i.get_metadata(),
            UnaryOp::FPToSI(i) => i.get_metadata(),
            UnaryOp::FPToUI(i) => i.get_metadata(),
            UnaryOp::FPTrunc(i) => i.get_metadata(),
            #[cfg(feature="llvm-10-or-greater")]
            UnaryOp::Freeze(i) => i.get_metadata(),
            UnaryOp::IntToPtr(i) => i.get_metadata(),
            UnaryOp::PtrToInt(i) => i.get_metadata(),
            UnaryOp::SExt(i) => i.get_metadata(),
            UnaryOp::SIToFP(i) => i.get_metadata(),
            UnaryOp::Trunc(i) => i.get_metadata(),
            UnaryOp::UIToFP(i) => i.get_metadata(),
            UnaryOp::ZExt(i) => i.get_metadata(),
        }
    }
}
*/

impl HasResult for BinaryOp {
    fn get_result(&self) -> &Name {
        match self {
            BinaryOp::Add(i) => i.get_result(),
            BinaryOp::Sub(i) => i.get_result(),
            BinaryOp::Mul(i) => i.get_result(),
            BinaryOp::UDiv(i) => i.get_result(),
            BinaryOp::SDiv(i) => i.get_result(),
            BinaryOp::URem(i) => i.get_result(),
            BinaryOp::SRem(i) => i.get_result(),
            BinaryOp::And(i) => i.get_result(),
            BinaryOp::Or(i) => i.get_result(),
            BinaryOp::Xor(i) => i.get_result(),
            BinaryOp::Shl(i) => i.get_result(),
            BinaryOp::LShr(i) => i.get_result(),
            BinaryOp::AShr(i) => i.get_result(),
            BinaryOp::FAdd(i) => i.get_result(),
            BinaryOp::FSub(i) => i.get_result(),
            BinaryOp::FMul(i) => i.get_result(),
            BinaryOp::FDiv(i) => i.get_result(),
            BinaryOp::FRem(i) => i.get_result(),
        }
    }
}

impl HasResult for UnaryOp {
    fn get_result(&self) -> &Name {
        match self {
            UnaryOp::AddrSpaceCast(i) => i.get_result(),
            UnaryOp::BitCast(i) => i.get_result(),
            UnaryOp::FNeg(i) => i.get_result(),
            UnaryOp::FPExt(i) => i.get_result(),
            UnaryOp::FPToSI(i) => i.get_result(),
            UnaryOp::FPToUI(i) => i.get_result(),
            UnaryOp::FPTrunc(i) => i.get_result(),
            UnaryOp::Freeze(i) => i.get_result(),
            UnaryOp::IntToPtr(i) => i.get_result(),
            UnaryOp::PtrToInt(i) => i.get_result(),
            UnaryOp::SExt(i) => i.get_result(),
            UnaryOp::SIToFP(i) => i.get_result(),
            UnaryOp::Trunc(i) => i.get_result(),
            UnaryOp::UIToFP(i) => i.get_result(),
            UnaryOp::ZExt(i) => i.get_result(),
        }
    }
}

impl super::BinaryOp for BinaryOp {
    fn get_operand0(&self) -> &Operand {
        match self {
            BinaryOp::Add(i) => i.get_operand0(),
            BinaryOp::Sub(i) => i.get_operand0(),
            BinaryOp::Mul(i) => i.get_operand0(),
            BinaryOp::UDiv(i) => i.get_operand0(),
            BinaryOp::SDiv(i) => i.get_operand0(),
            BinaryOp::URem(i) => i.get_operand0(),
            BinaryOp::SRem(i) => i.get_operand0(),
            BinaryOp::And(i) => i.get_operand0(),
            BinaryOp::Or(i) => i.get_operand0(),
            BinaryOp::Xor(i) => i.get_operand0(),
            BinaryOp::Shl(i) => i.get_operand0(),
            BinaryOp::LShr(i) => i.get_operand0(),
            BinaryOp::AShr(i) => i.get_operand0(),
            BinaryOp::FAdd(i) => i.get_operand0(),
            BinaryOp::FSub(i) => i.get_operand0(),
            BinaryOp::FMul(i) => i.get_operand0(),
            BinaryOp::FDiv(i) => i.get_operand0(),
            BinaryOp::FRem(i) => i.get_operand0(),
        }
    }

    fn get_operand1(&self) -> &Operand {
        match self {
            BinaryOp::Add(i) => i.get_operand1(),
            BinaryOp::Sub(i) => i.get_operand1(),
            BinaryOp::Mul(i) => i.get_operand1(),
            BinaryOp::UDiv(i) => i.get_operand1(),
            BinaryOp::SDiv(i) => i.get_operand1(),
            BinaryOp::URem(i) => i.get_operand1(),
            BinaryOp::SRem(i) => i.get_operand1(),
            BinaryOp::And(i) => i.get_operand1(),
            BinaryOp::Or(i) => i.get_operand1(),
            BinaryOp::Xor(i) => i.get_operand1(),
            BinaryOp::Shl(i) => i.get_operand1(),
            BinaryOp::LShr(i) => i.get_operand1(),
            BinaryOp::AShr(i) => i.get_operand1(),
            BinaryOp::FAdd(i) => i.get_operand1(),
            BinaryOp::FSub(i) => i.get_operand1(),
            BinaryOp::FMul(i) => i.get_operand1(),
            BinaryOp::FDiv(i) => i.get_operand1(),
            BinaryOp::FRem(i) => i.get_operand1(),
        }
    }
}

impl super::UnaryOp for UnaryOp {
    fn get_operand(&self) -> &Operand {
        match self {
            UnaryOp::AddrSpaceCast(i) => i.get_operand(),
            UnaryOp::BitCast(i) => i.get_operand(),
            UnaryOp::FNeg(i) => i.get_operand(),
            UnaryOp::FPExt(i) => i.get_operand(),
            UnaryOp::FPToSI(i) => i.get_operand(),
            UnaryOp::FPToUI(i) => i.get_operand(),
            UnaryOp::FPTrunc(i) => i.get_operand(),
            UnaryOp::Freeze(i) => i.get_operand(),
            UnaryOp::IntToPtr(i) => i.get_operand(),
            UnaryOp::PtrToInt(i) => i.get_operand(),
            UnaryOp::SExt(i) => i.get_operand(),
            UnaryOp::SIToFP(i) => i.get_operand(),
            UnaryOp::Trunc(i) => i.get_operand(),
            UnaryOp::UIToFP(i) => i.get_operand(),
            UnaryOp::ZExt(i) => i.get_operand(),
        }
    }
}
