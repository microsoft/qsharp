// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::compile_qasm_best_effort;
use qsc::target::Profile;

/// We also had an issue where
///   1. naming a gate the same as a qubit parameter of a parent gate,
///   2. and then referencing the qubit parameter of the inner gate.
///
/// was causing a panic in the Q# resolver.
#[test]
fn fuzz_2297_referencing_qubit_parameter() {
    let source = r#"
    gate g q0 {
        gate q0 q1 {}
        q1;
    }
    "#;
    compile_qasm_best_effort(source, Profile::Unrestricted);
}

/// The same panic happened when referencing an angle parameter.
#[test]
fn fuzz_2297_referencing_angle_parameter() {
    let source = r#"
    gate g q0 {
        gate q0(r) q1 {}
        r;
    }
    "#;
    compile_qasm_best_effort(source, Profile::Unrestricted);
}

/// Subroutines didn't have this problem, even though they are also
/// compiled to operations when they take qubit arguments.
#[test]
fn fuzz_2297_def() {
    let source = r#"
    def g(qubit q0) {
        def q0(qubit q1) {}
        q1;
    }
    "#;
    compile_qasm_best_effort(source, Profile::Unrestricted);
}

/// We also had an issue where, in the same conditions as `fuzz_2297`,
/// a missing identifier in a comma separated list of formal paremeters
/// would generate an empty string Identifier and forward it to Q#,
/// which yields an invalid Q# AST.
#[test]
fn fuzz_2297_with_trailing_comma() {
    let source = r#"
        gate g q0 {
            gate q0 ,q1 {}
            q1;
        }
    "#;
    compile_qasm_best_effort(source, Profile::Unrestricted);
}

#[test]
fn fuzz_2298() {
    let source = r#"gate y()a{gate a,b{}b"#;
    compile_qasm_best_effort(source, Profile::Unrestricted);
}

#[test]
fn fuzz_2313() {
    let source = r#"ctrl(π/0s)@a"#;
    compile_qasm_best_effort(source, Profile::Unrestricted);
}

#[test]
fn fuzz_2332() {
    let source = r#"ctrl(0/0)@s"#;
    compile_qasm_best_effort(source, Profile::Unrestricted);
}

#[test]
fn fuzz_2348() {
    let source = r#"ctrl(0%0)@s"#;
    compile_qasm_best_effort(source, Profile::Unrestricted);
}

#[test]
fn fuzz_2366() {
    let source = "t[:π";
    compile_qasm_best_effort(source, Profile::Unrestricted);
}

#[test]
fn fuzz_2368() {
    let source = "c[:0s";
    compile_qasm_best_effort(source, Profile::Unrestricted);
}

#[test]
fn fuzz_2379() {
    let source = "1[true:";
    compile_qasm_best_effort(source, Profile::Unrestricted);
}

#[test]
fn fuzz_2391() {
    let source = "c[:0s";
    compile_qasm_best_effort(source, Profile::Unrestricted);
}

#[test]
fn fuzz_2392() {
    let source = "e[π:";
    compile_qasm_best_effort(source, Profile::Unrestricted);
}
