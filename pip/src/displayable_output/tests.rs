// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use num_bigint::BigUint;
use num_complex::Complex;
use rustc_hash::FxHashMap;

use crate::displayable_output::DisplayableState;

#[test]
fn display_neg_zero() {
    let s = DisplayableState(
        vec![(BigUint::default(), Complex::new(-0.0, -0.0))]
            .into_iter()
            .collect::<FxHashMap<_, _>>(),
        1,
    );
    // -0 should be displayed as 0.0000 without a minus sign
    assert_eq!("STATE:\n|0⟩: 0.0000+0.0000𝑖", s.to_plain());
}

#[test]
fn display_rounds_to_neg_zero() {
    let s = DisplayableState(
        vec![(BigUint::default(), Complex::new(-0.00001, -0.00001))]
            .into_iter()
            .collect::<FxHashMap<_, _>>(),
        1,
    );
    // -0.00001 should be displayed as 0.0000 without a minus sign
    assert_eq!("STATE:\n|0⟩: 0.0000+0.0000𝑖", s.to_plain());
}
