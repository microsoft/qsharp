// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_fir::{
    fir::{CallableDecl, SpecBody, SpecGen},
    ty::Ty,
};

/// A trait that defines extension methods for `fir::CallableDecl`.
pub trait CallableDeclExtension {
    /// Returns true if the callable is intrinsic.
    fn is_intrinsic(&self) -> bool;
    /// Returns true if the callable's output is unit.
    fn is_output_unit(&self) -> bool;
}

impl CallableDeclExtension for CallableDecl {
    fn is_intrinsic(&self) -> bool {
        match self.body.body {
            SpecBody::Gen(spec_gen) => spec_gen == SpecGen::Intrinsic,
            _ => false,
        }
    }

    fn is_output_unit(&self) -> bool {
        self.output == Ty::UNIT
    }
}
