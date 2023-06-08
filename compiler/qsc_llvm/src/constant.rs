// Portions copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::predicates::{FPPredicate, IntPredicate};
use crate::types::{FPType, NamedStructDef, Type, TypeRef, Typed, Types};
use std::convert::TryFrom;
use std::fmt::{self, Display};
use std::ops::Deref;
use std::sync::Arc;

/// See [LLVM 14 docs on Constants](https://releases.llvm.org/14.0.0/docs/LangRef.html#constants).
/// Constants can be either values, or expressions involving other constants (see [LLVM 14 docs on Constant Expressions](https://releases.llvm.org/14.0.0/docs/LangRef.html#constant-expressions)).
#[derive(PartialEq, Clone, Debug)]
pub enum Constant {
    Int {
        /// Number of bits in the constant integer
        bits: u32,
        /// The constant value itself.
        ///
        /// If `bits == 64`, this is the value.
        ///
        /// If `bits < 64`, the constant value is zero-extended to fit in this
        /// field.
        ///
        /// If `bits > 64`, the constant value is truncated to fit in this field;
        /// but if this truncation would change the value (i.e., if the value is
        /// >= 2^64 when interpreted as unsigned) then `Module::from_bc_path()`
        /// will fail. See [#5](https://github.com/cdisselkoen/llvm-ir/issues/5).
        //
        // Note that LLVM integers aren't signed or unsigned; each individual
        // instruction indicates whether it's treating the integer as signed or
        // unsigned if necessary (e.g., UDiv vs SDiv).
        value: u64,
    },
    Float(Float),
    /// The `TypeRef` here must be to a `PointerType`. See [LLVM 14 docs on Simple Constants](https://releases.llvm.org/14.0.0/docs/LangRef.html#simple-constants)
    Null(TypeRef),
    /// A zero-initialized array or struct (or scalar).
    AggregateZero(TypeRef),
    Struct {
        name: Option<String>, // llvm-hs-pure has Option<Name> here, but I don't think struct types can be numbered
        values: Vec<ConstantRef>,
        is_packed: bool,
    },
    Array {
        element_type: TypeRef,
        elements: Vec<ConstantRef>,
    },
    Vector(Vec<ConstantRef>),
    /// `Undef` can be used anywhere a constant is expected. See [LLVM 14 docs on Undefined Values](https://releases.llvm.org/14.0.0/docs/LangRef.html#undefined-values)
    Undef(TypeRef),
    /// See [LLVM 14 docs on Poison Values](https://releases.llvm.org/14.0.0/docs/LangRef.html#undefined-values)
    Poison(TypeRef),
    /// The address of the given (non-entry) [`BasicBlock`](../struct.BasicBlock.html). See [LLVM 14 docs on Addresses of Basic Blocks](https://releases.llvm.org/14.0.0/docs/LangRef.html#addresses-of-basic-blocks).
    /// `BlockAddress` needs more fields, but the necessary getter functions are apparently not exposed in the LLVM C API (only the C++ API)
    BlockAddress, // --TODO ideally we want BlockAddress { function: Name, block: Name },
    /// Global variable or function
    GlobalReference {
        /// Globals' names must be strings
        name: String,
        ty: TypeRef,
    },
    TokenNone,

    // Constants can also be expressed as operations applied to other constants:

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

    // Vector ops
    ExtractElement(ExtractElement),
    InsertElement(InsertElement),
    ShuffleVector(ShuffleVector),

    // Aggregate ops
    ExtractValue(ExtractValue),
    InsertValue(InsertValue),

    // Memory-related ops
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

    // Other ops
    ICmp(ICmp),
    FCmp(FCmp),
    Select(Select),
}

/// All of these `Float` variants should have data associated with them, but
/// Rust only has `f32` and `f64` floating-point types, and furthermore,
/// it's not clear how to get 16-, 80-, or 128-bit FP constant values through
/// the LLVM C API (the getters seem to only be exposed in the C++ API?)
#[derive(PartialEq, Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum Float {
    Half,   // TODO perhaps Half(u16)
    BFloat, // TODO perhaps BFloat(u16)
    Single(f32),
    Double(f64),
    Quadruple, // TODO perhaps Quadruple(u128)
    X86_FP80,  // TODO perhaps X86_FP80((u16, u64)) with the most-significant bits on the left
    PPC_FP128, // TODO perhaps PPC_FP128((u64, u64)) with the most-significant bits on the left
}

impl Typed for Float {
    fn get_type(&self, types: &Types) -> TypeRef {
        types.fp(match self {
            Float::Half => FPType::Half,
            Float::BFloat => FPType::BFloat,
            Float::Single(_) => FPType::Single,
            Float::Double(_) => FPType::Double,
            Float::Quadruple => FPType::FP128,
            Float::X86_FP80 => FPType::X86_FP80,
            Float::PPC_FP128 => FPType::PPC_FP128,
        })
    }
}

impl Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Float::Half => write!(f, "half"),
            Float::BFloat => write!(f, "bfloat"),
            Float::Single(s) => write!(f, "float {s}"),
            Float::Double(d) => write!(f, "double {d}"),
            Float::Quadruple => write!(f, "quadruple"),
            Float::X86_FP80 => write!(f, "x86_fp80"),
            Float::PPC_FP128 => write!(f, "ppc_fp128"),
        }
    }
}

impl Typed for Constant {
    fn get_type(&self, types: &Types) -> TypeRef {
        match self {
            Constant::Int { bits, .. } => types.int(*bits),
            Constant::Float(f) => types.type_of(f),
            Constant::Undef(t)
            | Constant::Null(t)
            | Constant::AggregateZero(t)
            | Constant::Poison(t) => t.clone(),
            Constant::Struct {
                values, is_packed, ..
            } => types.struct_of(
                values.iter().map(|v| types.type_of(v)).collect(),
                *is_packed,
            ),
            Constant::Array {
                element_type,
                elements,
            } => types.array_of(element_type.clone(), elements.len()),
            Constant::Vector(v) => types.vector_of(
                types.type_of(&v[0]),
                v.len(),
                false, // I don't think it's possible (at least as of LLVM 11) to have a constant of scalable vector type?
            ),
            Constant::BlockAddress { .. } => types.label_type(),
            Constant::GlobalReference { ty, .. } => types.pointer_to(ty.clone()),
            Constant::TokenNone => types.token_type(),
            Constant::Add(a) => types.type_of(a),
            Constant::Sub(s) => types.type_of(s),
            Constant::Mul(m) => types.type_of(m),
            Constant::UDiv(d) => types.type_of(d),
            Constant::SDiv(d) => types.type_of(d),
            Constant::URem(r) => types.type_of(r),
            Constant::SRem(r) => types.type_of(r),
            Constant::And(a) => types.type_of(a),
            Constant::Or(o) => types.type_of(o),
            Constant::Xor(x) => types.type_of(x),
            Constant::Shl(s) => types.type_of(s),
            Constant::LShr(l) => types.type_of(l),
            Constant::AShr(a) => types.type_of(a),
            Constant::FAdd(f) => types.type_of(f),
            Constant::FSub(f) => types.type_of(f),
            Constant::FMul(f) => types.type_of(f),
            Constant::FDiv(f) => types.type_of(f),
            Constant::FRem(f) => types.type_of(f),
            Constant::ExtractElement(e) => types.type_of(e),
            Constant::InsertElement(i) => types.type_of(i),
            Constant::ShuffleVector(s) => types.type_of(s),
            Constant::ExtractValue(e) => types.type_of(e),
            Constant::InsertValue(i) => types.type_of(i),
            Constant::GetElementPtr(g) => types.type_of(g),
            Constant::Trunc(t) => types.type_of(t),
            Constant::ZExt(z) => types.type_of(z),
            Constant::SExt(s) => types.type_of(s),
            Constant::FPTrunc(f) => types.type_of(f),
            Constant::FPExt(f) => types.type_of(f),
            Constant::FPToUI(f) => types.type_of(f),
            Constant::FPToSI(f) => types.type_of(f),
            Constant::UIToFP(u) => types.type_of(u),
            Constant::SIToFP(s) => types.type_of(s),
            Constant::PtrToInt(p) => types.type_of(p),
            Constant::IntToPtr(i) => types.type_of(i),
            Constant::BitCast(b) => types.type_of(b),
            Constant::AddrSpaceCast(a) => types.type_of(a),
            Constant::ICmp(i) => types.type_of(i),
            Constant::FCmp(f) => types.type_of(f),
            Constant::Select(s) => types.type_of(s),
        }
    }
}

impl Display for Constant {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Constant::Int { bits, value } => {
                if *bits == 1 {
                    if *value == 0 {
                        write!(f, "i1 false")
                    } else {
                        write!(f, "i1 true")
                    }
                } else {
                    write!(f, "i{} {}", bits, *value)
                }
            }
            Constant::Float(float) => write!(f, "{float}"),
            Constant::Null(ty) => write!(f, "{ty} null"),
            Constant::AggregateZero(ty) => write!(f, "{ty} zeroinitializer"),
            Constant::Struct {
                values, is_packed, ..
            } => {
                if *is_packed {
                    write!(f, "<")?;
                }
                write!(f, "{{ ")?;
                let (last, most) = values.split_last().expect("structs should not be empty");
                for val in most {
                    write!(f, "{val}, ")?;
                }
                write!(f, "{last}")?;
                write!(f, " }}")?;
                if *is_packed {
                    write!(f, ">")?;
                }
                Ok(())
            }
            Constant::Array { elements, .. } => {
                write!(f, "[ ")?;
                let (last, most) = elements.split_last().expect("array should not be empty");
                for elem in most {
                    write!(f, "{elem}, ")?;
                }
                write!(f, "{last}")?;
                write!(f, " ]")?;
                Ok(())
            }
            Constant::Vector(v) => {
                write!(f, "< ")?;
                let (last, most) = v.split_last().expect("array should not be empty");
                for elem in most {
                    write!(f, "{elem}, ")?;
                }
                write!(f, "{last}")?;
                write!(f, " >")?;
                Ok(())
            }
            Constant::Undef(ty) => write!(f, "{ty} undef"),
            Constant::Poison(ty) => write!(f, "{ty} poison"),
            Constant::BlockAddress => write!(f, "blockaddr"),
            Constant::GlobalReference { name, ty } => {
                match ty.as_ref() {
                    Type::FuncType { .. } => {
                        // function types: just write the name, not the type
                        write!(f, "@{name}")
                    }
                    _ => {
                        // non-function types: typical style with the type and name
                        write!(f, "{ty}* @{name}")
                    }
                }
            }
            Constant::TokenNone => write!(f, "none"),
            Constant::Add(a) => write!(f, "{a}"),
            Constant::Sub(s) => write!(f, "{s}"),
            Constant::Mul(m) => write!(f, "{m}"),
            Constant::UDiv(d) => write!(f, "{d}"),
            Constant::SDiv(d) => write!(f, "{d}"),
            Constant::URem(r) => write!(f, "{r}"),
            Constant::SRem(r) => write!(f, "{r}"),
            Constant::And(a) => write!(f, "{a}"),
            Constant::Or(o) => write!(f, "{o}"),
            Constant::Xor(x) => write!(f, "{x}"),
            Constant::Shl(s) => write!(f, "{s}"),
            Constant::LShr(l) => write!(f, "{l}"),
            Constant::AShr(a) => write!(f, "{a}"),
            Constant::FAdd(a) => write!(f, "{a}"),
            Constant::FSub(s) => write!(f, "{s}"),
            Constant::FMul(m) => write!(f, "{m}"),
            Constant::FDiv(d) => write!(f, "{d}"),
            Constant::FRem(r) => write!(f, "{r}"),
            Constant::ExtractElement(e) => write!(f, "{e}"),
            Constant::InsertElement(i) => write!(f, "{i}"),
            Constant::ShuffleVector(s) => write!(f, "{s}"),
            Constant::ExtractValue(e) => write!(f, "{e}"),
            Constant::InsertValue(i) => write!(f, "{i}"),
            Constant::GetElementPtr(g) => write!(f, "{g}"),
            Constant::Trunc(t) => write!(f, "{t}"),
            Constant::ZExt(z) => write!(f, "{z}"),
            Constant::SExt(s) => write!(f, "{s}"),
            Constant::FPTrunc(t) => write!(f, "{t}"),
            Constant::FPExt(e) => write!(f, "{e}"),
            Constant::FPToUI(t) => write!(f, "{t}"),
            Constant::FPToSI(t) => write!(f, "{t}"),
            Constant::UIToFP(t) => write!(f, "{t}"),
            Constant::SIToFP(t) => write!(f, "{t}"),
            Constant::PtrToInt(p) => write!(f, "{p}"),
            Constant::IntToPtr(i) => write!(f, "{i}"),
            Constant::BitCast(b) => write!(f, "{b}"),
            Constant::AddrSpaceCast(a) => write!(f, "{a}"),
            Constant::ICmp(i) => write!(f, "{i}"),
            Constant::FCmp(c) => write!(f, "{c}"),
            Constant::Select(s) => write!(f, "{s}"),
        }
    }
}

/// A `ConstantRef` is a reference to a [`Constant`](enum.Constant.html).
/// Most importantly, it implements `AsRef<Constant>` and `Deref<Target = Constant>`.
/// It also has a cheap `Clone` -- only the reference is cloned, not the
/// underlying `Constant`.
//
// `Arc` is used rather than `Rc` so that `Module` can remain `Sync`.
// This is important because it allows multiple threads to simultaneously access
// a single (immutable) `Module`.
#[derive(PartialEq, Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct ConstantRef(Arc<Constant>);

impl AsRef<Constant> for ConstantRef {
    fn as_ref(&self) -> &Constant {
        self.0.as_ref()
    }
}

impl Deref for ConstantRef {
    type Target = Constant;

    fn deref(&self) -> &Constant {
        &self.0
    }
}

impl Typed for ConstantRef {
    fn get_type(&self, types: &Types) -> TypeRef {
        self.as_ref().get_type(types)
    }
}

impl Display for ConstantRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl ConstantRef {
    /// Construct a new `ConstantRef` by consuming the given owned `Constant`.
    //
    // Internal users should get `ConstantRef`s from the `ModuleContext` cache
    // instead if possible, so that if we already have that `Constant`
    // somewhere, we can just give you a new `ConstantRef` to that `Constant`.
    #[must_use]
    pub fn new(c: Constant) -> Self {
        Self(Arc::new(c))
    }
}

pub trait ConstUnaryOp {
    fn get_operand(&self) -> ConstantRef;
}

pub trait ConstBinaryOp {
    fn get_operand0(&self) -> ConstantRef;
    fn get_operand1(&self) -> ConstantRef;
}

macro_rules! impl_constexpr {
    ($expr:ty, $id:ident) => {
        impl From<$expr> for Constant {
            fn from(expr: $expr) -> Constant {
                Constant::$id(expr)
            }
        }

        impl TryFrom<Constant> for $expr {
            type Error = &'static str;
            fn try_from(constant: Constant) -> Result<Self, Self::Error> {
                match constant {
                    Constant::$id(expr) => Ok(expr),
                    _ => Err("Constant is not of requested kind"),
                }
            }
        }
    };
}

// impls which are shared by all UnaryOps.
// If possible, prefer `unop_explicitly_typed!`, which provides additional impls
macro_rules! impl_unop {
    ($expr:ty) => {
        impl ConstUnaryOp for $expr {
            fn get_operand(&self) -> ConstantRef {
                self.operand.clone()
            }
        }
    };
}

// impls which are shared by all BinaryOps.
// If possible, prefer `binop_same_type!` or `binop_left_type!`, which
// provide additional impls
macro_rules! impl_binop {
    ($expr:ty, $dispname:expr) => {
        impl ConstBinaryOp for $expr {
            fn get_operand0(&self) -> ConstantRef {
                self.operand0.clone()
            }
            fn get_operand1(&self) -> ConstantRef {
                self.operand1.clone()
            }
        }
    };
}

// Use on binops where the result type is the same as both operand types
// (and the Display impl doesn't need to show any more information other than the operands)
macro_rules! binop_same_type {
    ($expr:ty, $dispname:expr) => {
        impl_binop!($expr, $dispname);

        impl Typed for $expr {
            fn get_type(&self, types: &Types) -> TypeRef {
                let t = types.type_of(&self.get_operand0());
                debug_assert_eq!(t, types.type_of(&self.get_operand1()));
                t
            }
        }

        impl Display for $expr {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{} ({}, {})", $dispname, &self.operand0, &self.operand1)
            }
        }
    };
}

// Use on binops where the result type is the same as the first operand type
// (and the Display impl doesn't need to show any more information other than the operands)
macro_rules! binop_left_type {
    ($expr:ty, $dispname:expr) => {
        impl_binop!($expr, $dispname);

        impl Typed for $expr {
            fn get_type(&self, types: &Types) -> TypeRef {
                types.type_of(&self.get_operand0())
            }
        }

        impl Display for $expr {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{} ({}, {})", $dispname, &self.operand0, &self.operand1)
            }
        }
    };
}

// Use on unops with a 'to_type' field which indicates the result type
macro_rules! unop_explicitly_typed {
    ($expr:ty, $dispname:expr) => {
        impl_unop!($expr);

        impl Typed for $expr {
            fn get_type(&self, _types: &Types) -> TypeRef {
                self.to_type.clone()
            }
        }

        impl Display for $expr {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    f,
                    "{} ({} to {})",
                    $dispname,
                    &self.get_operand(),
                    &self.to_type,
                )
            }
        }
    };
}

#[derive(PartialEq, Clone, Debug)]
pub struct Add {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
    // pub nsw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    // pub nuw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
}

impl_constexpr!(Add, Add);
binop_same_type!(Add, "add");

#[derive(PartialEq, Clone, Debug)]
pub struct Sub {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
    // pub nsw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    // pub nuw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
}

impl_constexpr!(Sub, Sub);
binop_same_type!(Sub, "sub");

#[derive(PartialEq, Clone, Debug)]
pub struct Mul {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
    // pub nsw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    // pub nuw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
}

impl_constexpr!(Mul, Mul);
binop_same_type!(Mul, "mul");

#[derive(PartialEq, Clone, Debug)]
pub struct UDiv {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
    // pub exact: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
}

impl_constexpr!(UDiv, UDiv);
binop_same_type!(UDiv, "udiv");

#[derive(PartialEq, Clone, Debug)]
pub struct SDiv {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
    // pub exact: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
}

impl_constexpr!(SDiv, SDiv);
binop_same_type!(SDiv, "sdiv");

#[derive(PartialEq, Clone, Debug)]
pub struct URem {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(URem, URem);
binop_same_type!(URem, "urem");

#[derive(PartialEq, Clone, Debug)]
pub struct SRem {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(SRem, SRem);
binop_same_type!(SRem, "srem");

#[derive(PartialEq, Clone, Debug)]
pub struct And {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(And, And);
binop_same_type!(And, "and");

#[derive(PartialEq, Clone, Debug)]
pub struct Or {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(Or, Or);
binop_same_type!(Or, "or");

#[derive(PartialEq, Clone, Debug)]
pub struct Xor {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(Xor, Xor);
binop_same_type!(Xor, "xor");

#[derive(PartialEq, Clone, Debug)]
pub struct Shl {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
    // pub nsw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    // pub nuw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
}

impl_constexpr!(Shl, Shl);
binop_left_type!(Shl, "shl");

#[derive(PartialEq, Clone, Debug)]
pub struct LShr {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
    // pub exact: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
}

impl_constexpr!(LShr, LShr);
binop_left_type!(LShr, "lshr");

#[derive(PartialEq, Clone, Debug)]
pub struct AShr {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
    // pub exact: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
}

impl_constexpr!(AShr, AShr);
binop_left_type!(AShr, "ashr");

#[derive(PartialEq, Clone, Debug)]
pub struct FAdd {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(FAdd, FAdd);
binop_same_type!(FAdd, "fadd");

#[derive(PartialEq, Clone, Debug)]
pub struct FSub {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(FSub, FSub);
binop_same_type!(FSub, "fsub");

#[derive(PartialEq, Clone, Debug)]
pub struct FMul {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(FMul, FMul);
binop_same_type!(FMul, "fmul");

#[derive(PartialEq, Clone, Debug)]
pub struct FDiv {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(FDiv, FDiv);
binop_same_type!(FDiv, "fdiv");

#[derive(PartialEq, Clone, Debug)]
pub struct FRem {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(FRem, FRem);
binop_same_type!(FRem, "frem");

#[derive(PartialEq, Clone, Debug)]
pub struct ExtractElement {
    pub vector: ConstantRef,
    pub index: ConstantRef,
}

impl_constexpr!(ExtractElement, ExtractElement);

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
        write!(f, "extractelement ({}, {})", &self.vector, &self.index)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct InsertElement {
    pub vector: ConstantRef,
    pub element: ConstantRef,
    pub index: ConstantRef,
}

impl_constexpr!(InsertElement, InsertElement);

impl Typed for InsertElement {
    fn get_type(&self, types: &Types) -> TypeRef {
        types.type_of(&self.vector)
    }
}

impl Display for InsertElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "insertelement ({}, {}, {})",
            &self.vector, &self.element, &self.index,
        )
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct ShuffleVector {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
    pub mask: ConstantRef,
}

impl_constexpr!(ShuffleVector, ShuffleVector);
impl_binop!(ShuffleVector, "shufflevector");

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
            "shufflevector ({}, {}, {})",
            &self.operand0, &self.operand1, &self.mask,
        )
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct ExtractValue {
    pub aggregate: ConstantRef,
    pub indices: Vec<u32>,
}

impl_constexpr!(ExtractValue, ExtractValue);

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
        write!(f, "extractvalue ({}", &self.aggregate)?;
        for idx in &self.indices {
            write!(f, ", {idx}")?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct InsertValue {
    pub aggregate: ConstantRef,
    pub element: ConstantRef,
    pub indices: Vec<u32>,
}

impl_constexpr!(InsertValue, InsertValue);

impl Typed for InsertValue {
    fn get_type(&self, types: &Types) -> TypeRef {
        types.type_of(&self.aggregate)
    }
}

impl Display for InsertValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "insertvalue ({}, {}", &self.aggregate, &self.element)?;
        for idx in &self.indices {
            write!(f, ", {idx}")?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct GetElementPtr {
    pub address: ConstantRef,
    pub indices: Vec<ConstantRef>,
    pub in_bounds: bool,
}

impl_constexpr!(GetElementPtr, GetElementPtr);

impl Typed for GetElementPtr {
    fn get_type(&self, types: &Types) -> TypeRef {
        gep_type(types.type_of(&self.address), self.indices.iter(), types)
    }
}

fn gep_type<'c>(
    cur_type: TypeRef,
    mut indices: impl Iterator<Item = &'c ConstantRef>,
    types: &Types,
) -> TypeRef {
    match indices.next() {
        None => types.pointer_to(cur_type), // iterator is done
        Some(index) => match cur_type.as_ref() {
            Type::PointerType { pointee_type, .. } => gep_type(pointee_type.clone(), indices, types),
            Type::VectorType { element_type, .. } | Type::ArrayType { element_type, .. } => gep_type(element_type.clone(), indices, types),
            Type::StructType { element_types, .. } => {
                if let Constant::Int { value, .. } = index.as_ref() {
                    gep_type(
                        element_types.get(*value as usize).cloned().expect("GEP index out of range"),
                        indices,
                        types,
                    )
                } else {
                    panic!("Expected GEP index on a constant struct to be a Constant::Int; got {index:?}")
                }
            },
            Type::NamedStructType { name } => match types.named_struct_def(name) {
                None => panic!("Named struct with no definition (struct name {name:?})"),
                Some(NamedStructDef::Opaque) => panic!("GEP on an opaque struct type"),
                Some(NamedStructDef::Defined(ty)) => match ty.as_ref() {
                    Type::StructType { element_types, .. } => {
                        if let Constant::Int { value, .. } = index.as_ref() {
                            gep_type(element_types.get(*value as usize).cloned().expect("GEP index out of range"), indices, types)
                        } else {
                            panic!("Expected GEP index on a struct to be a Constant::Int; got {index:?}")
                        }
                    },
                    ty => panic!("Expected NamedStructDef inner type to be a StructType; got {ty:?}"),
                },
            }
            _ => panic!("Expected GEP base type to be a PointerType, VectorType, ArrayType, StructType, or NamedStructType; got {cur_type:?}"),
        },
    }
}

impl Display for GetElementPtr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "getelementptr{} ({}",
            if self.in_bounds { " inbounds" } else { "" },
            &self.address
        )?;
        for idx in &self.indices {
            write!(f, ", {idx}")?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Trunc {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(Trunc, Trunc);
unop_explicitly_typed!(Trunc, "trunc");

#[derive(PartialEq, Clone, Debug)]
pub struct ZExt {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(ZExt, ZExt);
unop_explicitly_typed!(ZExt, "zext");

#[derive(PartialEq, Clone, Debug)]
pub struct SExt {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(SExt, SExt);
unop_explicitly_typed!(SExt, "sext");

#[derive(PartialEq, Clone, Debug)]
pub struct FPTrunc {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(FPTrunc, FPTrunc);
unop_explicitly_typed!(FPTrunc, "fptrunc");

#[derive(PartialEq, Clone, Debug)]
pub struct FPExt {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(FPExt, FPExt);
unop_explicitly_typed!(FPExt, "fpext");

#[derive(PartialEq, Clone, Debug)]
pub struct FPToUI {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(FPToUI, FPToUI);
unop_explicitly_typed!(FPToUI, "fptoui");

#[derive(PartialEq, Clone, Debug)]
pub struct FPToSI {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(FPToSI, FPToSI);
unop_explicitly_typed!(FPToSI, "fptosi");

#[derive(PartialEq, Clone, Debug)]
pub struct UIToFP {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(UIToFP, UIToFP);
unop_explicitly_typed!(UIToFP, "uitofp");

#[derive(PartialEq, Clone, Debug)]
pub struct SIToFP {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(SIToFP, SIToFP);
unop_explicitly_typed!(SIToFP, "sitofp");

#[derive(PartialEq, Clone, Debug)]
pub struct PtrToInt {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(PtrToInt, PtrToInt);
unop_explicitly_typed!(PtrToInt, "ptrtoint");

#[derive(PartialEq, Clone, Debug)]
pub struct IntToPtr {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(IntToPtr, IntToPtr);
unop_explicitly_typed!(IntToPtr, "inttoptr");

#[derive(PartialEq, Clone, Debug)]
pub struct BitCast {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(BitCast, BitCast);
unop_explicitly_typed!(BitCast, "bitcast");

#[derive(PartialEq, Clone, Debug)]
pub struct AddrSpaceCast {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(AddrSpaceCast, AddrSpaceCast);
unop_explicitly_typed!(AddrSpaceCast, "addrspacecast");

#[derive(PartialEq, Clone, Debug)]
pub struct ICmp {
    pub predicate: IntPredicate,
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(ICmp, ICmp);
impl_binop!(ICmp, "icmp");

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
            "icmp {} ({}, {})",
            &self.predicate, &self.operand0, &self.operand1,
        )
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct FCmp {
    pub predicate: FPPredicate,
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(FCmp, FCmp);
impl_binop!(FCmp, "fcmp");

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
            "fcmp {} ({}, {})",
            &self.predicate, &self.operand0, &self.operand1,
        )
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Select {
    pub condition: ConstantRef,
    pub true_value: ConstantRef,
    pub false_value: ConstantRef,
}

impl_constexpr!(Select, Select);

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
            "select ({}, {}, {})",
            &self.condition, &self.true_value, &self.false_value,
        )
    }
}
