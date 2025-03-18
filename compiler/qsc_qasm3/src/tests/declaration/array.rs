// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod bit;
mod qubit;

use crate::tests::{compile_fragments, fail_on_compilation_errors};
use miette::Report;

#[test]
#[ignore = "unimplemented"]
fn arrays() -> miette::Result<(), Vec<Report>> {
    let source = "
        array[uint[16], 1] x;
        array[int[8], 4] x;
        array[float[64], 4, 2] x;
        array[angle[32], 4, 3, 2] x;
        array[bit[8], 2] x;
        array[bit[16], 2, 2] x;
        array[complex[float[32]], 4] x;
        array[bool, 3] x;
        array[int[8], 4] x = {1, 2, 3, 4};
        array[int[8], 4] x = y;
        array[int[8], 2] x = {y, y+y};
        array[uint[32], 2, 2] x = {{3, 4}, {2-3, 5*y}};
        array[uint[32], 2, 2] x = {z, {2-3, 5*y}};
        array[uint[32], 2, 2] x = {2*z, {1, 2}};
        array[uint[32], 2, 2] x = y;
    ";

    let unit = compile_fragments(source)?;
    fail_on_compilation_errors(&unit);
    Ok(())
}
