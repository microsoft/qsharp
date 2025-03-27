// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::compile_qasm_to_qsharp;
use expect_test::expect;
use miette::Report;

#[test]
fn funcall_with_no_arguments_generates_correct_qsharp() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        def empty() {}
        empty();
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let empty : () -> Unit = () -> {};
        empty();
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn funcall_with_one_argument_generates_correct_qsharp() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        def square(int x) -> int {
            return x * x;
        }

        square(2);
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let square : (Int) -> Int = (x) -> {
            return x * x;
        };
        square(2);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn funcall_with_two_arguments_generates_correct_qsharp() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        def sum(int x, int y) -> int {
            return x + y;
        }

        sum(2, 3);
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let sum : (Int, Int) -> Int = (x, y) -> {
            return x + y;
        };
        sum(2, 3);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn funcall_with_qubit_argument() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        def parity(qubit[2] qs) -> bit {
            bit a = measure qs[0];
            bit b = measure qs[1];
            return a ^ b;
        }

        bit p = parity(2);
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        function __ResultAsInt__(input : Result) : Int {
            if Microsoft.Quantum.Convert.ResultAsBool(input) {
                1
            } else {
                0
            }
        }
        let parity : (Qubit[]) => Result = (qs) => {
            mutable a = QIR.Intrinsic.__quantum__qis__m__body(qs[0]);
            mutable b = QIR.Intrinsic.__quantum__qis__m__body(qs[1]);
            return if __ResultAsInt__(a) ^^^ __ResultAsInt__(b) == 0 {
                One
            } else {
                Zero
            };
        };
        mutable p = parity(2);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn funcall_with_too_few_arguments_generates_error() {
    let source = r#"
        def square(int x) -> int {
            return x * x;
        }

        square();
    "#;

    let Err(errors) = compile_qasm_to_qsharp(source) else {
        panic!("Expected error");
    };

    expect![[r#"
        [Qsc.Qasm3.Compile.InvalidNumberOfClassicalArgs

          x Gate expects 1 classical arguments, but 0 were provided.
           ,-[Test.qasm:6:9]
         5 | 
         6 |         square();
           :         ^^^^^^^^
         7 |     
           `----
        ]"#]]
    .assert_eq(&format!("{errors:?}"));
}

#[test]
fn funcall_with_too_many_arguments_generates_error() {
    let source = r#"
        def square(int x) -> int {
            return x * x;
        }

        square(2, 3);
    "#;

    let Err(errors) = compile_qasm_to_qsharp(source) else {
        panic!("Expected error");
    };

    expect![[r#"
        [Qsc.Qasm3.Compile.InvalidNumberOfClassicalArgs

          x Gate expects 1 classical arguments, but 2 were provided.
           ,-[Test.qasm:6:9]
         5 | 
         6 |         square(2, 3);
           :         ^^^^^^^^^^^^
         7 |     
           `----
        ]"#]]
    .assert_eq(&format!("{errors:?}"));
}

#[test]
fn funcall_accepts_qubit_argument() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        def h_wrapper(qubit q) {
            h q;
        }

        qubit q;
        h_wrapper(q);
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let h_wrapper : (Qubit) => Unit = (q) => {
            H(q);
        };
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        h_wrapper(q);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn classical_decl_initialized_with_funcall() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        def square(int x) -> int {
            return x * x;
        }

        int a = square(2);
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let square : (Int) -> Int = (x) -> {
            return x * x;
        };
        mutable a = square(2);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn classical_decl_initialized_with_incompatible_funcall_errors() {
    let source = r#"
        def square(float x) -> float {
            return x * x;
        }

        bit a = square(2.0);
    "#;

    let Err(errors) = compile_qasm_to_qsharp(source) else {
        panic!("Expected error");
    };

    expect![[r#"
        [Qsc.Qasm3.Compile.CannotCast

          x Cannot cast expression of type Float(None, false) to type Bit(false)
           ,-[Test.qasm:6:17]
         5 | 
         6 |         bit a = square(2.0);
           :                 ^^^^^^^^^^^
         7 |     
           `----
        ]"#]]
    .assert_eq(&format!("{errors:?}"));
}
