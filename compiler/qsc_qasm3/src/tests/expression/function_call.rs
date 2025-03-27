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

        qubit[2] qs;
        bit p = parity(qs);
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
        let qs = QIR.Runtime.AllocateQubitArray(2);
        mutable p = parity(qs);
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

#[test]
fn funcall_implicit_arg_cast_uint_to_bitarray() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        def parity(bit[2] arr) -> bit {
            return 1;
        }

        bit p = parity(2);
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        function __BoolAsResult__(input : Bool) : Result {
            Microsoft.Quantum.Convert.BoolAsResult(input)
        }
        function __IntAsResultArrayBE__(number : Int, bits : Int) : Result[] {
            mutable runningValue = number;
            mutable result = [];
            for _ in 1..bits {
                set result += [__BoolAsResult__((runningValue &&& 1) != 0)];
                set runningValue >>>= 1;
            }
            Microsoft.Quantum.Arrays.Reversed(result)
        }
        let parity : (Result[]) -> Result = (arr) -> {
            return 1;
        };
        mutable p = parity(__IntAsResultArrayBE__(2, 2));
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn funcall_implicit_arg_cast_uint_to_qubit_errors() {
    let source = r#"
        def parity(qubit[2] arr) -> bit {
            return 1;
        }

        bit p = parity(2);
    "#;

    let Err(errors) = compile_qasm_to_qsharp(source) else {
        panic!("Expected error");
    };

    expect![[r#"
        [Qsc.Qasm3.Compile.CannotCast

          x Cannot cast expression of type Int(None, true) to type QubitArray(One(2))
           ,-[Test.qasm:6:24]
         5 | 
         6 |         bit p = parity(2);
           :                        ^
         7 |     
           `----
        ]"#]]
    .assert_eq(&format!("{errors:?}"));
}
