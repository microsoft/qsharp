// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_fir::fir::{CallableDecl, SpecBody, SpecGen};

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
