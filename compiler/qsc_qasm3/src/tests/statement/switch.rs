// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::{compile_qasm_to_qsharp, compile_qasm_to_qsharp_file};
use expect_test::expect;
use miette::Report;

#[test]
fn default_is_optional() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        OPENQASM 3.0;
        int i = 15;
        switch (i) {
            case 1 {
                i = 2;
            }
        }
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable i = 15;
        if i == 1 {
            set i = 2;
        };
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn default_as_only_case_causes_parse_error() {
    let source = r#"
        OPENQASM 3.0;
        int i = 15;
        switch (i) {
            default {
                i = 2;
            }
        }
    "#;

    let res = compile_qasm_to_qsharp(source);
    let Err(errors) = res else {
        panic!("Expected an error, got {res:?}");
    };
    assert_eq!(errors.len(), 1);
    expect![r#"QASM3 Parse Error: expecting `case` keyword"#].assert_eq(&errors[0].to_string());
}

#[test]
fn no_cases_causes_parse_error() {
    let source = r#"
        OPENQASM 3.0;
        int i = 15;
        switch (i) {
        }
    "#;

    let res = compile_qasm_to_qsharp(source);
    let Err(errors) = res else {
        panic!("Expected an error, got {res:?}");
    };
    assert_eq!(errors.len(), 1);
    expect![r#"QASM3 Parse Error: expecting `case` keyword"#].assert_eq(&errors[0].to_string());
}

#[test]
fn spec_case_1() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        OPENQASM 3.0;
        include "stdgates.inc";
        qubit q;

        int i = 15;

        switch (i) {
        case 1, 3, 5 {
            h q;
        }
        case 2, 4, 6 {
            x q;
        }
        case -1 {
            y q;
        }
        default {
            z q;
        }
        }
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        mutable i = 15;
        if i == 1 or i == 3 or i == 5 {
            H(q);
        } elif i == 2 or i == 4 or i == 6 {
            X(q);
        } elif i == -1 {
            Y(q);
        } else {
            Z(q);
        };
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn spec_case_2() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        OPENQASM 3.0;
        include "stdgates.inc";
        qubit q;

        const int A = 0;
        const int B = 1;
        int i = 15;

        switch (i) {
        case A {
            h q;
        }
        case B {
            x q;
        }
        case B+1 {
            y q;
        }
        default {
            z q;
        }
        }
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        let A = 0;
        let B = 1;
        mutable i = 15;
        if i == A {
            H(q);
        } elif i == B {
            X(q);
        } elif i == B + 1 {
            Y(q);
        } else {
            Z(q);
        };
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn spec_case_3() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        OPENQASM 3.0;
        include "stdgates.inc";
        qubit q;
        bit[2] b;
        // int(b) orginally, but we don't support that yet
        switch (b) {
        case 0b00 {
            h q;
        }
        case 0b01 {
            x q;
        }
        case 0b10 {
            y q;
        }
        case 0b11 {
            z q;
        }
        }
    "#;

    let qsharp = compile_qasm_to_qsharp_file(source)?;
    expect![
        r#"
        namespace qasm3_import {
            @EntryPoint()
            operation Test() : Result[] {
                function __ResultArrayAsIntBE__(results : Result[]) : Int {
                    Microsoft.Quantum.Convert.ResultArrayAsInt(Microsoft.Quantum.Arrays.Reversed(results))
                }
                let q = QIR.Runtime.__quantum__rt__qubit_allocate();
                mutable b = [Zero, Zero];
                if __ResultArrayAsIntBE__(b) == 0 {
                    H(q);
                } elif __ResultArrayAsIntBE__(b) == 1 {
                    X(q);
                } elif __ResultArrayAsIntBE__(b) == 2 {
                    Y(q);
                } elif __ResultArrayAsIntBE__(b) == 3 {
                    Z(q);
                };
                b
            }
        }"#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "Function decls are not supported yet"]
fn spec_case_4() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        OPENQASM 3.0;
        include "stdgates.inc";
        qubit q;
        bit[2] b;
        def foo(int i, qubit[8] d) -> bit {
            return measure d[i];
        }

        int i = 15;

        int j = 1;
        int k = 2;

        bit c1;

        qubit[8] q0;

        switch (i) {
        case 1 {
            j = k + foo(k, q0);
        }
        case 2 {
            float[64] d = j / k;
        }
        case 3 {
        }
        default {
        }
        }
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn spec_case_5() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        OPENQASM 3.0;
        include "stdgates.inc";


        qubit[8] q;

        int j = 30;
        int i;

        switch (i) {
        case 1, 2, 5, 12 {
        }
        case 3 {
        switch (j) {
        case 10, 15, 20 {
            h q;
        }
        }
        }
        }
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.AllocateQubitArray(8);
        mutable j = 30;
        mutable i = 0;
        if i == 1 or i == 2 or i == 5 or i == 12 {} elif i == 3 {
            if j == 10 or j == 15 or j == 20 {
                H(q);
            };
        };
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}
