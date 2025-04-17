// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::compile_qasm_stmt_to_qsharp;
use expect_test::expect;
use miette::Report;

#[test]
fn no_parameters_no_return() -> miette::Result<(), Vec<Report>> {
    let source = "def empty() {}";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![[r#"
        function empty() : Unit {}
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn single_parameter() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        def square(int x) -> int {
            return x * x;
        }
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![[r#"
        function square(x : Int) : Int {
            return x * x;
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn qubit_parameter() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        def square(qubit q) -> uint {
            return 1;
        }
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![[r#"
        operation square(q : Qubit) : Int {
            return 1;
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn qubit_array_parameter() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        def square(qubit[3] qs) -> uint {
            return 1;
        }
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![[r#"
        operation square(qs : Qubit[]) : Int {
            return 1;
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn implicit_cast_to_function_return_type() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        def square(int a) -> bit {
            return a;
        }
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![[r#"
        function square(a : Int) : Result {
            return if a == 0 {
                One
            } else {
                Zero
            };
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn return_from_void_function() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        def square(int a) {
            return;
        }
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![[r#"
        function square(a : Int) : Unit {
            return ();
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn return_expr_on_void_function_fails() {
    let source = r#"
        def square(int val) {
            return val;
        }
    "#;

    let Err(errors) = compile_qasm_stmt_to_qsharp(source) else {
        panic!("Expected error");
    };

    expect![[r#"
        [Qasm.Lowerer.ReturningExpressionFromVoidSubroutine

          x cannot return an expression from a void subroutine
           ,-[Test.qasm:3:20]
         2 |         def square(int val) {
         3 |             return val;
           :                    ^^^
         4 |         }
           `----
        ]"#]]
    .assert_eq(&format!("{errors:?}"));
}

#[test]
fn missing_return_expr_on_non_void_function_fails() {
    let source = r#"
        def square(int a) -> bit {
            return;
        }
    "#;

    let Err(errors) = compile_qasm_stmt_to_qsharp(source) else {
        panic!("Expected error");
    };

    expect![[r#"
        [Qasm.Lowerer.MissingTargetExpressionInReturnStmt

          x return statements on a non-void subroutine should have a target expression
           ,-[Test.qasm:3:13]
         2 |         def square(int a) -> bit {
         3 |             return;
           :             ^^^^^^^
         4 |         }
           `----
        ]"#]]
    .assert_eq(&format!("{errors:?}"));
}

#[test]
fn capturing_external_variables_const_evaluate_them() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 2;
        const int b = 3;
        const int c = a * b;
        def f() -> int {
            return c;
        }
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![[r#"
        function f() : Int {
            return 6;
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn capturing_non_const_external_variable_fails() {
    let source = r#"
        int a = 2 << (-3);
        def f() -> int {
            return a;
        }
    "#;

    let Err(errors) = compile_qasm_stmt_to_qsharp(source) else {
        panic!("Expected error");
    };

    expect![[r#"
        [Qasm.Lowerer.UndefinedSymbol

          x undefined symbol: a
           ,-[Test.qasm:4:20]
         3 |         def f() -> int {
         4 |             return a;
           :                    ^
         5 |         }
           `----
        , Qasm.Lowerer.CannotCast

          x cannot cast expression of type Err to type Int(None, false)
           ,-[Test.qasm:4:20]
         3 |         def f() -> int {
         4 |             return a;
           :                    ^
         5 |         }
           `----
        ]"#]]
    .assert_eq(&format!("{errors:?}"));
}

#[test]
fn capturing_non_const_evaluatable_external_variable_fails() {
    let source = r#"
        const int a = 2 << (-3);
        def f() -> int {
            return a;
        }
    "#;

    let Err(errors) = compile_qasm_stmt_to_qsharp(source) else {
        panic!("Expected error");
    };

    expect![[r#"
        [Qasm.Compiler.NegativeUIntValue

          x uint expression must evaluate to a non-negative value, but it evaluated
          | to -3
           ,-[Test.qasm:2:28]
         1 | 
         2 |         const int a = 2 << (-3);
           :                            ^^^^
         3 |         def f() -> int {
           `----
        , Qasm.Lowerer.ExprMustBeConst

          x a captured variable must be a const expression
           ,-[Test.qasm:4:20]
         3 |         def f() -> int {
         4 |             return a;
           :                    ^
         5 |         }
           `----
        ]"#]]
    .assert_eq(&format!("{errors:?}"));
}
