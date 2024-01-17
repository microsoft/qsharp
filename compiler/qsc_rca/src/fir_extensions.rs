// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_fir::{
    fir::{CallableDecl, SpecBody, SpecGen},
    ty::{Prim, Ty},
};
use qsc_frontend::compile::RuntimeCapabilityFlags;

/// A trait that defines extension methods for `fir::CallableDecl`.
pub trait CallableDeclExt {
    fn is_intrinsic(&self) -> bool;
}

impl CallableDeclExt for CallableDecl {
    fn is_intrinsic(&self) -> bool {
        match self.body.body {
            SpecBody::Gen(spec_gen) => spec_gen == SpecGen::Intrinsic,
            _ => false,
        }
    }
}

/// A trait that defines extension methods for `fir::Ty`.
pub trait TyExt {
    fn infer_rt_caps(&self) -> RuntimeCapabilityFlags;
}

impl TyExt for Ty {
    fn infer_rt_caps(&self) -> RuntimeCapabilityFlags {
        fn infer_rt_caps_from_prim(prim: &Prim) -> RuntimeCapabilityFlags {
            match prim {
                Prim::BigInt => RuntimeCapabilityFlags::HigherLevelConstructs,
                Prim::Bool => RuntimeCapabilityFlags::ConditionalForwardBranching,
                Prim::Double => RuntimeCapabilityFlags::FloatingPointComputation,
                Prim::Int => RuntimeCapabilityFlags::IntegerComputations,
                Prim::Pauli => RuntimeCapabilityFlags::IntegerComputations,
                Prim::Qubit => RuntimeCapabilityFlags::empty(),
                Prim::Range | Prim::RangeFrom | Prim::RangeTo | Prim::RangeFull => {
                    RuntimeCapabilityFlags::IntegerComputations
                }
                Prim::Result => RuntimeCapabilityFlags::ConditionalForwardBranching,
                Prim::String => RuntimeCapabilityFlags::HigherLevelConstructs,
            }
        }

        fn infer_rt_caps_from_tuple(tuple: &Vec<Ty>) -> RuntimeCapabilityFlags {
            let mut rt_caps = RuntimeCapabilityFlags::empty();
            for item_type in tuple {
                let item_rt_caps = item_type.infer_rt_caps();
                rt_caps |= item_rt_caps;
            }
            rt_caps
        }

        match self {
            // N.B. Inferred array runtime capabilities can be more nuanced by taking into account the contained type.
            Ty::Array(_) => RuntimeCapabilityFlags::HigherLevelConstructs,
            // N.B. Inferred array runtime capabilities can be more nuanced by taking into account the input and output
            // types.
            Ty::Arrow(_) => RuntimeCapabilityFlags::HigherLevelConstructs,
            Ty::Prim(prim) => infer_rt_caps_from_prim(prim),
            Ty::Tuple(tuple) => infer_rt_caps_from_tuple(tuple),
            // N.B. Inferred UDT runtime capabilities can be more nuanced by taking into account the type of each UDT
            // item.
            Ty::Udt(_) => RuntimeCapabilityFlags::HigherLevelConstructs,
            _ => panic!("Unexpected type"),
        }
    }
}
