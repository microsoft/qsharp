// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;
use miette::Report;

use crate::tests::compile_qasm_to_qsharp;

#[test]
#[ignore = "OPENQASM 3.0 parser bug"]
fn bitwise_not_int() -> miette::Result<(), Vec<Report>> {
    let source = "
        int x = 5;
        int y = ~x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        mutable x = 5;
        mutable y = ~~~x;
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn not_bool() -> miette::Result<(), Vec<Report>> {
    let source = "
        bool x = true;
        bool y = !x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        mutable x = true;
        mutable y = not x;
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn not_result() -> miette::Result<(), Vec<Report>> {
    let source = "
        bit x = 1;
        bit y = !x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        mutable x = One;
        mutable y = __BoolAsResult__(not __ResultAsBool__(x));
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn logical_not_int() -> miette::Result<(), Vec<Report>> {
    let source = "
        int x = 159;
        bool y = !x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        mutable x = 159;
        mutable y = not if x == 0 {
            false
        } else {
            true
        };
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "negating a Result type is an invalid Q# operation"]
fn bitwise_not_result() -> miette::Result<(), Vec<Report>> {
    let source = "
        bit[1] x;
        bool success = ~x[0];
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn logical_not_indexed_bit_array_in_if_cond() -> miette::Result<(), Vec<Report>> {
    let source = "
        bit[10] Classical;
        if (!Classical[1]) {
            Classical[0] = 1;
        }
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        mutable Classical = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
        if not __ResultAsBool__(Classical[1]) {
            set Classical w/= 0 <- One;
        };
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn neg_angle() -> miette::Result<(), Vec<Report>> {
    let source = "
        angle[4] x = 1.0;
        angle[4] y = -x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        mutable x = new __Angle__ {
            Value = 3,
            Size = 4
        };
        mutable y = __NegAngle__(x);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn notb_angle() -> miette::Result<(), Vec<Report>> {
    let source = "
        angle[4] x = 1.0;
        angle[4] y = ~x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        mutable x = new __Angle__ {
            Value = 3,
            Size = 4
        };
        mutable y = __AngleNotB__(x);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}
