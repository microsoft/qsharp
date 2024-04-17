// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

pub mod test_utils;

use indoc::indoc;
use test_utils::compile_and_partially_evaluate;

#[ignore = "WIP"]
#[test]
fn call_to_single_qubit_unitary_with_two_calls_to_the_same_intrinsic() {
    let program = compile_and_partially_evaluate(indoc! {r#"
        namespace Test {
            operation Op(q : Qubit) : Unit { body intrinsic; }
            operation OpSquared(q : Qubit) : Unit {
                Op(q);
                Op(q);
            }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                OpSquared(q);
            }
        }
    "#});
    println!("{program}");
}

#[ignore = "WIP"]
#[test]
fn call_to_single_qubit_unitary_with_calls_to_different_intrinsics() {
    let program = compile_and_partially_evaluate(indoc! {r#"
        namespace Test {
            operation OpA(q : Qubit) : Unit { body intrinsic; }
            operation OpB(q : Qubit) : Unit { body intrinsic; }
            operation Combined(q : Qubit) : Unit {
                OpA(q);
                OpB(q);
            }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                Combined(q);
            }
        }
    "#});
    println!("{program}");
}

#[ignore = "WIP"]
#[test]
fn call_to_two_qubit_unitary() {
    let program = compile_and_partially_evaluate(indoc! {r#"
        namespace Test {
            operation Op(q0 : Qubit, q1 : Qubit) : Unit { body intrinsic; }
            operation ApplyOpCombinations(q0 : Qubit, q1 : Qubit) : Unit {
                Op(q0, q1);
                Op(q1, q0);
            }
            @EntryPoint()
            operation Main() : Unit {
                use (q0, q1) = (Qubit(), Qubit());
                ApplyOpCombinations(q0, q1);
            }
        }
    "#});
    println!("{program}");
}

#[ignore = "WIP"]
#[test]
fn call_to_unitary_that_receives_double_and_qubit() {
    let program = compile_and_partially_evaluate(indoc! {r#"
        namespace Test {
            operation DoubleFirst(d : Double, q : Qubit) : Unit { body intrinsic; }
            operation QubitFirst(q : Qubit, d : Double) : Unit { body intrinsic; }
            operation Op(d : Double, q : Qubit) : Unit {
                DoubleFirst(d, q);
                QubitFirst(q, d);
            }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                Op(1.0, q);
            }
        }
    "#});
    println!("{program}");
}

#[ignore = "WIP"]
#[test]
fn call_to_unitary_rotation_unitary_with_computation() {
    let program = compile_and_partially_evaluate(indoc! {r#"
        namespace Test {
            operation Rotation(d : Double, q : Qubit) : Unit { body intrinsic; }
            operation RotationWithComputation(d : Double, q : Qubit) : Unit {
                Rotation(2.0 * d, q);
            }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                RotationWithComputation(2.0, q);
                RotationWithComputation(3.0, q);
            }
        }
    "#});
    println!("{program}");
}
