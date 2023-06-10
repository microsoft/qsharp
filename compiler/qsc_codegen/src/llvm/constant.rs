// Portions copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::predicates::{FPPredicate, IntPredicate};
use super::types::{Type, TypeRef};
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
    Half,
    BFloat,
    Single(f32),
    Double(f64),
    Quadruple,
    X86_FP80,
    PPC_FP128,
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
            Constant::GlobalReference { name, ty } => {
                match ty.as_ref() {
                    Type::Func { .. } => {
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
pub struct ConstantRef(pub Arc<Constant>);

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
macro_rules! impl_unop {
    ($expr:ty, $dispname:expr) => {
        impl ConstUnaryOp for $expr {
            fn get_operand(&self) -> ConstantRef {
                self.operand.clone()
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

// impls which are shared by all BinaryOps.
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

        impl Display for $expr {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{} ({}, {})", $dispname, &self.operand0, &self.operand1)
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
impl_binop!(Add, "add");

#[derive(PartialEq, Clone, Debug)]
pub struct Sub {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
    // pub nsw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    // pub nuw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
}

impl_constexpr!(Sub, Sub);
impl_binop!(Sub, "sub");

#[derive(PartialEq, Clone, Debug)]
pub struct Mul {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
    // pub nsw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    // pub nuw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
}

impl_constexpr!(Mul, Mul);
impl_binop!(Mul, "mul");

#[derive(PartialEq, Clone, Debug)]
pub struct UDiv {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
    // pub exact: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
}

impl_constexpr!(UDiv, UDiv);
impl_binop!(UDiv, "udiv");

#[derive(PartialEq, Clone, Debug)]
pub struct SDiv {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
    // pub exact: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
}

impl_constexpr!(SDiv, SDiv);
impl_binop!(SDiv, "sdiv");

#[derive(PartialEq, Clone, Debug)]
pub struct URem {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(URem, URem);
impl_binop!(URem, "urem");

#[derive(PartialEq, Clone, Debug)]
pub struct SRem {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(SRem, SRem);
impl_binop!(SRem, "srem");

#[derive(PartialEq, Clone, Debug)]
pub struct And {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(And, And);
impl_binop!(And, "and");

#[derive(PartialEq, Clone, Debug)]
pub struct Or {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(Or, Or);
impl_binop!(Or, "or");

#[derive(PartialEq, Clone, Debug)]
pub struct Xor {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(Xor, Xor);
impl_binop!(Xor, "xor");

#[derive(PartialEq, Clone, Debug)]
pub struct Shl {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
    // pub nsw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
    // pub nuw: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
}

impl_constexpr!(Shl, Shl);
impl_binop!(Shl, "shl");

#[derive(PartialEq, Clone, Debug)]
pub struct LShr {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
    // pub exact: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
}

impl_constexpr!(LShr, LShr);
impl_binop!(LShr, "lshr");

#[derive(PartialEq, Clone, Debug)]
pub struct AShr {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
    // pub exact: bool,  // getters for these seem to not be exposed in the LLVM C API, only in the C++ one
}

impl_constexpr!(AShr, AShr);
impl_binop!(AShr, "ashr");

#[derive(PartialEq, Clone, Debug)]
pub struct FAdd {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(FAdd, FAdd);
impl_binop!(FAdd, "fadd");

#[derive(PartialEq, Clone, Debug)]
pub struct FSub {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(FSub, FSub);
impl_binop!(FSub, "fsub");

#[derive(PartialEq, Clone, Debug)]
pub struct FMul {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(FMul, FMul);
impl_binop!(FMul, "fmul");

#[derive(PartialEq, Clone, Debug)]
pub struct FDiv {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(FDiv, FDiv);
impl_binop!(FDiv, "fdiv");

#[derive(PartialEq, Clone, Debug)]
pub struct FRem {
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(FRem, FRem);
impl_binop!(FRem, "frem");

#[derive(PartialEq, Clone, Debug)]
pub struct ExtractElement {
    pub vector: ConstantRef,
    pub index: ConstantRef,
}

impl_constexpr!(ExtractElement, ExtractElement);

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

impl ConstBinaryOp for ShuffleVector {
    fn get_operand0(&self) -> ConstantRef {
        self.operand0.clone()
    }
    fn get_operand1(&self) -> ConstantRef {
        self.operand1.clone()
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
impl_unop!(Trunc, "trunc");

#[derive(PartialEq, Clone, Debug)]
pub struct ZExt {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(ZExt, ZExt);
impl_unop!(ZExt, "zext");

#[derive(PartialEq, Clone, Debug)]
pub struct SExt {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(SExt, SExt);
impl_unop!(SExt, "sext");

#[derive(PartialEq, Clone, Debug)]
pub struct FPTrunc {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(FPTrunc, FPTrunc);
impl_unop!(FPTrunc, "fptrunc");

#[derive(PartialEq, Clone, Debug)]
pub struct FPExt {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(FPExt, FPExt);
impl_unop!(FPExt, "fpext");

#[derive(PartialEq, Clone, Debug)]
pub struct FPToUI {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(FPToUI, FPToUI);
impl_unop!(FPToUI, "fptoui");

#[derive(PartialEq, Clone, Debug)]
pub struct FPToSI {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(FPToSI, FPToSI);
impl_unop!(FPToSI, "fptosi");

#[derive(PartialEq, Clone, Debug)]
pub struct UIToFP {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(UIToFP, UIToFP);
impl_unop!(UIToFP, "uitofp");

#[derive(PartialEq, Clone, Debug)]
pub struct SIToFP {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(SIToFP, SIToFP);
impl_unop!(SIToFP, "sitofp");

#[derive(PartialEq, Clone, Debug)]
pub struct PtrToInt {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(PtrToInt, PtrToInt);
impl_unop!(PtrToInt, "ptrtoint");

#[derive(PartialEq, Clone, Debug)]
pub struct IntToPtr {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(IntToPtr, IntToPtr);
impl_unop!(IntToPtr, "inttoptr");

#[derive(PartialEq, Clone, Debug)]
pub struct BitCast {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(BitCast, BitCast);
impl_unop!(BitCast, "bitcast");

#[derive(PartialEq, Clone, Debug)]
pub struct AddrSpaceCast {
    pub operand: ConstantRef,
    pub to_type: TypeRef,
}

impl_constexpr!(AddrSpaceCast, AddrSpaceCast);
impl_unop!(AddrSpaceCast, "addrspacecast");

#[derive(PartialEq, Clone, Debug)]
pub struct ICmp {
    pub predicate: IntPredicate,
    pub operand0: ConstantRef,
    pub operand1: ConstantRef,
}

impl_constexpr!(ICmp, ICmp);
impl ConstBinaryOp for ICmp {
    fn get_operand0(&self) -> ConstantRef {
        self.operand0.clone()
    }
    fn get_operand1(&self) -> ConstantRef {
        self.operand1.clone()
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
impl ConstBinaryOp for FCmp {
    fn get_operand0(&self) -> ConstantRef {
        self.operand0.clone()
    }
    fn get_operand1(&self) -> ConstantRef {
        self.operand1.clone()
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

impl Display for Select {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "select ({}, {}, {})",
            &self.condition, &self.true_value, &self.false_value,
        )
    }
}
