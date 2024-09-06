// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::fmt::{self, Display, Formatter};

use oq3_semantics::types::ArrayDims;
use qsc::Span;
use rustc_hash::FxHashMap;

thread_local! {
    /// <https://github.com/openqasm/openqasm/blob/main/examples/stdgates.inc>
    pub static GATE_MAP: FxHashMap<&'static str, &'static str> = {
        let mut m = FxHashMap::default();
        // p is rz, should have been replaced by rz by transpile

        m.insert("x", "X");
        m.insert("y", "Y");
        m.insert("z", "Z");

        m.insert("h", "H");

        m.insert("s", "S");
        m.insert("sdg", "sdg");

        m.insert("t", "T");
        m.insert("tdg", "tdg");

        // sx q is Rx(pi/2, q), should have been replaced by Rx by transpile

        m.insert("crx", "crx");
        m.insert("cry", "cry");
        m.insert("crz", "crz");

        m.insert("rx", "Rx");
        m.insert("ry", "Ry");
        m.insert("rz", "Rz");

        m.insert("rxx", "Rxx");
        m.insert("ryy", "Ryy");
        m.insert("rzz", "Rzz");

        m.insert("cx", "CNOT");
        m.insert("cy", "CY");
        m.insert("cz", "CZ");

        // cp (controlled-phase), should have been replaced by transpile

        m.insert("ch", "ch");

        m.insert("id", "I");

        m.insert("swap", "SWAP");

        m.insert("ccx", "CCNOT");

        // cswap (controlled-swap), should have been replaced by transpile

        // cu (controlled-U), should have been replaced by transpile

        // openqasm 2.0 gates should have been replaced by transpile
        // CX, phase, cphase, id, u1, u2, u3
        m
    };
}

pub(crate) fn get_qsharp_gate_name<S: AsRef<str>>(gate_name: S) -> Option<&'static str> {
    GATE_MAP.with(|map| map.get(gate_name.as_ref()).copied())
}

/// When compiling QASM3 expressions, we need to keep track of the sematic QASM
/// type of the expression. This allows us to perform type checking and casting
/// when necessary.
#[derive(Debug, Clone, PartialEq)]
pub struct QasmTypedExpr {
    pub ty: oq3_semantics::types::Type,
    pub expr: qsc::ast::Expr,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GateModifier {
    /// The `adjoint` modifier.
    Inv(Span),
    Pow(Option<i64>, Span),
    Ctrl(Option<usize>, Span),
    NegCtrl(Option<usize>, Span),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Complex {
    pub real: f64,
    pub imaginary: f64,
}

impl Complex {
    pub fn new(real: f64, imaginary: f64) -> Self {
        Self { real, imaginary }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Type {
    Bool(bool),
    BigInt(bool),
    Complex(bool),
    Int(bool),
    Double(bool),
    Qubit,
    Result(bool),
    Tuple(Vec<Type>),
    Range,
    BoolArray(ArrayDimensions, bool),
    BigIntArray(ArrayDimensions, bool),
    IntArray(ArrayDimensions, bool),
    DoubleArray(ArrayDimensions),
    QubitArray(ArrayDimensions),
    ResultArray(ArrayDimensions, bool),
    TupleArray(ArrayDimensions, Vec<Type>),
    /// Function or operation, with the number of classical parameters and qubits.
    Callable(CallableKind, usize, usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallableKind {
    /// A function.
    #[allow(dead_code)]
    Function,
    /// An operation.
    Operation,
}

impl Display for CallableKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CallableKind::Function => write!(f, "Function"),
            CallableKind::Operation => write!(f, "Operation"),
        }
    }
}

/// QASM supports up to seven dimensions, but we are going to limit it to three.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayDimensions {
    One(usize),
    Two(usize, usize),
    Three(usize, usize, usize),
}

impl From<&ArrayDims> for ArrayDimensions {
    fn from(value: &ArrayDims) -> Self {
        match value {
            ArrayDims::D1(dim) => ArrayDimensions::One(*dim),
            ArrayDims::D2(dim1, dim2) => ArrayDimensions::Two(*dim1, *dim2),
            ArrayDims::D3(dim1, dim2, dim3) => ArrayDimensions::Three(*dim1, *dim2, *dim3),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Type::Bool(_) => write!(f, "bool"),
            Type::BigInt(_) => write!(f, "BigInt"),
            Type::Complex(_) => write!(f, "Complex"),
            Type::Int(_) => write!(f, "Int"),
            Type::Double(_) => write!(f, "Double"),
            Type::Qubit => write!(f, "Qubit"),
            Type::Range => write!(f, "Range"),
            Type::Result(_) => write!(f, "Result"),
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{ty}")?;
                }
                write!(f, ")")
            }
            Type::BoolArray(dim, _) => write!(f, "bool{dim}"),
            Type::BigIntArray(dim, _) => write!(f, "BigInt{dim}"),
            Type::IntArray(dim, _) => write!(f, "Int{dim}"),
            Type::DoubleArray(dim) => write!(f, "Double{dim}"),
            Type::QubitArray(dim) => write!(f, "Qubit{dim}"),
            Type::ResultArray(dim, _) => write!(f, "Result{dim}"),
            Type::TupleArray(dim, types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{ty}")?;
                }
                write!(f, "){dim}")
            }
            Type::Callable(kind, num_classical, num_qubits) => {
                write!(f, "Callable({kind}, {num_classical}, {num_qubits})")
            }
        }
    }
}

impl Display for ArrayDimensions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ArrayDimensions::One(..) => write!(f, "[]"),
            ArrayDimensions::Two(..) => write!(f, "[][]"),
            ArrayDimensions::Three(..) => write!(f, "[][][]"),
        }
    }
}

/// Get the indexed type of a given type.
/// For example, if the type is `Int[2][3]`, the indexed type is `Int[2]`.
/// If the type is `Int[2]`, the indexed type is `Int`.
/// If the type is `Int`, the indexed type is `None`.
///
/// This is useful for determining the type of an array element.
pub(crate) fn get_indexed_type(
    ty: &oq3_semantics::types::Type,
) -> Option<oq3_semantics::types::Type> {
    use oq3_semantics::types::{IsConst, Type};
    let ty = match &ty {
        Type::AngleArray(dims) => match dims {
            ArrayDims::D1(_) => Type::Angle(None, IsConst::False),
            ArrayDims::D2(l, _) => Type::AngleArray(ArrayDims::D1(*l)),
            ArrayDims::D3(l, w, _) => Type::AngleArray(ArrayDims::D2(*l, *w)),
        },
        Type::BitArray(dims, is_const) => match dims {
            ArrayDims::D1(_) => Type::Bit(is_const.clone()),
            ArrayDims::D2(l, _) => Type::BitArray(ArrayDims::D1(*l), is_const.clone()),
            ArrayDims::D3(l, w, _) => Type::BitArray(ArrayDims::D2(*l, *w), is_const.clone()),
        },
        Type::BoolArray(dims) => match dims {
            ArrayDims::D1(_) => Type::Bool(IsConst::False),
            ArrayDims::D2(l, _) => Type::BoolArray(ArrayDims::D1(*l)),
            ArrayDims::D3(l, w, _) => Type::BoolArray(ArrayDims::D2(*l, *w)),
        },
        Type::ComplexArray(dims) => match dims {
            ArrayDims::D1(_) => Type::Complex(None, IsConst::False),
            ArrayDims::D2(l, _) => Type::ComplexArray(ArrayDims::D1(*l)),
            ArrayDims::D3(l, w, _) => Type::ComplexArray(ArrayDims::D2(*l, *w)),
        },
        Type::FloatArray(dims) => match dims {
            ArrayDims::D1(_) => Type::Float(None, IsConst::False),
            ArrayDims::D2(l, _) => Type::FloatArray(ArrayDims::D1(*l)),
            ArrayDims::D3(l, w, _) => Type::FloatArray(ArrayDims::D2(*l, *w)),
        },
        Type::IntArray(dims) => match dims {
            ArrayDims::D1(_) => Type::Int(None, IsConst::False),
            ArrayDims::D2(l, _) => Type::IntArray(ArrayDims::D1(*l)),
            ArrayDims::D3(l, w, _) => Type::IntArray(ArrayDims::D2(*l, *w)),
        },
        Type::QubitArray(dims) => match dims {
            ArrayDims::D1(_) => Type::Qubit,
            ArrayDims::D2(l, _) => Type::QubitArray(ArrayDims::D1(*l)),
            ArrayDims::D3(l, w, _) => Type::QubitArray(ArrayDims::D2(*l, *w)),
        },
        Type::UIntArray(dims) => match dims {
            ArrayDims::D1(_) => Type::UInt(None, IsConst::False),
            ArrayDims::D2(l, _) => Type::UIntArray(ArrayDims::D1(*l)),
            ArrayDims::D3(l, w, _) => Type::UIntArray(ArrayDims::D2(*l, *w)),
        },
        _ => return None,
    };
    Some(ty)
}
