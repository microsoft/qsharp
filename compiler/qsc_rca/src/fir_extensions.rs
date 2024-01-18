// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_fir::{
    fir::{CallableDecl, SpecBody, SpecGen},
    ty::{Prim, Ty},
};
use qsc_frontend::compile::RuntimeCapabilityFlags;

/// A trait that defines extension methods for `fir::CallableDecl`.
pub trait CallableDeclExtension {
    fn is_intrinsic(&self) -> bool;
}

impl CallableDeclExtension for CallableDecl {
    fn is_intrinsic(&self) -> bool {
        match self.body.body {
            SpecBody::Gen(spec_gen) => spec_gen == SpecGen::Intrinsic,
            _ => false,
        }
    }
}

/// A trait that defines extension methods for `fir::Ty`.
pub trait TyExtension {
    fn derive_runtime_capabilities(&self) -> RuntimeCapabilityFlags;
}

impl TyExtension for Ty {
    fn derive_runtime_capabilities(&self) -> RuntimeCapabilityFlags {
        fn derive_runtime_capabilities_from_primitive(prim: &Prim) -> RuntimeCapabilityFlags {
            match prim {
                Prim::BigInt => RuntimeCapabilityFlags::HigherLevelConstructs,
                Prim::Bool => RuntimeCapabilityFlags::ForwardBranching,
                Prim::Double => RuntimeCapabilityFlags::FloatingPointComputations,
                Prim::Int => RuntimeCapabilityFlags::IntegerComputations,
                Prim::Pauli => RuntimeCapabilityFlags::IntegerComputations,
                Prim::Qubit => RuntimeCapabilityFlags::empty(),
                Prim::Range | Prim::RangeFrom | Prim::RangeTo | Prim::RangeFull => {
                    RuntimeCapabilityFlags::IntegerComputations
                }
                Prim::Result => RuntimeCapabilityFlags::ForwardBranching,
                Prim::String => RuntimeCapabilityFlags::HigherLevelConstructs,
            }
        }

        fn derive_runtime_capabilities_from_tuple(tuple: &Vec<Ty>) -> RuntimeCapabilityFlags {
            let mut runtime_capabilities = RuntimeCapabilityFlags::empty();
            for item_type in tuple {
                let item_runtime_capabilities = item_type.derive_runtime_capabilities();
                runtime_capabilities |= item_runtime_capabilities;
            }
            runtime_capabilities
        }

        match self {
            // N.B. Derived array runtime capabilities can be more nuanced by taking into account the contained type.
            Ty::Array(_) => RuntimeCapabilityFlags::HigherLevelConstructs,
            // N.B. Derived array runtime capabilities can be more nuanced by taking into account the input and output
            // types.
            Ty::Arrow(_) => RuntimeCapabilityFlags::HigherLevelConstructs,
            Ty::Prim(prim) => derive_runtime_capabilities_from_primitive(prim),
            Ty::Tuple(tuple) => derive_runtime_capabilities_from_tuple(tuple),
            // N.B. Derived UDT runtime capabilities can be more nuanced by taking into account the type of each UDT
            // item.
            Ty::Udt(_) => RuntimeCapabilityFlags::HigherLevelConstructs,
            _ => panic!("Unexpected type"),
        }
    }
}
