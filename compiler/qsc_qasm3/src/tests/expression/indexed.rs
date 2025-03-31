// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::{compile_qasm_stmt_to_qsharp, compile_qasm_to_qsharp};

use expect_test::expect;
use miette::Report;

#[test]
fn indexed_bit_cannot_be_implicitly_converted_to_float() {
    let source = "
        bit[5] x;
        if (x[0] == 1.) {
        }
    ";

    let Err(errors) = compile_qasm_stmt_to_qsharp(source) else {
        panic!("Expected an error");
    };

    assert_eq!(1, errors.len(), "Expected one error");
    expect!["Cannot cast expression of type Bit(false) to type Float(None, true)"]
        .assert_eq(&errors[0].to_string());
}

#[test]
fn indexed_bit_can_implicitly_convert_to_int() -> miette::Result<(), Vec<Report>> {
    let source = "
        bit[5] x;
        if (x[0] == 1) {
            x[1] = 1;
        }
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        mutable x = [Zero, Zero, Zero, Zero, Zero];
        if __ResultAsInt__(x[0]) == 1 {
            set x w/= 1 <- One;
        };
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn indexed_bit_can_implicitly_convert_to_bool() -> miette::Result<(), Vec<Report>> {
    let source = "
        bit[5] x;
        if (x[0]) {
            x[1] = 1;
        }
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        mutable x = [Zero, Zero, Zero, Zero, Zero];
        if __ResultAsBool__(x[0]) {
            set x w/= 1 <- One;
        };
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn bit_indexed_ty_is_same_as_element_ty() -> miette::Result<(), Vec<Report>> {
    let source = "
        bit[5] x;
        bit y = x[0];
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        mutable x = [Zero, Zero, Zero, Zero, Zero];
        mutable y = x[0];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "Not yet implemented"]
fn bool_indexed_ty_is_same_as_element_ty() -> miette::Result<(), Vec<Report>> {
    let source = "
        array[bool, 5] x;
        bool y = x[0];
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        mutable x = [false, false, false, false, false];
        mutable y = x[0];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "Not yet implemented"]
fn bitstring_slicing() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        bit[5] ans = "10101";
        qubit qq;
        if(ans[0:3] == 4) x qq;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#""#]].assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "Not yet implemented"]
fn bitstring_slicing_with_step() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        bit[5] ans = "10101";
        qubit qq;
        if(ans[0:3:2] == 4) x qq;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#""#]].assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "Not yet implemented"]
fn bitstring_index_set() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        bit[5] ans = "10101";
        qubit qq;
        if(ans[{1, 3}] == 4) x qq;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#""#]].assert_eq(&qsharp);
    Ok(())
}
