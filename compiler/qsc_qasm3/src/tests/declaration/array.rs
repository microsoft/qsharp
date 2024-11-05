// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod bit;
mod qubit;

use crate::tests::{fail_on_compilation_errors, parse, qasm_to_program_fragments};
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

    let res = parse(source)?;
    assert!(!res.has_errors());
    let unit = qasm_to_program_fragments(res.source, res.source_map);
    fail_on_compilation_errors(&unit);
    Ok(())
}
