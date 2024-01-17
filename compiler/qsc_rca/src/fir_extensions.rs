// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_fir::fir::{CallableDecl, SpecBody, SpecGen};

/// A trait that implements extesion methods for `fir::CallableDecl`.
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
