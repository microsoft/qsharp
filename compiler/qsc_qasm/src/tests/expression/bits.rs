// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::{compile_qasm_to_qsharp, compile_qasm_to_qsharp_file};

use expect_test::expect;
use miette::Report;

#[test]
fn bit_array_bits_and_register_ops() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        bit[8] a = "10001111";
        bit[8] b = "01110000";
        output bit[8] ls_a_1;
        ls_a_1 = (a << 1); // Bit shift left produces "00011110"
        output bit[8] a_or_b;
        a_or_b = (a | b); // Bitwise OR produces "11111111"
        output bit[8] a_and_b;
        a_and_b = (a & b); // Bitwise AND produces "00000000"
        output bit[8] a_xor_b;
        a_xor_b = (a ^ b); // Bitwise XOR produces "11111111"
        //output bit[8] not_a;
        //not_a = ~a; // Bitwise NOT produces "01110000"
        output bit[8] rs_a_1;
        rs_a_1 = (a >> 1); // Bit shift right produces "01000111"
    "#;
    let qsharp = compile_qasm_to_qsharp_file(source)?;
    expect![[r#"
        namespace qasm_import {
            import Std.OpenQASM.Intrinsic.*;
            @EntryPoint()
            operation Test() : (Result[], Result[], Result[], Result[], Result[]) {
                mutable a = [One, Zero, Zero, Zero, One, One, One, One];
                mutable b = [Zero, One, One, One, Zero, Zero, Zero, Zero];
                mutable ls_a_1 = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
                set ls_a_1 = (Std.OpenQASM.Convert.IntAsResultArrayBE(Std.OpenQASM.Convert.ResultArrayAsIntBE(a) <<< 1, 8));
                mutable a_or_b = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
                set a_or_b = (Std.OpenQASM.Convert.IntAsResultArrayBE(Std.OpenQASM.Convert.ResultArrayAsIntBE(a) ||| Std.OpenQASM.Convert.ResultArrayAsIntBE(b), 8));
                mutable a_and_b = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
                set a_and_b = (Std.OpenQASM.Convert.IntAsResultArrayBE(Std.OpenQASM.Convert.ResultArrayAsIntBE(a) &&& Std.OpenQASM.Convert.ResultArrayAsIntBE(b), 8));
                mutable a_xor_b = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
                set a_xor_b = (Std.OpenQASM.Convert.IntAsResultArrayBE(Std.OpenQASM.Convert.ResultArrayAsIntBE(a) ^^^ Std.OpenQASM.Convert.ResultArrayAsIntBE(b), 8));
                mutable rs_a_1 = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
                set rs_a_1 = (Std.OpenQASM.Convert.IntAsResultArrayBE(Std.OpenQASM.Convert.ResultArrayAsIntBE(a) >>> 1, 8));
                (ls_a_1, a_or_b, a_and_b, a_xor_b, rs_a_1)
            }
        }"#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn bit_array_left_shift() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        bit[8] a = "10001111";
        output bit[8] ls_a_1;
        ls_a_1 = (a << 1); // Bit shift left produces "00011110"
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import Std.OpenQASM.Intrinsic.*;
        mutable a = [One, Zero, Zero, Zero, One, One, One, One];
        mutable ls_a_1 = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
        set ls_a_1 = (Std.OpenQASM.Convert.IntAsResultArrayBE(Std.OpenQASM.Convert.ResultArrayAsIntBE(a) <<< 1, 8));
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}
