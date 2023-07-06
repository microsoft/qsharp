// Copyright (c) 2019 Craig Disselkoen
// Portions copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::debugloc::{DebugLoc, HasDebugLoc};
use super::function::Attribute;
use super::name::Name;
use super::operand::Operand;
use super::types::TypeRef;
use super::{
    predicates::{FPPredicate, IntPredicate},
    Constant,
};
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

    // Aggregate ops
    ExtractValue(ExtractValue),
    InsertValue(InsertValue),

    // Memory-related ops
    Alloca(Alloca),
    Load(Load),
    Store(Store),
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

    // LLVM's "other operations" category
    ICmp(ICmp),
    FCmp(FCmp),
    Phi(Phi),
    Select(Select),
    Call(Call),
    VAArg(VAArg),
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
            Instruction::ExtractValue(i) => i.get_debug_loc(),
            Instruction::InsertValue(i) => i.get_debug_loc(),
            Instruction::Alloca(i) => i.get_debug_loc(),
            Instruction::Load(i) => i.get_debug_loc(),
            Instruction::Store(i) => i.get_debug_loc(),
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
            Instruction::ICmp(i) => i.get_debug_loc(),
            Instruction::FCmp(i) => i.get_debug_loc(),
            Instruction::Phi(i) => i.get_debug_loc(),
            Instruction::Select(i) => i.get_debug_loc(),
            Instruction::Call(i) => i.get_debug_loc(),
            Instruction::VAArg(i) => i.get_debug_loc(),
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
            Instruction::ExtractValue(i) => Some(&i.dest),
            Instruction::InsertValue(i) => Some(&i.dest),
            Instruction::Alloca(i) => Some(&i.dest),
            Instruction::Load(i) => Some(&i.dest),
            Instruction::Store(_) => None,
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
            Instruction::ICmp(i) => Some(&i.dest),
            Instruction::FCmp(i) => Some(&i.dest),
            Instruction::Phi(i) => Some(&i.dest),
            Instruction::Select(i) => Some(&i.dest),
            Instruction::Call(i) => i.dest.as_ref(),
            Instruction::VAArg(i) => Some(&i.dest),
        }
    }
}

pub trait HasResult: Debug {
    fn get_result(&self) -> &Name;
}

pub trait UnaryOp: HasResult {
    fn get_operand(&self) -> &Operand;
}

pub trait BinaryOp: HasResult {
    fn get_operand0(&self) -> &Operand;
    fn get_operand1(&self) -> &Operand;
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
            Instruction::ExtractValue(i) => write!(f, "{i}"),
            Instruction::InsertValue(i) => write!(f, "{i}"),
            Instruction::Alloca(i) => write!(f, "{i}"),
            Instruction::Load(i) => write!(f, "{i}"),
            Instruction::Store(i) => write!(f, "{i}"),
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
            Instruction::ICmp(i) => write!(f, "{i}"),
            Instruction::FCmp(i) => write!(f, "{i}"),
            Instruction::Phi(i) => write!(f, "{i}"),
            Instruction::Select(i) => write!(f, "{i}"),
            Instruction::Call(i) => write!(f, "{i}"),
            Instruction::VAArg(i) => write!(f, "{i}"),
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

        impl Display for $inst {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{} = {} {}, ", &self.dest, $dispname, &self.operand0,)?;
                self.operand1.fmt_without_type(f)?;
                // if self.debugloc.is_some() {
                // write!(f, " (with debugloc)")?;
                // }
                Ok(())
            }
        }
    };
}

// Use on unops where the result type is the same as the operand type
macro_rules! unop_same_type {
    ($inst:ty, $dispname:expr) => {
        impl_unop!($inst);

        impl Display for $inst {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{} = {} {}", &self.dest, $dispname, &self.operand)?;
                // if self.debugloc.is_some() {
                // write!(f, " (with debugloc)")?;
                // }
                Ok(())
            }
        }
    };
}

// Use on unops with a `to_type` field which indicates the result type
macro_rules! unop_explicitly_typed {
    ($inst:ty, $dispname:expr) => {
        impl_unop!($inst);

        impl Display for $inst {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    f,
                    "{} = {} {} to {}",
                    &self.dest, $dispname, &self.operand, &self.to_type,
                )?;
                // if self.debugloc.is_some() {
                // write!(f, " (with debugloc)")?;
                // }
                Ok(())
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
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(Add, Add);
impl_binop!(Add, Add, "add");

/// Integer subtract.
/// See [LLVM 14 docs on the 'sub' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#sub-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Sub {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(Sub, Sub);
impl_binop!(Sub, Sub, "sub");

/// Integer multiply.
/// See [LLVM 14 docs on the 'mul' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#mul-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Mul {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(Mul, Mul);
impl_binop!(Mul, Mul, "mul");

/// Unsigned integer divide.
/// See [LLVM 14 docs on the 'udiv' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#udiv-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct UDiv {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(UDiv, UDiv);
impl_binop!(UDiv, UDiv, "udiv");

/// Signed integer divide.
/// See [LLVM 14 docs on the 'sdiv' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#sdiv-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct SDiv {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(SDiv, SDiv);
impl_binop!(SDiv, SDiv, "sdiv");

/// Unsigned integer remainder.
/// See [LLVM 14 docs on the 'urem' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#urem-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct URem {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(URem, URem);
impl_binop!(URem, URem, "urem");

/// Signed integer remainder.
/// See [LLVM 14 docs on the 'srem' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#srem-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct SRem {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(SRem, SRem);
impl_binop!(SRem, SRem, "srem");

/// Bitwise logical and.
/// See [LLVM 14 docs on the 'and' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#and-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct And {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(And, And);
impl_binop!(And, And, "and");

/// Bitwise logical inclusive or.
/// See [LLVM 14 docs on the 'or' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#or-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Or {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(Or, Or);
impl_binop!(Or, Or, "or");

/// Bitwise logical exclusive or.
/// See [LLVM 14 docs on the 'xor' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#xor-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Xor {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(Xor, Xor);
impl_binop!(Xor, Xor, "xor");

/// Shift left.
/// See [LLVM 14 docs on the 'shl' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#shl-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Shl {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(Shl, Shl);
impl_binop!(Shl, Shl, "shl");

/// Logical shift right.
/// See [LLVM 14 docs on the 'lshr' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#lshr-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct LShr {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(LShr, LShr);
impl_binop!(LShr, LShr, "lshr");

/// Arithmetic shift right.
/// See [LLVM 14 docs on the 'ashr' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#ashr-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct AShr {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(AShr, AShr);
impl_binop!(AShr, AShr, "ashr");

/// Floating-point add.
/// See [LLVM 14 docs on the 'fadd' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#fadd-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct FAdd {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(FAdd, FAdd);
impl_binop!(FAdd, FAdd, "fadd");

/// Floating-point subtract.
/// See [LLVM 14 docs on the 'fsub' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#fsub-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct FSub {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(FSub, FSub);
impl_binop!(FSub, FSub, "fsub");

/// Floating-point multiply.
/// See [LLVM 14 docs on the 'fmul' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#fmul-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct FMul {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(FMul, FMul);
impl_binop!(FMul, FMul, "fmul");

/// Floating-point divide.
/// See [LLVM 14 docs on the 'fdiv' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#fdiv-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct FDiv {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(FDiv, FDiv);
impl_binop!(FDiv, FDiv, "fdiv");

/// Floating-point remainder.
/// See [LLVM 14 docs on the 'frem' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#frem-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct FRem {
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(FRem, FRem);
impl_binop!(FRem, FRem, "frem");

/// Floating-point unary negation.
/// See [LLVM 14 docs on the 'fneg' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#fneg-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct FNeg {
    pub operand: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
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
}

impl_inst!(ExtractElement, ExtractElement);
impl_hasresult!(ExtractElement);

impl Display for ExtractElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} = extractelement {}, {}",
            &self.dest, &self.vector, &self.index,
        )?;
        // if self.debugloc.is_some() {
        // write!(f, " (with debugloc)")?;
        // }
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
}

impl_inst!(InsertElement, InsertElement);
impl_hasresult!(InsertElement);

impl Display for InsertElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} = insertelement {}, {}, {}",
            &self.dest, &self.vector, &self.element, &self.index,
        )?;
        // if self.debugloc.is_some() {
        // write!(f, " (with debugloc)")?;
        // }
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
}

impl_inst!(ExtractValue, ExtractValue);
impl_hasresult!(ExtractValue);

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
        // if self.debugloc.is_some() {
        // write!(f, " (with debugloc)")?;
        // }
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
}

impl_inst!(InsertValue, InsertValue);
impl_hasresult!(InsertValue);

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
        // if self.debugloc.is_some() {
        // write!(f, " (with debugloc)")?;
        // }
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
}

impl_inst!(Alloca, Alloca);
impl_hasresult!(Alloca);

impl Display for Alloca {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} = alloca {}", &self.dest, &self.allocated_type,)?;
        if let Some(Constant::Int { value: 1, .. }) = self.num_elements.as_constant() {
            // omit num_elements
        } else {
            write!(f, ", {}", &self.num_elements)?;
        }
        write!(f, ", align {}", &self.alignment)?;
        // if self.debugloc.is_some() {
        // write!(f, " (with debugloc)")?;
        // }
        Ok(())
    }
}

/// Load a value from memory.
/// See [LLVM 14 docs on the 'load' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#load-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Load {
    pub address: Operand,
    pub dest: Name,
    pub ty: TypeRef,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(Load, Load);
impl_hasresult!(Load);

impl Display for Load {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} = load {}, {}", &self.dest, &self.ty, &self.address)?;
        // if self.debugloc.is_some() {
        // write!(f, " (with debugloc)")?;
        // }
        Ok(())
    }
}

/// Store a value to memory.
/// See [LLVM 14 docs on the 'store' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#store-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Store {
    pub address: Operand,
    pub value: Operand,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(Store, Store);

impl Display for Store {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "store {}, {}", &self.value, &self.address)?;
        // if self.debugloc.is_some() {
        // write!(f, " (with debugloc)")?;
        // }
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
    pub ty: TypeRef,
    pub in_bounds: bool,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(GetElementPtr, GetElementPtr);
impl_hasresult!(GetElementPtr);

impl Display for GetElementPtr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} = getelementptr ", &self.dest)?;
        if self.in_bounds {
            write!(f, "inbounds ")?;
        }
        write!(f, "{}, {}", &self.ty, &self.address)?;
        for idx in &self.indices {
            write!(f, ", {idx}")?;
        }
        // if self.debugloc.is_some() {
        // write!(f, " (with debugloc)")?;
        // }
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
}

impl_inst!(BitCast, BitCast);
unop_explicitly_typed!(BitCast, "bitcast");

/// Compare integers, pointers, or vectors of integers or pointers.
/// See [LLVM 14 docs on the 'icmp' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#icmp-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct ICmp {
    pub predicate: IntPredicate,
    pub operand0: Operand,
    pub operand1: Operand,
    pub dest: Name,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(ICmp, ICmp);
impl_hasresult!(ICmp);

impl Display for ICmp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} = icmp {} {}, {}",
            &self.dest, &self.predicate, &self.operand0, &self.operand1,
        )?;
        // if self.debugloc.is_some() {
        // write!(f, " (with debugloc)")?;
        // }
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
}

impl_inst!(FCmp, FCmp);
impl_hasresult!(FCmp);

impl Display for FCmp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} = fcmp {} {}, {}",
            &self.dest, &self.predicate, &self.operand0, &self.operand1,
        )?;
        // if self.debugloc.is_some() {
        // write!(f, " (with debugloc)")?;
        // }
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
}

impl_inst!(Phi, Phi);
impl_hasresult!(Phi);

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
        // if self.debugloc.is_some() {
        // write!(f, " (with debugloc)")?;
        // }
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
}

impl_inst!(Select, Select);
impl_hasresult!(Select);

impl Display for Select {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} = select {}, {}, {}",
            &self.dest, &self.condition, &self.true_value, &self.false_value,
        )?;
        // if self.debugloc.is_some() {
        // write!(f, " (with debugloc)")?;
        // }
        Ok(())
    }
}

/// Function call.
/// See [LLVM 14 docs on the 'call' instruction](https://releases.llvm.org/14.0.0/docs/LangRef.html#call-instruction)
#[derive(PartialEq, Clone, Debug)]
pub struct Call {
    pub function: Operand,
    pub arguments: Vec<Operand>,
    pub dest: Option<Name>, // will be None if the `function` returns void
    pub function_attributes: Vec<Attribute>,
    pub is_tail_call: bool,
    pub debugloc: Option<DebugLoc>,
}

impl_inst!(Call, Call);

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
        write!(f, "call {}(", self.function,)?;
        for (i, arg) in self.arguments.iter().enumerate() {
            if i == self.arguments.len() - 1 {
                write!(f, "{arg}")?;
            } else {
                write!(f, "{arg}, ")?;
            }
        }
        write!(f, ")")?;
        // if self.debugloc.is_some() {
        // write!(f, " (with debugloc)")?;
        // }
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
}

impl_inst!(VAArg, VAArg);
impl_hasresult!(VAArg);

impl Display for VAArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} = va_arg {}, {}",
            &self.dest, &self.arg_list, &self.cur_type,
        )?;
        // if self.debugloc.is_some() {
        // write!(f, " (with debugloc)")?;
        // }
        Ok(())
    }
}
