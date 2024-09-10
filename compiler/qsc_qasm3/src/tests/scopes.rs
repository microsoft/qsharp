// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use crate::tests::compile_qasm_to_qsharp;
use expect_test::expect;
use miette::Report;

#[test]
fn can_access_const_decls_from_global_scope() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        const int i = 7;
        gate my_h q {
            if (i == 0) {
                h q;
            }
        }
        qubit q;
        my_h q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let i = 7;
        let my_h : (Qubit) => Unit = (q) => {
            if i == 0 {
                H(q);
            };
        };
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        my_h(q);
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn cannot_access_mutable_decls_from_global_scope() {
    let source = r#"
        include "stdgates.inc";
        int i;
        gate my_h q {
            if (i == 0) {
                h q;
            }
        }
    "#;

    let Err(errors) = compile_qasm_to_qsharp(source) else {
        panic!("Expected an error");
    };
    expect![r#"Undefined symbol: i."#].assert_eq(&errors[0].to_string());
}
