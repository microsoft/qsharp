// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(
    clippy::needless_raw_string_hashes,
    clippy::similar_names,
    clippy::too_many_lines
)]

pub mod test_utils;

use indoc::indoc;
use test_utils::get_rir_program;

#[test]
fn assign_result_register_updates_value() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Result {
                use q = Qubit();
                mutable r = Zero;
                set r = MResetZ(q);
                r
            }
        }
    "#});
    println!("{program}");
}
