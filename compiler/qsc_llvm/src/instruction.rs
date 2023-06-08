// Portions copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod groups;

use crate::constant::ConstantRef;
use crate::debugloc::{DebugLoc, HasDebugLoc};
use crate::function::{Attribute, CallingConvention, ParameterAttribute};
use crate::name::Name;
use crate::operand::Operand;
use crate::types::{NamedStructDef, Type, TypeRef, Typed, Types};
use crate::{
    predicates::{FPPredicate, IntPredicate},
    Constant,
};
use either::Either;
use std::convert::TryFrom;
use std::fmt::{self, Debug, Display};

/// Non-terminator instructions.
#[derive(PartialEq, Clone, Debug)]
pub enum Instruction {
    // Integer binary ops
    Add(Add),
    Sub(Sub),
    Mul(Mul),
    UDiv(UDiv),
    SDiv(SDiv),
    URem(URem),
    SRem(SRem),

    // Bitwise binary ops
    And(And),
    Or(Or),
    Xor(Xor),
    Shl(Shl),
    LShr(LShr),
    AShr(AShr),

    // Floating-point ops
    FAdd(FAdd),
    FSub(FSub),
    FMul(FMul),
    FDiv(FDiv),
    FRem(FRem),
    FNeg(FNeg),

    // Vector ops
    ExtractElement(ExtractElement),
    InsertElement(InsertElement),
    ShuffleVector(ShuffleVector),

    // Aggregate ops
    ExtractValue(ExtractValue),
    InsertValue(InsertValue),

    // Memory-related ops
    Alloca(Alloca),
    Load(Load),
    Store(Store),
    Fence(Fence),
    CmpXchg(CmpXchg),
    AtomicRMW(AtomicRMW),
    GetElementPtr(GetElementPtr),

    // Conversion ops
    Trunc(Trunc),
    ZExt(ZExt),
    SExt(SExt),
    FPTrunc(FPTrunc),
    FPExt(FPExt),
    FPToUI(FPToUI),
    FPToSI(FPToSI),
    UIToFP(UIToFP),
    SIToFP(SIToFP),
    PtrToInt(PtrToInt),
    IntToPtr(IntToPtr),
    BitCast(BitCast),
    AddrSpaceCast(AddrSpaceCast),

    // LLVM's "other operations" category
    ICmp(ICmp),
    FCmp(FCmp),
    Phi(Phi),
    Select(Select),
    Freeze(Freeze),
    Call(Call),
    VAArg(VAArg),
    LandingPad(LandingPad),
    CatchPad(CatchPad),
    CleanupPad(CleanupPad),
}

/// The [`Type`](../enum.Type.html) of an `Instruction` (or any subtype of `Instruction`) is its result type.
impl Typed for Instruction {
    fn get_type(&self, types: &Types) -> TypeRef {
        match self {
            Instruction::Add(i) => types.type_of(i),
            Instruction::Sub(i) => types.type_of(i),
            Instruction::Mul(i) => types.type_of(i),
            Instruction::UDiv(i) => types.type_of(i),
            Instruction::SDiv(i) => types.type_of(i),
            Instruction::URem(i) => types.type_of(i),
            Instruction::SRem(i) => types.type_of(i),
            Instruction::And(i) => types.type_of(i),
            Instruction::Or(i) => types.type_of(i),
            Instruction::Xor(i) => types.type_of(i),
            Instruction::Shl(i) => types.type_of(i),
            Instruction::LShr(i) => types.type_of(i),
            Instruction::AShr(i) => types.type_of(i),
            Instruction::FAdd(i) => types.type_of(i),
            Instruction::FSub(i) => types.type_of(i),
            Instruction::FMul(i) => types.type_of(i),
            Instruction::FDiv(i) => types.type_of(i),
            Instruction::FRem(i) => types.type_of(i),
            Instruction::FNeg(i) => types.type_of(i),
            Instruction::ExtractElement(i) => types.type_of(i),
            Instruction::InsertElement(i) => types.type_of(i),
            Instruction::ShuffleVector(i) => types.type_of(i),
            Instruction::ExtractValue(i) => types.type_of(i),
            Instruction::InsertValue(i) => types.type_of(i),
            Instruction::Alloca(i) => types.type_of(i),
            Instruction::Load(i) => types.type_of(i),
            Instruction::Store(i) => types.type_of(i),
            Instruction::Fence(i) => types.type_of(i),
            Instruction::CmpXchg(i) => types.type_of(i),
            Instruction::AtomicRMW(i) => types.type_of(i),
            Instruction::GetElementPtr(i) => types.type_of(i),
            Instruction::Trunc(i) => types.type_of(i),
            Instruction::ZExt(i) => types.type_of(i),
            Instruction::SExt(i) => types.type_of(i),
            Instruction::FPTrunc(i) => types.type_of(i),
            Instruction::FPExt(i) => types.type_of(i),
            Instruction::FPToUI(i) => types.type_of(i),
            Instruction::FPToSI(i) => types.type_of(i),
            Instruction::UIToFP(i) => types.type_of(i),
            Instruction::SIToFP(i) => types.type_of(i),
            Instruction::PtrToInt(i) => types.type_of(i),
            Instruction::IntToPtr(i) => types.type_of(i),
            Instruction::BitCast(i) => types.type_of(i),
            Instruction::AddrSpaceCast(i) => types.type_of(i),
            Instruction::ICmp(i) => types.type_of(i),
            Instruction::FCmp(i) => types.type_of(i),
            Instruction::Phi(i) => types.type_of(i),
            Instruction::Select(i) => types.type_of(i),
            Instruction::Freeze(i) => types.type_of(i),
            Instruction::Call(i) => types.type_of(i),
            Instruction::VAArg(i) => types.type_of(i),
            Instruction::LandingPad(i) => types.type_of(i),
            Instruction::CatchPad(i) => types.type_of(i),
            Instruction::CleanupPad(i) => types.type_of(i),
        }
    }
}
impl HasDebugLoc for Instruction {
    fn get_debug_loc(&self) -> &Option<DebugLoc> {
        match self {
            Instruction::Add(i) => i.get_debug_loc(),
            Instruction::Sub(i) => i.get_debug_loc(),
            Instruction::Mul(i) => i.get_debug_loc(),
            Instruction::UDiv(i) => i.get_debug_loc(),
            Instruction::SDiv(i) => i.get_debug_loc(),
            Instruction::URem(i) => i.get_debug_loc(),
            Instruction::SRem(i) => i.get_debug_loc(),
            Instruction::And(i) => i.get_debug_loc(),
            Instruction::Or(i) => i.get_debug_loc(),
            Instruction::Xor(i) => i.get_debug_loc(),
            Instruction::Shl(i) => i.get_debug_loc(),
            Instruction::LShr(i) => i.get_debug_loc(),
            Instruction::AShr(i) => i.get_debug_loc(),
            Instruction::FAdd(i) => i.get_debug_loc(),
            Instruction::FSub(i) => i.get_debug_loc(),
            Instruction::FMul(i) => i.get_debug_loc(),
            Instruction::FDiv(i) => i.get_debug_loc(),
            Instruction::FRem(i) => i.get_debug_loc(),
            Instruction::FNeg(i) => i.get_debug_loc(),
            Instruction::ExtractElement(i) => i.get_debug_loc(),
            Instruction::InsertElement(i) => i.get_debug_loc(),
            Instruction::ShuffleVector(i) => i.get_debug_loc(),
            Instruction::ExtractValue(i) => i.get_debug_loc(),
            Instruction::InsertValue(i) => i.get_debug_loc(),
            Instruction::Alloca(i) => i.get_debug_loc(),
            Instruction::Load(i) => i.get_debug_loc(),
            Instruction::Store(i) => i.get_debug_loc(),
            Instruction::Fence(i) => i.get_debug_loc(),
            Instruction::CmpXchg(i) => i.get_debug_loc(),
            Instruction::AtomicRMW(i) => i.get_debug_loc(),
            Instruction::GetElementPtr(i) => i.get_debug_loc(),
            Instruction::Trunc(i) => i.get_debug_loc(),
            Instruction::ZExt(i) => i.get_debug_loc(),
            Instruction::SExt(i) => i.get_debug_loc(),
            Instruction::FPTrunc(i) => i.get_debug_loc(),
            Instruction::FPExt(i) => i.get_debug_loc(),
            Instruction::FPToUI(i) => i.get_debug_loc(),
            Instruction::FPToSI(i) => i.get_debug_loc(),
            Instruction::UIToFP(i) => i.get_debug_loc(),
            Instruction::SIToFP(i) => i.get_debug_loc(),
            Instruction::PtrToInt(i) => i.get_debug_loc(),
            Instruction::IntToPtr(i) => i.get_debug_loc(),
            Instruction::BitCast(i) => i.get_debug_loc(),
            Instruction::AddrSpaceCast(i) => i.get_debug_loc(),
            Instruction::ICmp(i) => i.get_debug_loc(),
            Instruction::FCmp(i) => i.get_debug_loc(),
            Instruction::Phi(i) => i.get_debug_loc(),
            Instruction::Select(i) => i.get_debug_loc(),
            Instruction::Freeze(i) => i.get_debug_loc(),
            Instruction::Call(i) => i.get_debug_loc(),
            Instruction::VAArg(i) => i.get_debug_loc(),
            Instruction::LandingPad(i) => i.get_debug_loc(),
            Instruction::CatchPad(i) => i.get_debug_loc(),
            Instruction::CleanupPad(i) => i.get_debug_loc(),
        }
    }
}

impl Instruction {
    /// Get the result (destination) of the `Instruction`, or `None` if the
    /// `Instruction` doesn't have a result (has void type).
    #[must_use]
    pub fn try_get_result(&self) -> Option<&Name> {
        match self {
            Instruction::Add(i) => Some(&i.dest),
            Instruction::Sub(i) => Some(&i.dest),
            Instruction::Mul(i) => Some(&i.dest),
            Instruction::UDiv(i) => Some(&i.dest),
            Instruction::SDiv(i) => Some(&i.dest),
            Instruction::URem(i) => Some(&i.dest),
            Instruction::SRem(i) => Some(&i.dest),
            Instruction::And(i) => Some(&i.dest),
            Instruction::Or(i) => Some(&i.dest),
            Instruction::Xor(i) => Some(&i.dest),
            Instruction::Shl(i) => Some(&i.dest),
            Instruction::LShr(i) => Some(&i.dest),
            Instruction::AShr(i) => Some(&i.dest),
            Instruction::FAdd(i) => Some(&i.dest),
            Instruction::FSub(i) => Some(&i.dest),
            Instruction::FMul(i) => Some(&i.dest),
            Instruction::FDiv(i) => Some(&i.dest),
            Instruction::FRem(i) => Some(&i.dest),
            Instruction::FNeg(i) => Some(&i.dest),
            Instruction::ExtractElement(i) => Some(&i.dest),
            Instruction::InsertElement(i) => Some(&i.dest),
            Instruction::ShuffleVector(i) => Some(&i.dest),
            Instruction::ExtractValue(i) => Some(&i.dest),
            Instruction::InsertValue(i) => Some(&i.dest),
            Instruction::Alloca(i) => Some(&i.dest),
            Instruction::Load(i) => Some(&i.dest),
            Instruction::Store(_) | Instruction::Fence(_) => None,
            Instruction::CmpXchg(i) => Some(&i.dest),
            Instruction::AtomicRMW(i) => Some(&i.dest),
            Instruction::GetElementPtr(i) => Some(&i.dest),
            Instruction::Trunc(i) => Some(&i.dest),
            Instruction::ZExt(i) => Some(&i.dest),
            Instruction::SExt(i) => Some(&i.dest),
            Instruction::FPTrunc(i) => Some(&i.dest),
            Instruction::FPExt(i) => Some(&i.dest),
            Instruction::FPToUI(i) => Some(&i.dest),
            Instruction::FPToSI(i) => Some(&i.dest),
            Instruction::UIToFP(i) => Some(&i.dest),
            Instruction::SIToFP(i) => Some(&i.dest),
            Instruction::PtrToInt(i) => Some(&i.dest),
            Instruction::IntToPtr(i) => Some(&i.dest),
            Instruction::BitCast(i) => Some(&i.dest),
            Instruction::AddrSpaceCast(i) => Some(&i.dest),
            Instruction::ICmp(i) => Some(&i.dest),
            Instruction::FCmp(i) => Some(&i.dest),
            Instruction::Phi(i) => Some(&i.dest),
            Instruction::Select(i) => Some(&i.dest),
            Instruction::Freeze(i) => Some(&i.dest),
            Instruction::Call(i) => i.dest.as_ref(),
            Instruction::VAArg(i) => Some(&i.dest),
            Instruction::LandingPad(i) => Some(&i.dest),
            Instruction::CatchPad(i) => Some(&i.dest),
            Instruction::CleanupPad(i) => Some(&i.dest),
        }
    }

    /// Whether the `Instruction` is atomic
    #[must_use]
    pub fn is_atomic(&self) -> bool {
        match self {
            Instruction::Add(_)
            | Instruction::Sub(_)
            | Instruction::Mul(_)
            | Instruction::UDiv(_)
            | Instruction::SDiv(_)
            | Instruction::URem(_)
            | Instruction::SRem(_)
            | Instruction::And(_)
            | Instruction::Or(_)
            | Instruction::Xor(_)
            | Instruction::Shl(_)
            | Instruction::LShr(_)
            | Instruction::AShr(_)
            | Instruction::FAdd(_)
            | Instruction::FSub(_)
            | Instruction::FMul(_)
            | Instruction::FDiv(_)
            | Instruction::FRem(_)
            | Instruction::FNeg(_)
            | Instruction::ExtractElement(_)
            | Instruction::InsertElement(_)
            | Instruction::ShuffleVector(_)
            | Instruction::ExtractValue(_)
            | Instruction::InsertValue(_)
            | Instruction::Alloca(_)
            | Instruction::GetElementPtr(_)
            | Instruction::Trunc(_)
            | Instruction::ZExt(_)
            | Instruction::SExt(_)
            | Instruction::FPTrunc(_)
            | Instruction::FPExt(_)
            | Instruction::FPToUI(_)
            | Instruction::FPToSI(_)
            | Instruction::UIToFP(_)
            | Instruction::SIToFP(_)
            | Instruction::PtrToInt(_)
            | Instruction::IntToPtr(_)
            | Instruction::BitCast(_)
            | Instruction::AddrSpaceCast(_)
            | Instruction::ICmp(_)
            | Instruction::FCmp(_)
            | Instruction::Phi(_)
            | Instruction::Select(_)
            | Instruction::Freeze(_)
            | Instruction::Call(_)
            | Instruction::VAArg(_)
            | Instruction::LandingPad(_)
            | Instruction::CatchPad(_)
            | Instruction::CleanupPad(_) => false,

            Instruction::Fence(_) | Instruction::CmpXchg(_) | Instruction::AtomicRMW(_) => true,

            Instruction::Load(i) => i.atomicity.is_some(),
            Instruction::Store(i) => i.atomicity.is_some(),
        }
    }
}

/* --TODO not yet implemented: metadata
pub trait HasMetadata {
    fn get_metadata(&self) -> &InstructionMetadata;
}

impl HasMetadata for Instruction {
    fn get_metadata(&self) -> &InstructionMetadata {
        match self {
            Instruction::Add(i) => &i.metadata,
            Instruction::Sub(i) => &i.metadata,
            Instruction::Mul(i) => &i.metadata,
            Instruction::UDiv(i) => &i.metadata,
            Instruction::SDiv(i) => &i.metadata,
            Instruction::URem(i) => &i.metadata,
            Instruction::SRem(i) => &i.metadata,
            Instruction::And(i) => &i.metadata,
            Instruction::Or(i) => &i.metadata,
            Instruction::Xor(i) => &i.metadata,
            Instruction::Shl(i) => &i.metadata,
            Instruction::LShr(i) => &i.metadata,
            Instruction::AShr(i) => &i.metadata,
            Instruction::FAdd(i) => &i.metadata,
            Instruction::FSub(i) => &i.metadata,
            Instruction::FMul(i) => &i.metadata,
            Instruction::FDiv(i) => &i.metadata,
            Instruction::FRem(i) => &i.metadata,
            Instruction::FNeg(i) => &i.metadata,
            Instruction::ExtractElement(i) => &i.metadata,
            Instruction::InsertElement(i) => &i.metadata,
            Instruction::ShuffleVector(i) => &i.metadata,
            Instruction::ExtractValue(i) => &i.metadata,
            Instruction::InsertValue(i) => &i.metadata,
            Instruction::Alloca(i) => &i.metadata,
            Instruction::Load(i) => &i.metadata,
            Instruction::Store(i) => &i.metadata,
            Instruction::Fence(i) => &i.metadata,
            Instruction::CmpXchg(i) => &i.metadata,
            Instruction::AtomicRMW(i) => &i.metadata,
            Instruction::GetElementPtr(i) => &i.metadata,
            Instruction::Trunc(i) => &i.metadata,
            Instruction::ZExt(i) => &i.metadata,
            Instruction::SExt(i) => &i.metadata,
            Instruction::FPTrunc(i) => &i.metadata,
            Instruction::FPExt(i) => &i.metadata,
            Instruction::FPToUI(i) => &i.metadata,
            Instruction::FPToSI(i) => &i.metadata,
            Instruction::UIToFP(i) => &i.metadata,
            Instruction::SIToFP(i) => &i.metadata,
            Instruction::PtrToInt(i) => &i.metadata,
            Instruction::IntToPtr(i) => &i.metadata,
            Instruction::BitCast(i) => &i.metadata,
            Instruction::AddrSpaceCast(i) => &i.metadata,
            Instruction::ICmp(i) => &i.metadata,
            Instruction::FCmp(i) => &i.metadata,
            Instruction::Phi(i) => &i.metadata,
            Instruction::Select(i) => &i.metadata,
            #[cfg(feature="llvm-10-or-greater")]
            Instruction::Freeze(i) => &i.metadata,
            Instruction::Call(i) => &i.metadata,
            Instruction::VAArg(i) => &i.metadata,
            Instruction::LandingPad(i) => &i.metadata,
            Instruction::CatchPad(i) => &i.metadata,
            Instruction::CleanupPad(i) => &i.metadata,
        }
    }
}
*/

pub trait HasResult: Debug + Typed {
    fn get_result(&self) -> &Name;
}

pub trait UnaryOp: HasResult {
    fn get_operand(&self) -> &Operand;
}

pub trait BinaryOp: HasResult {
    fn get_operand0(&self) -> &Operand;
    fn get_operand1(&self) -> &Operand;
}

impl Instruction {
    /// Determine if the `Instruction` is one of the ones in
    /// [`groups::BinaryOp`](groups/enum.BinaryOp.html), without actually using
    /// `try_into()` (which would consume it)
    #[must_use]
    pub fn is_binary_op(&self) -> bool {
        matches!(
            self,
            Instruction::Add(_)
                | Instruction::Sub(_)
                | Instruction::Mul(_)
                | Instruction::UDiv(_)
                | Instruction::SDiv(_)
                | Instruction::URem(_)
                | Instruction::SRem(_)
                | Instruction::And(_)
                | Instruction::Or(_)
                | Instruction::Xor(_)
                | Instruction::Shl(_)
                | Instruction::LShr(_)
                | Instruction::AShr(_)
                | Instruction::FAdd(_)
                | Instruction::FSub(_)
                | Instruction::FMul(_)
                | Instruction::FDiv(_)
                | Instruction::FRem(_)
        )
    }

    /// Determine if the `Instruction` is one of the ones in
    /// [`groups::UnaryOp`](groups/enum.UnaryOp.html), without actually using
    /// `try_into()` (which would consume it)
    #[must_use]
    pub fn is_unary_op(&self) -> bool {
        matches!(
            self,
            Instruction::AddrSpaceCast(_)
                | Instruction::BitCast(_)
                | Instruction::FNeg(_)
                | Instruction::FPExt(_)
                | Instruction::FPToSI(_)
                | Instruction::FPToUI(_)
                | Instruction::FPTrunc(_)
                | Instruction::Freeze(_)
                | Instruction::IntToPtr(_)
                | Instruction::PtrToInt(_)
                | Instruction::SExt(_)
                | Instruction::SIToFP(_)
                | Instruction::Trunc(_)
                | Instruction::UIToFP(_)
                | Instruction::ZExt(_)
        )
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instruction::Add(i) => write!(f, "{i}"),
            Instruction::Sub(i) => write!(f, "{i}"),
            Instruction::Mul(i) => write!(f, "{i}"),
            Instruction::UDiv(i) => write!(f, "{i}"),
            Instruction::SDiv(i) => write!(f, "{i}"),
            Instruction::URem(i) => write!(f, "{i}"),
            Instruction::SRem(i) => write!(f, "{i}"),
            Instruction::And(i) => write!(f, "{i}"),
            Instruction::Or(i) => write!(f, "{i}"),
            Instruction::Xor(i) => write!(f, "{i}"),
            Instruction::Shl(i) => write!(f, "{i}"),
            Instruction::LShr(i) => write!(f, "{i}"),
            Instruction::AShr(i) => write!(f, "{i}"),
            Instruction::FAdd(i) => write!(f, "{i}"),
            Instruction::FSub(i) => write!(f, "{i}"),
            Instruction::FMul(i) => write!(f, "{i}"),
            Instruction::FDiv(i) => write!(f, "{i}"),
            Instruction::FRem(i) => write!(f, "{i}"),
            Instruction::FNeg(i) => write!(f, "{i}"),
            Instruction::ExtractElement(i) => write!(f, "{i}"),
            Instruction::InsertElement(i) => write!(f, "{i}"),
            Instruction::ShuffleVector(i) => write!(f, "{i}"),
            Instruction::ExtractValue(i) => write!(f, "{i}"),
            Instruction::InsertValue(i) => write!(f, "{i}"),
            Instruction::Alloca(i) => write!(f, "{i}"),
            Instruction::Load(i) => write!(f, "{i}"),
            Instruction::Store(i) => write!(f, "{i}"),
            Instruction::Fence(i) => write!(f, "{i}"),
            Instruction::CmpXchg(i) => write!(f, "{i}"),
            Instruction::AtomicRMW(i) => write!(f, "{i}"),
            Instruction::GetElementPtr(i) => write!(f, "{i}"),
            Instruction::Trunc(i) => write!(f, "{i}"),
            Instruction::ZExt(i) => write!(f, "{i}"),
            Instruction::SExt(i) => write!(f, "{i}"),
            Instruction::FPTrunc(i) => write!(f, "{i}"),
            Instruction::FPExt(i) => write!(f, "{i}"),
            Instruction::FPToUI(i) => write!(f, "{i}"),
            Instruction::FPToSI(i) => write!(f, "{i}"),
            Instruction::UIToFP(i) => write!(f, "{i}"),
            Instruction::SIToFP(i) => write!(f, "{i}"),
            Instruction::PtrToInt(i) => write!(f, "{i}"),
            Instruction::IntToPtr(i) => write!(f, "{i}"),
            Instruction::BitCast(i) => write!(f, "{i}"),
            Instruction::AddrSpaceCast(i) => write!(f, "{i}"),
            Instruction::ICmp(i) => write!(f, "{i}"),
            Instruction::FCmp(i) => write!(f, "{i}"),
            Instruction::Phi(i) => write!(f, "{i}"),
            Instruction::Select(i) => write!(f, "{i}"),
            Instruction::Freeze(i) => write!(f, "{i}"),
            Instruction::Call(i) => write!(f, "{i}"),
            Instruction::VAArg(i) => write!(f, "{i}"),
            Instruction::LandingPad(i) => write!(f, "{i}"),
            Instruction::CatchPad(i) => write!(f, "{i}"),
            Instruction::CleanupPad(i) => write!(f, "{i}"),
        }
    }
}

macro_rules! impl_inst {
    ($inst:ty, $id:ident) => {
        impl From<$inst> for Instruction {
            fn from(inst: $inst) -> Instruction {
                Instruction::$id(inst)
            }
        }

        impl TryFrom<Instruction> for $inst {
            type Error = &'static str;
            fn try_from(inst: Instruction) -> Result<Self, Self::Error> {
                match inst {
                    Instruction::$id(inst) => Ok(inst),
                    _ => Err("Instruction is not of requested type"),
                }
            }
        }

        impl HasDebugLoc for $inst {
            fn get_debug_loc(&self) -> &Option<DebugLoc> {
                &self.debugloc
            }
        }

        /* --TODO not yet implemented: metadata
        impl HasMetadata for $inst {
            fn get_metadata(&self) -> &InstructionMetadata {
                &self.metadata
            }
        }
        */
    };
}

macro_rules! impl_hasresult {
    ($inst:ty) => {
        impl HasResult for $inst {
            fn get_result(&self) -> &Name {
                &self.dest
            }
        }
    };
}

// impls which are shared by all UnaryOps.
// If possible, prefer `unop_same_type!` or `unop_explicitly_typed!`, which
// provide additional impls
macro_rules! impl_unop {
    ($inst:ty) => {
        impl_hasresult!($inst);

        impl UnaryOp for $inst {
            fn get_operand(&self) -> &Operand {
                &self.operand
            }
        }
    };
}

// impls which are shared by all BinaryOps.
// If possible, prefer `binop_same_type!` or `binop_left_type!`, which
// provide additional impls
macro_rules! impl_binop {
    ($inst:ty, $id:ident, $dispname:expr) => {
        impl_hasresult!($inst);

        impl BinaryOp for $inst {
            fn get_operand0(&self) -> &Operand {
                &self.operand0
            }
            fn get_operand1(&self) -> &Operand {
                &self.operand1
            }
        }

        impl From<$inst> for groups::BinaryOp {
            fn from(inst: $inst) -> Self {
                groups::BinaryOp::$id(inst)
            }
        }

        impl TryFrom<groups::BinaryOp> for $inst {
            type Error = &'static str;
            fn try_from(bo: groups::BinaryOp) -> Result<Self, Self::Error> {
                match bo {
                    groups::BinaryOp::$id(i) => Ok(i),
                    _ => Err("BinaryOp is not of requested type"),
                }
            }
        }

        impl Display for $inst {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    f,
                    "{} = {} {}, {}",
                    &self.dest, $dispname, &self.operand0, &self.operand1,
                )?;
                if self.debugloc.is_some() {
                    write!(f, " (with debugloc)")?;
                }
                Ok(())
            }
        }
    };
}

// Use on unops where the result type is the same as the operand type
macro_rules! unop_same_type {
    ($inst:ty, $dispname:expr) => {
        impl_unop!($inst);

        impl Typed for $inst {
            fn get_type(&self, types: &Types) -> TypeRef {
                types.type_of(self.get_operand())
            }
        }

        impl Display for $inst {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{} = {} {}", &self.dest, $dispname, &self.operand)?;
                if self.debugloc.is_some() {
                    write!(f, " (with debugloc)")?;
                }
                Ok(())
            }
        }
    };
}

// Use on unops with a `to_type` field which indicates the result type
macro_rules! unop_explicitly_typed {
    ($inst:ty, $dispname:expr) => {
        impl_unop!($inst);
        explicitly_typed!($inst);

        impl Display for $inst {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    f,
                    "{} = {} {} to {}",
                    &self.dest, $dispname, &self.operand, &self.to_type,
                )?;
                if self.debugloc.is_some() {
                    write!(f, " (with debugloc)")?;
                }
                Ok(())
            }
        }
    };
}

// Use on binops where the result type is the same as both operand types
macro_rules! binop_same_type {
    ($inst:ty, $id:ident, $dispname:expr) => {
        impl_binop!($inst, $id, $dispname);

        impl Typed for $inst {
            fn get_type(&self, types: &Types) -> TypeRef {
                let ty = types.type_of(self.get_operand0());
                debug_assert_eq!(ty, types.type_of(self.get_operand1()));
                ty
            }
        }
    };
}

// Use on binops where the result type is the same as the first operand type
macro_rules! binop_left_type {
    ($inst:ty, $id:ident, $dispname:expr) => {
        impl_binop!($inst, $id, $dispname);

        impl Typed for $inst {
            fn get_type(&self, types: &Types) -> TypeRef {
                types.type_of(self.get_operand0())
            }
        }
    };
}

// Use on anything that has a 'to_type' field which indicates its result type
macro_rules! explicitly_typed {
    ($inst:ty) => {
        impl Typed for $inst {
            fn get_type(&self, _types: &Types) -> TypeRef {
                self.to_type.clone()
            }
        }
    };
}

macro_rules! void_typed {
    ($inst:ty) => {
        impl Typed for $inst {
            fn get_type(&self, types: &Types) -> TypeRef {
                types.void()
            }
        }
    };
}

/// Integer add.
/// See [LLVM 14 docs on the 'add' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#add-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Add {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    // pub nsw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    // pub nuw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(Add, Add);
binop_same_type!(Add, Add, "add");

/// Integer subtract.
/// See [LLVM 14 docs on the 'sub' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#sub-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Sub {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    // pub nsw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    // pub nuw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(Sub, Sub);
binop_same_type!(Sub, Sub, "sub");

/// Integer multiply.
/// See [LLVM 14 docs on the 'mul' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#mul-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Mul {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    // pub nsw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    // pub nuw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(Mul, Mul);
binop_same_type!(Mul, Mul, "mul");

/// Unsigned integer divide.
/// See [LLVM 14 docs on the 'udiv' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#udiv-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct UDiv {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    // pub exact: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(UDiv, UDiv);
binop_same_type!(UDiv, UDiv, "udiv");

/// Signed integer divide.
/// See [LLVM 14 docs on the 'sdiv' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#sdiv-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct SDiv {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    // pub exact: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(SDiv, SDiv);
binop_same_type!(SDiv, SDiv, "sdiv");

/// Unsigned integer remainder.
/// See [LLVM 14 docs on the 'urem' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#urem-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct URem {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(URem, URem);
binop_same_type!(URem, URem, "urem");

/// Signed integer remainder.
/// See [LLVM 14 docs on the 'srem' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#srem-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct SRem {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(SRem, SRem);
binop_same_type!(SRem, SRem, "srem");

/// Bitwise logical and.
/// See [LLVM 14 docs on the 'and' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#and-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct And {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(And, And);
binop_same_type!(And, And, "and");

/// Bitwise logical inclusive or.
/// See [LLVM 14 docs on the 'or' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#or-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Or {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(Or, Or);
binop_same_type!(Or, Or, "or");

/// Bitwise logical exclusive or.
/// See [LLVM 14 docs on the 'xor' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#xor-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Xor {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(Xor, Xor);
binop_same_type!(Xor, Xor, "xor");

/// Shift left.
/// See [LLVM 14 docs on the 'shl' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#shl-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Shl {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    // pub nsw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    // pub nuw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(Shl, Shl);
binop_left_type!(Shl, Shl, "shl");

/// Logical shift right.
/// See [LLVM 14 docs on the 'lshr' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#lshr-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct LShr {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    // pub exact: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(LShr, LShr);
binop_left_type!(LShr, LShr, "lshr");

/// Arithmetic shift right.
/// See [LLVM 14 docs on the 'ashr' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#ashr-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct AShr {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    // pub exact: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(AShr, AShr);
binop_left_type!(AShr, AShr, "ashr");

/// Floating-point add.
/// See [LLVM 14 docs on the 'fadd' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#fadd-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct FAdd {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    // pub fast_math_flags: FastMathFlags,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(FAdd, FAdd);
binop_same_type!(FAdd, FAdd, "fadd");

/// Floating-point subtract.
/// See [LLVM 14 docs on the 'fsub' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#fsub-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct FSub {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    // pub fast_math_flags: FastMathFlags,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(FSub, FSub);
binop_same_type!(FSub, FSub, "fsub");

/// Floating-point multiply.
/// See [LLVM 14 docs on the 'fmul' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#fmul-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct FMul {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    // pub fast_math_flags: FastMathFlags,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(FMul, FMul);
binop_same_type!(FMul, FMul, "fmul");

/// Floating-point divide.
/// See [LLVM 14 docs on the 'fdiv' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#fdiv-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct FDiv {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    // pub fast_math_flags: FastMathFlags,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(FDiv, FDiv);
binop_same_type!(FDiv, FDiv, "fdiv");

/// Floating-point remainder.
/// See [LLVM 14 docs on the 'frem' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#frem-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct FRem {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    // pub fast_math_flags: FastMathFlags,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(FRem, FRem);
binop_same_type!(FRem, FRem, "frem");

/// Floating-point unary negation.
/// See [LLVM 14 docs on the 'fneg' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#fneg-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct FNeg {
    pub operand: Operand,
    pub dest: Name,
    // pub fast_math_flags: FastMathFlags,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(FNeg, FNeg);
unop_same_type!(FNeg, "fneg");

/// Get an element from a vector at a specified index.
/// See [LLVM 14 docs on the 'extractelement' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#extractelement-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct ExtractElement {
    pub vector: Operand,
    pub index: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(ExtractElement, ExtractElement);
impl_hasresult!(ExtractElement);

impl Typed for ExtractElement {
    fn get_type(&self, types: &Types) -> TypeRef {
        match types.type_of(&self.vector).as_ref() {
            Type::VectorType { element_type, .. } => element_type.clone(),
            ty => panic!("Expected an ExtractElement vector to be VectorType, got {ty:?}"),
        }
    }
}

impl Display for ExtractElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} = extractelement {}, {}",
            &self.dest, &self.vector, &self.index,
        )?;
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// Insert an element into a vector at a specified index.
/// See [LLVM 14 docs on the 'insertelement' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#insertelement-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct InsertElement {
    pub vector: Operand,
    pub element: Operand,
    pub index: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(InsertElement, InsertElement);
impl_hasresult!(InsertElement);

impl Typed for InsertElement {
    fn get_type(&self, types: &Types) -> TypeRef {
        types.type_of(&self.vector)
    }
}

impl Display for InsertElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} = insertelement {}, {}, {}",
            &self.dest, &self.vector, &self.element, &self.index,
        )?;
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// Permute elements from two input vectors into a single output vector.
/// See [LLVM 14 docs on the 'shufflevector' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#shufflevector-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct ShuffleVector {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub mask: ConstantRef,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(ShuffleVector, ShuffleVector);
impl_hasresult!(ShuffleVector);

impl Typed for ShuffleVector {
    fn get_type(&self, types: &Types) -> TypeRef {
        let ty = types.type_of(&self.operand0);
        debug_assert_eq!(ty, types.type_of(&self.operand1));
        match ty.as_ref() {
            Type::VectorType { element_type, .. } => match types.type_of(&self.mask).as_ref() {
                Type::VectorType {
                    num_elements,
                    scalable,
                    ..
                } => types.vector_of(element_type.clone(), *num_elements, *scalable),
                ty => panic!("Expected a ShuffleVector mask to be VectorType, got {ty:?}"),
            },
            _ => panic!("Expected a ShuffleVector operand to be VectorType, got {ty:?}"),
        }
    }
}

impl Display for ShuffleVector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} = shufflevector {}, {}, {}",
            &self.dest, &self.operand0, &self.operand1, &self.mask,
        )?;
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// Extract the value of a member field from an aggregate (struct or array) type.
/// See [LLVM 14 docs on the 'extractvalue' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#extractvalue-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct ExtractValue {
    pub aggregate: Operand,
    pub indices: Vec<u32>,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(ExtractValue, ExtractValue);
impl_hasresult!(ExtractValue);

impl Typed for ExtractValue {
    fn get_type(&self, types: &Types) -> TypeRef {
        ev_type(types.type_of(&self.aggregate), self.indices.iter().copied())
    }
}

fn ev_type(cur_type: TypeRef, mut indices: impl Iterator<Item = u32>) -> TypeRef {
    match indices.next() {
        None => cur_type,
        Some(index) => match cur_type.as_ref() {
            Type::ArrayType { element_type, .. } => ev_type(element_type.clone(), indices),
            Type::StructType { element_types, .. } => ev_type(
                element_types
                    .get(index as usize)
                    .expect("ExtractValue index out of range")
                    .clone(),
                indices,
            ),
            _ => panic!(
                "ExtractValue from something that's not ArrayType or StructType; its type is {cur_type:?}"
            ),
        },
    }
}

impl Display for ExtractValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} = extractvalue {}, {}",
            &self.dest,
            &self.aggregate,
            &self.indices.first().expect("ExtractValue with no indices")
        )?;
        for idx in &self.indices[1..] {
            write!(f, ", {idx}")?;
        }
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// Insert a value into a member field of an aggregate (struct or array) type.
/// See [LLVM 14 docs on the 'insertvalue' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#insertvalue-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct InsertValue {
    pub aggregate: Operand,
    pub element: Operand,
    pub indices: Vec<u32>,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(InsertValue, InsertValue);
impl_hasresult!(InsertValue);

impl Typed for InsertValue {
    fn get_type(&self, types: &Types) -> TypeRef {
        types.type_of(&self.aggregate)
    }
}

impl Display for InsertValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} = insertvalue {}, {}, {}",
            &self.dest,
            &self.aggregate,
            &self.element,
            &self.indices.first().expect("InsertValue with no indices"),
        )?;
        for idx in &self.indices[1..] {
            write!(f, ", {idx}")?;
        }
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// Allocate memory on the stack.
/// See [LLVM 14 docs on the 'alloca' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#alloca-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Alloca {
    pub allocated_type: TypeRef,
    pub num_elements: Operand, // llvm-hs-pure has Option<Operand>
    pub dest: Name,
    pub alignment: u32,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(Alloca, Alloca);
impl_hasresult!(Alloca);

impl Typed for Alloca {
    fn get_type(&self, types: &Types) -> TypeRef {
        types.pointer_to(self.allocated_type.clone())
    }
}

impl Display for Alloca {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} = alloca {}", &self.dest, &self.allocated_type,)?;
        if let Some(Constant::Int { value: 1, .. }) = self.num_elements.as_constant() {
            // omit num_elements
        } else {
            write!(f, ", {}", &self.num_elements)?;
        }
        write!(f, ", align {}", &self.alignment)?;
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// Load a value from memory.
/// See [LLVM 14 docs on the 'load' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#load-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Load {
    pub address: Operand,
    pub dest: Name,
    pub volatile: bool,
    pub atomicity: Option<Atomicity>,
    pub alignment: u32,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(Load, Load);
impl_hasresult!(Load);

impl Typed for Load {
    fn get_type(&self, types: &Types) -> TypeRef {
        match types.type_of(&self.address).as_ref() {
            Type::PointerType { pointee_type, .. } => pointee_type.clone(),
            ty => panic!("Expected a load address to be PointerType, got {ty:?}"),
        }
    }
}

impl Display for Load {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // we differ from the LLVM IR text syntax here because we don't include
        // the destination type (that's a little hard to get for us here, and
        // it's completely redundant with the address type anyway)
        write!(f, "{} = load ", &self.dest)?;
        if self.atomicity.is_some() {
            write!(f, "atomic ")?;
        }
        if self.volatile {
            write!(f, "volatile ")?;
        }
        write!(f, "{}", &self.address)?;
        if let Some(a) = &self.atomicity {
            write!(f, " {a}")?;
        }
        write!(f, ", align {}", &self.alignment)?;
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// Store a value to memory.
/// See [LLVM 14 docs on the 'store' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#store-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Store {
    pub address: Operand,
    pub value: Operand,
    pub volatile: bool,
    pub atomicity: Option<Atomicity>,
    pub alignment: u32,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(Store, Store);
void_typed!(Store);

impl Display for Store {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "store ")?;
        if self.atomicity.is_some() {
            write!(f, "atomic ")?;
        }
        if self.volatile {
            write!(f, "volatile ")?;
        }
        write!(f, "{}, {}", &self.value, &self.address)?;
        if let Some(a) = &self.atomicity {
            write!(f, " {a}")?;
        }
        write!(f, ", align {}", &self.alignment)?;
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// Memory-ordering fence.
/// See [LLVM 14 docs on the 'fence' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#fence-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Fence {
    pub atomicity: Atomicity,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(Fence, Fence);
void_typed!(Fence);

impl Display for Fence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "fence {}", &self.atomicity)?;
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// Atomic compare and exchange.
/// See [LLVM 14 docs on the 'cmpxchg' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#cmpxchg-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct CmpXchg {
    pub address: Operand,
    pub expected: Operand,
    pub replacement: Operand,
    pub dest: Name,
    pub volatile: bool,
    /// This includes the "success" `MemoryOrdering`
    pub atomicity: Atomicity,
    /// This is the "failure" `MemoryOrdering`
    pub failure_memory_ordering: MemoryOrdering,
    pub weak: bool,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(CmpXchg, CmpXchg);
impl_hasresult!(CmpXchg);

impl Typed for CmpXchg {
    fn get_type(&self, types: &Types) -> TypeRef {
        let ty = types.type_of(&self.expected);
        debug_assert_eq!(ty, types.type_of(&self.replacement));
        types.struct_of(vec![ty, types.bool()], false)
    }
}

impl Display for CmpXchg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} = cmpxchg ", &self.dest)?;
        if self.weak {
            write!(f, "weak ")?;
        }
        if self.volatile {
            write!(f, "volatile ")?;
        }
        write!(
            f,
            "{}, {}, {} {} {}",
            &self.address,
            &self.expected,
            &self.replacement,
            &self.atomicity,
            &self.failure_memory_ordering,
        )?;
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// Atomic read-modify-write.
/// See [LLVM 14 docs on the 'atomicrmw' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#atomicrmw-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct AtomicRMW {
    // the binop-getter was added to the LLVM C API in LLVM 10
    pub operation: RMWBinOp,
    pub address: Operand,
    pub value: Operand,
    pub dest: Name,
    pub volatile: bool,
    pub atomicity: Atomicity,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(AtomicRMW, AtomicRMW);
impl_hasresult!(AtomicRMW);

impl Typed for AtomicRMW {
    fn get_type(&self, types: &Types) -> TypeRef {
        match types.type_of(&self.address).as_ref() {
            Type::PointerType { pointee_type, .. } => pointee_type.clone(),
            ty => panic!("Expected an AtomicRMW address to be PointerType, got {ty:?}"),
        }
    }
}

impl Display for AtomicRMW {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} = atomicrmw ", &self.dest)?;
        if self.volatile {
            write!(f, "volatile ")?;
        }
        write!(f, "{} ", &self.operation)?;
        write!(f, "{}, {} {}", &self.address, &self.value, &self.atomicity)?;
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// Get the address of a subelement of an aggregate data structure.
/// Only performs address calculation, does not actually access memory.
/// See [LLVM 14 docs on the 'getelementptr' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#getelementptr-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct GetElementPtr {
    pub address: Operand,
    pub indices: Vec<Operand>,
    pub dest: Name,
    pub in_bounds: bool,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(GetElementPtr, GetElementPtr);
impl_hasresult!(GetElementPtr);

impl Typed for GetElementPtr {
    fn get_type(&self, types: &Types) -> TypeRef {
        gep_type(types.type_of(&self.address), self.indices.iter(), types)
    }
}

fn gep_type<'o>(
    cur_type: TypeRef,
    mut indices: impl Iterator<Item = &'o Operand>,
    types: &Types,
) -> TypeRef {
    if let Type::NamedStructType { name } = cur_type.as_ref() {
        match types.named_struct_def(name) {
            None => panic!("Named struct without a definition (name {name:?})"),
            Some(NamedStructDef::Opaque) => {
                panic!("GEP on an opaque struct type (name {name:?})")
            }
            Some(NamedStructDef::Defined(ty)) => {
                return gep_type(ty.clone(), indices, types);
            }
        }
    }
    match indices.next() {
        None => types.pointer_to(cur_type),  // iterator is done
        Some(index) => match cur_type.as_ref() {
            Type::PointerType { pointee_type, .. } => gep_type(pointee_type.clone(), indices, types),
            Type::VectorType { element_type, .. } | Type::ArrayType { element_type, .. } => gep_type(element_type.clone(), indices, types),
            Type::StructType { element_types, .. } => {
                if let Operand::ConstantOperand(cref) = index {
                    if let Constant::Int { value, .. } = cref.as_ref() {
                        gep_type(element_types.get(*value as usize).cloned().expect("GEP index out of range"), indices, types)
                    } else {
                        panic!("Expected GEP index on a struct to be a Constant::Int; got {cref:?}")
                    }
                } else {
                    panic!("Expected GEP index on a struct to be a Operand::ConstantOperand(Constant::Int); got {index:?}")
                }
            },
            Type::NamedStructType { .. } => panic!("This case should have been handled above"),
            _ => panic!("Expected GEP base type to be a PointerType, VectorType, ArrayType, StructType, or NamedStructType; got {cur_type:?}"),
        }
    }
}

impl Display for GetElementPtr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Like for `Load` (see notes there), we differ from the LLVM IR text
        // syntax here because we don't include the destination type (that's a
        // little hard to get for us here, and it's derivable from the other
        // information anyway)
        write!(f, "{} = getelementptr ", &self.dest)?;
        if self.in_bounds {
            write!(f, "inbounds ")?;
        }
        write!(f, "{}", &self.address)?;
        for idx in &self.indices {
            write!(f, ", {idx}")?;
        }
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// Truncate.
/// See [LLVM 14 docs on the 'trunc' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#trunc-to-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Trunc {
    pub operand: Operand,
    pub to_type: TypeRef,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(Trunc, Trunc);
unop_explicitly_typed!(Trunc, "trunc");

/// Zero-extend.
/// See [LLVM 14 docs on the 'zext' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#zext-to-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct ZExt {
    pub operand: Operand,
    pub to_type: TypeRef,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(ZExt, ZExt);
unop_explicitly_typed!(ZExt, "zext");

/// Sign-extend.
/// See [LLVM 14 docs on the 'sext' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#sext-to-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct SExt {
    pub operand: Operand,
    pub to_type: TypeRef,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(SExt, SExt);
unop_explicitly_typed!(SExt, "sext");

/// Truncate a floating-point value.
/// See [LLVM 14 docs on the 'fptrunc' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#fptrunc-to-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct FPTrunc {
    pub operand: Operand,
    pub to_type: TypeRef,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(FPTrunc, FPTrunc);
unop_explicitly_typed!(FPTrunc, "fptrunc");

/// Extend a floating-point value.
/// See [LLVM 14 docs on the 'fpext' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#fpext-to-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct FPExt {
    pub operand: Operand,
    pub to_type: TypeRef,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(FPExt, FPExt);
unop_explicitly_typed!(FPExt, "fpext");

/// Convert floating-point to unsigned integer.
/// See [LLVM 14 docs on the 'fptoui' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#fptoui-to-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct FPToUI {
    pub operand: Operand,
    pub to_type: TypeRef,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(FPToUI, FPToUI);
unop_explicitly_typed!(FPToUI, "fptoui");

/// Convert floating-point to signed integer.
/// See [LLVM 14 docs on the 'fptosi' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#fptosi-to-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct FPToSI {
    pub operand: Operand,
    pub to_type: TypeRef,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(FPToSI, FPToSI);
unop_explicitly_typed!(FPToSI, "fptosi");

/// Convert unsigned integer to floating-point.
/// See [LLVM 14 docs on the 'uitofp' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#uitofp-to-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct UIToFP {
    pub operand: Operand,
    pub to_type: TypeRef,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(UIToFP, UIToFP);
unop_explicitly_typed!(UIToFP, "uitofp");

/// Convert signed integer to floating-point.
/// See [LLVM 14 docs on the 'sitofp' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#sitofp-to-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct SIToFP {
    pub operand: Operand,
    pub to_type: TypeRef,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(SIToFP, SIToFP);
unop_explicitly_typed!(SIToFP, "sitofp");

/// Convert pointer to integer.
/// See [LLVM 14 docs on the 'ptrtoint' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#ptrtoint-to-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct PtrToInt {
    pub operand: Operand,
    pub to_type: TypeRef,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(PtrToInt, PtrToInt);
unop_explicitly_typed!(PtrToInt, "ptrtoint");

/// Convert integer to pointer.
/// See [LLVM 14 docs on the 'inttoptr' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#inttoptr-to-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct IntToPtr {
    pub operand: Operand,
    pub to_type: TypeRef,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(IntToPtr, IntToPtr);
unop_explicitly_typed!(IntToPtr, "inttoptr");

/// Convert between types without changing any bits.
/// See [LLVM 14 docs on the 'bitcast' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#bitcast-to-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct BitCast {
    pub operand: Operand,
    pub to_type: TypeRef,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(BitCast, BitCast);
unop_explicitly_typed!(BitCast, "bitcast");

/// Convert a pointer to a different address space.
/// See [LLVM 14 docs on the 'addrspacecast' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#addrspacecast-to-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct AddrSpaceCast {
    pub operand: Operand,
    pub to_type: TypeRef,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(AddrSpaceCast, AddrSpaceCast);
unop_explicitly_typed!(AddrSpaceCast, "addrspacecast");

/// Compare integers, pointers, or vectors of integers or pointers.
/// See [LLVM 14 docs on the 'icmp' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#icmp-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct ICmp {
    pub predicate: IntPredicate,
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(ICmp, ICmp);
impl_hasresult!(ICmp);

impl Typed for ICmp {
    fn get_type(&self, types: &Types) -> TypeRef {
        let ty = types.type_of(&self.operand0);
        debug_assert_eq!(ty, types.type_of(&self.operand1));
        match ty.as_ref() {
            Type::VectorType {
                num_elements,
                scalable,
                ..
            } => types.vector_of(types.bool(), *num_elements, *scalable),
            _ => types.bool(),
        }
    }
}

impl Display for ICmp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} = icmp {} {}, {}",
            &self.dest, &self.predicate, &self.operand0, &self.operand1,
        )?;
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// Compare floating-point values or vectors of floating-point values.
/// See [LLVM 14 docs on the 'fcmp' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#fcmp-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct FCmp {
    pub predicate: FPPredicate,
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(FCmp, FCmp);
impl_hasresult!(FCmp);

impl Typed for FCmp {
    fn get_type(&self, types: &Types) -> TypeRef {
        let ty = types.type_of(&self.operand0);
        debug_assert_eq!(ty, types.type_of(&self.operand1));
        match ty.as_ref() {
            Type::VectorType {
                num_elements,
                scalable,
                ..
            } => types.vector_of(types.bool(), *num_elements, *scalable),
            _ => types.bool(),
        }
    }
}

impl Display for FCmp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} = fcmp {} {}, {}",
            &self.dest, &self.predicate, &self.operand0, &self.operand1,
        )?;
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// See [LLVM 14 docs on the 'phi' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#phi-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Phi {
    pub incoming_values: Vec<(Operand, Name)>,
    pub dest: Name,
    pub to_type: TypeRef,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(Phi, Phi);
impl_hasresult!(Phi);
explicitly_typed!(Phi);

impl Display for Phi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (first_val, first_label) = &self
            .incoming_values
            .get(0)
            .expect("Phi with no incoming values");
        write!(
            f,
            "{} = phi {} [ {}, {} ]",
            &self.dest, &self.to_type, first_val, first_label,
        )?;
        for (val, label) in &self.incoming_values[1..] {
            write!(f, ", [ {val}, {label} ]")?;
        }
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// Choose between two values depending on a condition.
/// See [LLVM 14 docs on the 'select' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#select-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Select {
    pub condition: Operand,
    pub true_value: Operand,
    pub false_value: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(Select, Select);
impl_hasresult!(Select);

impl Typed for Select {
    fn get_type(&self, types: &Types) -> TypeRef {
        let t = types.type_of(&self.true_value);
        debug_assert_eq!(t, types.type_of(&self.false_value));
        t
    }
}

impl Display for Select {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} = select {}, {}, {}",
            &self.dest, &self.condition, &self.true_value, &self.false_value,
        )?;
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// Stop the propagation of `undef` or `poison` values.
/// See [LLVM 14 docs on the 'freeze' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#freeze-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Freeze {
    pub operand: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}
impl_inst!(Freeze, Freeze);
unop_same_type!(Freeze, "freeze");

/// Function call.
/// See [LLVM 14 docs on the 'call' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#call-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Call {
    pub function: Either<InlineAssembly, Operand>,
    pub arguments: Vec<(Operand, Vec<ParameterAttribute>)>,
    pub return_attributes: Vec<ParameterAttribute>,
    pub dest: Option<Name>, // will be None if the `function` returns void
    pub function_attributes: Vec<Attribute>, // llvm-hs has the equivalent of Vec<Either<GroupID, FunctionAttribute>>, but I'm not sure how the GroupID option comes up
    pub is_tail_call: bool, // llvm-hs has the more sophisticated structure Option<TailCallKind>, but the LLVM C API just gives us true/false
    pub calling_convention: CallingConvention,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(Call, Call);

impl Typed for Call {
    fn get_type(&self, types: &Types) -> TypeRef {
        match types.type_of(&self.function).as_ref() {
            Type::PointerType { pointee_type, .. } => match pointee_type.as_ref() {
                Type::FuncType { result_type, .. } => result_type.clone(),
                ty => panic!("Expected Call's function argument to be of type pointer-to-function, got pointer-to-{ty:?}"),
            },
            ty => panic!("Expected Call's function argument to be of type pointer-to-function, got {ty:?}"),
        }
    }
}

impl Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // We choose not to include all the detailed information available in
        // the `Call` struct in this `Display` impl
        if let Some(dest) = &self.dest {
            write!(f, "{dest} = ")?;
        }
        if self.is_tail_call {
            write!(f, "tail ")?;
        }
        write!(
            f,
            "call {}(",
            match &self.function {
                Either::Left(_) => "<inline assembly>".into(),
                Either::Right(op) => format!("{op}"),
            }
        )?;
        for (i, (arg, _)) in self.arguments.iter().enumerate() {
            if i == self.arguments.len() - 1 {
                write!(f, "{arg}")?;
            } else {
                write!(f, "{arg}, ")?;
            }
        }
        write!(f, ")")?;
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// Used to access variadic arguments passed to a function.
/// See [LLVM 14 docs on the 'va_arg' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#va-arg-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct VAArg {
    pub arg_list: Operand,
    pub cur_type: TypeRef,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(VAArg, VAArg);
impl_hasresult!(VAArg);

impl Typed for VAArg {
    fn get_type(&self, _types: &Types) -> TypeRef {
        self.cur_type.clone()
    }
}

impl Display for VAArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} = va_arg {}, {}",
            &self.dest, &self.arg_list, &self.cur_type,
        )?;
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// Used for exception handling.
/// See [LLVM 14 docs on the 'landingpad' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#landingpad-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct LandingPad {
    pub result_type: TypeRef,
    pub clauses: Vec<LandingPadClause>,
    pub dest: Name,
    pub cleanup: bool,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(LandingPad, LandingPad);
impl_hasresult!(LandingPad);

impl Typed for LandingPad {
    fn get_type(&self, _types: &Types) -> TypeRef {
        self.result_type.clone()
    }
}

impl Display for LandingPad {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} = landingpad {}", &self.dest, &self.result_type)?;
        if self.cleanup {
            write!(f, " cleanup")?;
        }
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// Used for exception handling.
/// See [LLVM 14 docs on the 'catchpad' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#catchpad-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct CatchPad {
    pub catch_switch: Operand,
    pub args: Vec<Operand>,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(CatchPad, CatchPad);
impl_hasresult!(CatchPad);

impl Typed for CatchPad {
    fn get_type(&self, types: &Types) -> TypeRef {
        types.token_type()
    }
}

impl Display for CatchPad {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} = catchpad within {} [",
            &self.dest, &self.catch_switch,
        )?;
        for (i, arg) in self.args.iter().enumerate() {
            if i == self.args.len() - 1 {
                write!(f, "{arg}")?;
            } else {
                write!(f, "{arg}, ")?;
            }
        }
        write!(f, "]")?;
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/// Used for exception handling.
/// See [LLVM 14 docs on the 'cleanuppad' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#cleanuppad-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct CleanupPad {
    pub parent_pad: Operand,
    pub args: Vec<Operand>,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
    // --TODO not yet implemented-- pub metadata: InstructionMetadata,
}

impl_inst!(CleanupPad, CleanupPad);
impl_hasresult!(CleanupPad);

impl Typed for CleanupPad {
    fn get_type(&self, types: &Types) -> TypeRef {
        types.token_type()
    }
}

impl Display for CleanupPad {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} = cleanuppad within {} [",
            &self.dest, &self.parent_pad,
        )?;
        for (i, arg) in self.args.iter().enumerate() {
            if i == self.args.len() - 1 {
                write!(f, "{arg}")?;
            } else {
                write!(f, "{arg}, ")?;
            }
        }
        write!(f, "]")?;
        if self.debugloc.is_some() {
            write!(f, " (with debugloc)")?;
        }
        Ok(())
    }
}

/*
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TailCallKind {
    Tail,
    MustTail,
    NoTail,
}
*/

/// See [LLVM 14 docs on Atomic Memory Ordering Constraints](https://releases.llvm.org/14.0.0/docs/LangRef.html#ordering)
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Atomicity {
    pub synch_scope: SynchronizationScope,
    pub mem_ordering: MemoryOrdering,
}

impl Display for Atomicity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.synch_scope {
            SynchronizationScope::SingleThread => write!(f, "syncscope(\"singlethread\") "),
            SynchronizationScope::System => Ok(()),
        }?;
        write!(f, "{}", &self.mem_ordering)?;
        Ok(())
    }
}

/// See [LLVM 14 docs on Atomic Memory Ordering Constraints](https://releases.llvm.org/14.0.0/docs/LangRef.html#ordering)
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SynchronizationScope {
    SingleThread,
    System,
}

/// See [LLVM 14 docs on Atomic Memory Ordering Constraints](https://releases.llvm.org/14.0.0/docs/LangRef.html#ordering)
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MemoryOrdering {
    Unordered,
    Monotonic,
    Acquire,
    Release,
    AcquireRelease,
    SequentiallyConsistent,
    NotAtomic, // since we only have a `MemoryOrdering` on atomic instructions, we should never need this. But empirically, some atomic instructions -- e.g. the first 'atomicrmw' instruction in our 'atomic_no_syncscope' test -- have this `MemoryOrdering`
}

impl Display for MemoryOrdering {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MemoryOrdering::Unordered => write!(f, "unordered"),
            MemoryOrdering::Monotonic => write!(f, "monotonic"),
            MemoryOrdering::Acquire => write!(f, "acquire"),
            MemoryOrdering::Release => write!(f, "release"),
            MemoryOrdering::AcquireRelease => write!(f, "acq_rel"),
            MemoryOrdering::SequentiallyConsistent => write!(f, "seq_cst"),
            MemoryOrdering::NotAtomic => write!(f, "not_atomic"),
        }
    }
}

// --TODO this seems to be the data structure we want. But see notes on
// InlineAssembly::from_llvm_ref()
/*
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct InlineAssembly {
    pub assembly: String,
    pub ty: TypeRef,
    pub constraints: String,
    pub has_side_effects: bool,
    pub align_stack: bool,
    pub dialect: AssemblyDialect,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AssemblyDialect {
    ATT,
    Intel,
}
*/
// Instead we have this for now
/// See [LLVM 14 docs on Inline Assembler Expressions](https://releases.llvm.org/14.0.0/docs/LangRef.html#inline-assembler-expressions)
///
/// `InlineAssembly` needs more fields, but the necessary getter functions are
/// apparently not exposed in the LLVM C API (only the C++ API)
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct InlineAssembly {
    pub ty: TypeRef,
}

impl Typed for InlineAssembly {
    fn get_type(&self, _types: &Types) -> TypeRef {
        self.ty.clone()
    }
}

/// See [LLVM 14 docs on the 'atomicrmw' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#i-atomicrmw)
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RMWBinOp {
    Xchg,
    Add,
    Sub,
    And,
    Nand,
    Or,
    Xor,
    Max,
    Min,
    UMax,
    UMin,
    FAdd,
    FSub,
}

impl Display for RMWBinOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Xchg => write!(f, "xchg"),
            Self::Add => write!(f, "add"),
            Self::Sub => write!(f, "sub"),
            Self::And => write!(f, "and"),
            Self::Nand => write!(f, "nand"),
            Self::Or => write!(f, "or"),
            Self::Xor => write!(f, "xor"),
            Self::Max => write!(f, "max"),
            Self::Min => write!(f, "min"),
            Self::UMax => write!(f, "umax"),
            Self::UMin => write!(f, "umin"),
            Self::FAdd => write!(f, "fadd"),
            Self::FSub => write!(f, "fsub"),
        }
    }
}

// --TODO this seems to be the data structure we want. But see notes on
// LandingPadClause::from_llvm_ref()
/*
#[derive(PartialEq, Clone, Debug)]
pub enum LandingPadClause {
    Catch(Constant),
    Filter(Constant),
}
*/
// Instead we have this for now
/// See [LLVM 14 docs on the 'landingpad' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#landingpad-instruction)
///
/// `LandingPadClause` needs more fields, but the necessary getter functions are
/// apparently not exposed in the LLVM C API (only the C++ API)
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct LandingPadClause {}
