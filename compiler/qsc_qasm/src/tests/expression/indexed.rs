// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::{check_qasm_to_qsharp, compile_qasm_to_qsharp};

use expect_test::expect;
use miette::Report;

#[test]
fn indexed_bit_can_be_implicitly_converted_to_float() {
    let source = "
        bit[5] x;
        if (x[0] == 1.) {
        }
    ";
    check_qasm_to_qsharp(
        source,
        &expect![[r#"
        import Std.OpenQASM.Intrinsic.*;
        mutable x = [Zero, Zero, Zero, Zero, Zero];
        if Std.OpenQASM.Convert.ResultAsDouble(x[0]) == 1. {};
    "#]],
    );
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
        import Std.OpenQASM.Intrinsic.*;
        mutable x = [Zero, Zero, Zero, Zero, Zero];
        if Std.OpenQASM.Convert.ResultAsInt(x[0]) == 1 {
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
        import Std.OpenQASM.Intrinsic.*;
        mutable x = [Zero, Zero, Zero, Zero, Zero];
        if Std.OpenQASM.Convert.ResultAsBool(x[0]) {
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
        import Std.OpenQASM.Intrinsic.*;
        mutable x = [Zero, Zero, Zero, Zero, Zero];
        mutable y = x[0];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn bool_indexed_ty_is_same_as_element_ty() -> miette::Result<(), Vec<Report>> {
    let source = "
        array[bool, 5] x;
        bool y = x[0];
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import Std.OpenQASM.Intrinsic.*;
        mutable x = [false, false, false, false, false];
        mutable y = x[0];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn bitstring_slicing() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        bit[5] ans = "10101";
        qubit qq;
        if(ans[0:3] == 4) x qq;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import Std.OpenQASM.Intrinsic.*;
        mutable ans = [One, Zero, One, Zero, One];
        let qq = QIR.Runtime.__quantum__rt__qubit_allocate();
        if Std.OpenQASM.Convert.ResultArrayAsIntBE(ans[0..3]) == 4 {
            x(qq);
        };
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn bitstring_slicing_with_step() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        bit[5] ans = "10101";
        qubit qq;
        if(ans[0:3:2] == 4) x qq;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import Std.OpenQASM.Intrinsic.*;
        mutable ans = [One, Zero, One, Zero, One];
        let qq = QIR.Runtime.__quantum__rt__qubit_allocate();
        if Std.OpenQASM.Convert.ResultArrayAsIntBE(ans[0..3..2]) == 4 {
            x(qq);
        };
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn index_set_in_non_alias_stmt_fails() {
    let source = r#"
        include "stdgates.inc";
        bit[5] ans = "10101";
        qubit qq;
        if(ans[{1, 3}] == 4) x qq;
    "#;

    let Err(errors) = compile_qasm_to_qsharp(source) else {
        panic!("Expected error");
    };

    expect![[r#"
        [Qasm.Lowerer.IndexSetOnlyAllowedInAliasStmt

          x index sets are only allowed in alias statements
           ,-[Test.qasm:5:16]
         4 |         qubit qq;
         5 |         if(ans[{1, 3}] == 4) x qq;
           :                ^^^^^^
         6 |     
           `----
        , Qasm.Lowerer.CannotCast

          x cannot cast expression of type Err to type Float(None, true)
           ,-[Test.qasm:5:12]
         4 |         qubit qq;
         5 |         if(ans[{1, 3}] == 4) x qq;
           :            ^^^^^^^^^^^
         6 |     
           `----
        ]"#]]
    .assert_eq(&format!("{errors:?}"));
}
