// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::compile_qasm_stmt_to_qsharp;
use expect_test::expect;
use miette::Report;

#[test]
fn single_qubit() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        gate my_h q {
            h q;
        }
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![[r#"
        operation my_h(q : Qubit) : Unit is Adj + Ctl {
            h(q);
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn two_qubits() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        gate my_h q, q2 {
            h q2;
            h q;
        }
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![[r#"
        operation my_h(q : Qubit, q2 : Qubit) : Unit is Adj + Ctl {
            h(q2);
            h(q);
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn single_angle_single_qubit() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        gate my_h(θ) q {
            rx(θ) q;
        }
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![[r#"
        operation my_h(θ : __Angle__, q : Qubit) : Unit is Adj + Ctl {
            rx(θ, q);
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn two_angles_two_qubits() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        gate my_h(θ, φ) q, q2 {
            rx(θ) q2;
            ry(φ) q;
        }
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![[r#"
        operation my_h(θ : __Angle__, φ : __Angle__, q : Qubit, q2 : Qubit) : Unit is Adj + Ctl {
            rx(θ, q2);
            ry(φ, q);
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn capturing_external_variables_const_evaluate_them() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 2;
        const int b = 3;
        const int c = a * b;
        gate my_gate q {
            int x = c;
        }
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![[r#"
        operation my_gate(q : Qubit) : Unit is Adj + Ctl {
            mutable x = 6;
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn capturing_non_const_external_variable_fails() {
    let source = r#"
        int a = 2 << (-3);
        gate my_gate q {
            int x = a;
        }
    "#;

    let Err(errors) = compile_qasm_stmt_to_qsharp(source) else {
        panic!("Expected error");
    };

    expect![[r#"
        [Qsc.Qasm3.Lowerer.UndefinedSymbol

          x undefined symbol: a
           ,-[Test.qasm:4:21]
         3 |         gate my_gate q {
         4 |             int x = a;
           :                     ^
         5 |         }
           `----
        , Qsc.Qasm3.Lowerer.CannotCast

          x cannot cast expression of type Err to type Int(None, false)
           ,-[Test.qasm:4:21]
         3 |         gate my_gate q {
         4 |             int x = a;
           :                     ^
         5 |         }
           `----
        ]"#]]
    .assert_eq(&format!("{errors:?}"));
}

#[test]
fn capturing_non_const_evaluatable_external_variable_fails() {
    let source = r#"
        const int a = 2 << (-3);
        gate my_gate q {
            int x = a;
        }
    "#;

    let Err(errors) = compile_qasm_stmt_to_qsharp(source) else {
        panic!("Expected error");
    };

    expect![[r#"
        [Qsc.Qasm3.Compile.NegativeUIntValue

          x uint expression must evaluate to a non-negative value, but it evaluated
          | to -3
           ,-[Test.qasm:2:28]
         1 | 
         2 |         const int a = 2 << (-3);
           :                            ^^^^
         3 |         gate my_gate q {
           `----
        , Qsc.Qasm3.Lowerer.ExprMustBeConst

          x a captured variable must be a const expression
           ,-[Test.qasm:4:21]
         3 |         gate my_gate q {
         4 |             int x = a;
           :                     ^
         5 |         }
           `----
        ]"#]]
    .assert_eq(&format!("{errors:?}"));
}

#[test]
fn fuzz_2297() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        gate g q0 {
            gate q0 q1 {}
            q1;
        }
    "#;

    let config = crate::CompilerConfig::new(
        crate::QubitSemantics::Qiskit,
        crate::OutputSemantics::Qiskit,
        crate::ProgramType::Fragments,
        None,
        None,
    );
    let mut resolver = crate::io::InMemorySourceResolver::from_iter([]);
    let unit = crate::compile_to_qsharp_ast_with_config(
        source,
        "source.qasm",
        Some(&mut resolver),
        config,
    );
    if unit.has_errors() {
        for e in unit.errors.into_iter().map(Report::new) {
            println!("{e}");
        }
    }
    let Some(package) = unit.package else {
        panic!("Expected package, got None");
    };
    let qsharp = crate::tests::get_last_statement_as_qsharp(&package);
    expect![[r#"
        operation g(q0 : Qubit) : Unit is Adj + Ctl {
            operation q0(q1 : Qubit) : Unit is Adj + Ctl {}
            q1;
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}
