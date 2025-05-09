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
    super::compare_qasm_and_qasharp_asts(source);
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
    super::compare_qasm_and_qasharp_asts(source);
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
    super::compare_qasm_and_qasharp_asts(source);
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
    super::compare_qasm_and_qasharp_asts(source);
    compile_qasm_best_effort(source, Profile::Unrestricted);
}

#[test]
fn fuzz_2298() {
    let source = r#"gate y()a{gate a,b{}b"#;
    super::compare_qasm_and_qasharp_asts(source);
    compile_qasm_best_effort(source, Profile::Unrestricted);
}

#[test]
fn fuzz_2313() {
    let source = r#"ctrl(π/0s)@a"#;
    super::compare_qasm_and_qasharp_asts(source);
    compile_qasm_best_effort(source, Profile::Unrestricted);
}

#[test]
fn fuzz_2332() {
    let source = r#"ctrl(0/0)@s"#;
    super::compare_qasm_and_qasharp_asts(source);
    compile_qasm_best_effort(source, Profile::Unrestricted);
}

#[test]
fn fuzz_2348() {
    let source = r#"ctrl(0%0)@s"#;
    super::compare_qasm_and_qasharp_asts(source);
    compile_qasm_best_effort(source, Profile::Unrestricted);
}

#[test]
fn fuzz_2369() {
    let source = r#"// Ope0 standard gate) mibrary
//
// Notely the set that the intarlenly.  See the
// `source/language/standard_lurcc; }

// four parameter contrloled-U gat pow(0.5) @ s a; }

// sqrt(NOT) gate
gate sx a { pow(0.5)verse of sqrt(S)
gaoooooooote tg  i {adnv @ pow(0.` docume5) @ πtdgpow(0.` documeni )5 tanverx a { po0.5)verse of sqrt(S)
gaoooooooote tg  i {adnv @ pow(0.` docume5) @ pow(0.` docume5) nta inverse of sqrt(S)
g,aoooooootoe i {adnv @WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW _lurcc; }

// four parameter contrloled-U gat pow(0.5) @ s a; }

// sqrt(NOT) gate
gate sx a { pow(0.5)verse off sqrt(S)
gaoooooooote tg  i {adnv @ pow(0.` docume5) @ pow(0.` docume5) nta inverse of sqrt(S)
g,aoooooootoe i {adnv @WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW _lurcc; }

// four parameter contrloled-U gat pow(0.5) @ s a; }

// sqrt(NOT) gate
gate sx a { pow(0.5)verse of sqrt(S)
gaoooooooote tg  i {adnv @ pow(0.` docume5) @ pow(0.` docume5) nta inverse of sqrt(S)
gaoooooootoe i {adnv @ pow(0.` docume5) @ pow(0.` docume5) ntaoooooote tg  i {adnv @ pow(0.` docume5) @ pow(0.` docume5) nta inverse of sqrt(S)
gaoogaoooooooote tg  i {adnv @ pow(0.` docume5) @ pow(0.` docume5) nta inverse of sqrt(S)
gaoooooootoe i {adnv @ pow(0.` docume5) @ pow(0.` docume5) ntaoooooote tg  i {adnv @ pow(0.` docume5) @ pow(0.` docume5) nta inverse.` docume5) nta inverse of sqrt(S)
gaoooooootoe i {adnv @ pow(0.` docume5) @ pow(0.` docume5) ntaoooooote tg  i {adnv @ pow(0.` docume5) @ pow(0.` docume5) nta inverse of sqrt(S)
gaoogaoooooooote tg  i {adnv @ pow(0.` docume5) @ pow(0.` docume5) nta inverse of sqrt(S)
gaoooooootoe i {adnv @ pow(0.` docume5) @ pow(0.` docume5) ntaoooooote tg  i {adnv @ pow(0.` w(0.` docume5) @ pow(0.` docume5) nta inverse of sqrt(S)
gaoooooootoe i {adnv @ pow(0.` docume5) @ pow(0.` docume5) ntaoooooote tg  i {adnv @ pow(0.` docume5) @ pow(0.` docume5) nta inverse of sqrt(S)
gaoooooootoe i {adnv @ pow(0.` docume5) @ pow(0.` docume5) nta inverse of sqrt(5) @ pow(0.` docume5) nta inverse of sqrt(S)
gaoooooootoe i {adnv @ pow(0.` docume5) @ pow(0.` docume5) nta inver5) @ pow(0.` docume5) nta inverse of sqrt(5) @ pow(0.` docume5) nta inverse of sqrt(S)
gaoooooootoe i {adnv @ pow(0.` docume5) @ pow(0.` docume5) nta inverse of sqrt(S)
gaoooooootoe tg  i {adnv @ pow(0.` docume)5@  pow(0.` docume5) ntation for full @ staildnv @ pow(0.` docume)5@  pow(0.` docume5) ntation for full @ stail."#;
    super::compare_qasm_and_qasharp_asts(source);
    compile_qasm_best_effort(source, Profile::Unrestricted);
}
