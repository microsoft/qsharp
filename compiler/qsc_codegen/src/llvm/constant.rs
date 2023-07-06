// Copyright (c) 2019 Craig Disselkoen
// Portions copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::types::{Type, TypeRef};
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
        // Note that LLVM integers aren't signed or unsigned; each individual
        // instruction indicates whether it's treating the integer as signed or
        // unsigned if necessary (e.g., UDiv vs SDiv).
        value: u64,
    },
    Float(Double),
    /// The `TypeRef` here must be to a `PointerType`. See [LLVM 14 docs on Simple Constants](https://releases.llvm.org/14.0.0/docs/LangRef.html#simple-constants)
    Null(TypeRef),
    /// A zero-initialized array or struct (or scalar).
    AggregateZero(TypeRef),
    Struct {
        name: Option<String>,
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
}

#[derive(PartialEq, Clone, Debug)]
pub struct Double(pub f64);

impl Display for Double {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "double {}", self.0)
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
        }
    }
}

impl Constant {
    #[allow(clippy::too_many_lines)]
    pub fn fmt_without_type(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Constant::Int { bits, value } => {
                if *bits == 1 {
                    if *value == 0 {
                        write!(f, "false")
                    } else {
                        write!(f, "true")
                    }
                } else {
                    write!(f, "{}", *value)
                }
            }
            Constant::Float(float) => write!(f, "{}", float.0),
            Constant::Null(_) => write!(f, "null"),
            Constant::AggregateZero(_) => write!(f, "zeroinitializer"),
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
            Constant::Undef(_) => write!(f, "undef"),
            Constant::Poison(_) => write!(f, "poison"),
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
