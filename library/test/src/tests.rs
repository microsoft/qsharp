// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::run_stdlib_test;
use qsc::interpret::Value;

#[test]
fn check_expmodi_int() {
    run_stdlib_test("Microsoft.Quantum.Math.ExpModI(1,10,10)", &Value::Int(1));
    run_stdlib_test("Microsoft.Quantum.Math.ExpModI(10,0,10)", &Value::Int(1));
    run_stdlib_test("Microsoft.Quantum.Math.ExpModI(2,10,10)", &Value::Int(4));
}

#[test]
fn check_fst_snd() {
    run_stdlib_test("Fst(7,6)", &Value::Int(7));
    run_stdlib_test("Snd(7,6)", &Value::Int(6));
}

#[test]
fn check_index_range() {
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.IndexRange([7,6,5,4])::Start",
        &Value::Int(0),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.IndexRange([7,6,5,4])::Step",
        &Value::Int(1),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.IndexRange([7,6,5,4])::End",
        &Value::Int(3),
    );
}

#[test]
fn check_bitsize_i() {
    run_stdlib_test("Microsoft.Quantum.Math.BitSizeI(0)", &Value::Int(0));
    run_stdlib_test("Microsoft.Quantum.Math.BitSizeI(1)", &Value::Int(1));
    run_stdlib_test("Microsoft.Quantum.Math.BitSizeI(2)", &Value::Int(2));
    run_stdlib_test("Microsoft.Quantum.Math.BitSizeI(3)", &Value::Int(2));
    run_stdlib_test(
        "Microsoft.Quantum.Math.BitSizeI(0x7FFFFFFFFFFFFFFF)",
        &Value::Int(63),
    );
}
