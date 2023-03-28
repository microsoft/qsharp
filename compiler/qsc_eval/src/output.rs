// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use num_bigint::BigUint;
use num_complex::Complex64;

pub trait Receiver {
    fn state(&self, state: Vec<(BigUint, Complex64)>);
    fn message(&self, msg: String);
}

#[derive(Default)]
pub struct StdoutReceiver {}

impl Receiver for StdoutReceiver {
    fn state(&self, state: Vec<(BigUint, Complex64)>) {
        println!("{state:?}");
    }

    fn message(&self, msg: String) {
        println!("{msg}");
    }
}
