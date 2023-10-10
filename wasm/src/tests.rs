// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;
use indoc::indoc;

use super::run_internal;

#[test]
fn test_run_simple_program_multiple_shots() {
    let mut output = Vec::new();
    run_internal(
        indoc! {"
            namespace Test {
                @EntryPoint()
                operation Main() : Int { 4 }
            }"
        },
        "",
        |s| output.push(s.to_string()),
        3,
    )
    .expect("code should compile and run");
    expect![[r#"
        {"result":"4","success":true,"type":"Result"}
        {"result":"4","success":true,"type":"Result"}
        {"result":"4","success":true,"type":"Result"}"#]]
    .assert_eq(&output.join("\n"));
}

#[test]
fn test_run_error_program_multiple_shots() {
    let mut output = Vec::new();
    run_internal(
        indoc! {"
            namespace Test {
                @EntryPoint()
                operation Main() : Unit {
                    use q = Qubit();
                    CNOT(q, q)
                }
            }"
        },
        "",
        |s| output.push(s.to_string()),
        3,
    )
    .expect("code should compile and run");
    expect![[r#"
        {"result":{"code":{"target":"","value":"Qsc.Eval.QubitUniqueness"},"end_pos":89260,"message":"runtime error: qubits in gate invocation are not unique","severity":"error","start_pos":89243},"success":false,"type":"Result"}
        {"result":{"code":{"target":"","value":"Qsc.Eval.QubitUniqueness"},"end_pos":89260,"message":"runtime error: qubits in gate invocation are not unique","severity":"error","start_pos":89243},"success":false,"type":"Result"}
        {"result":{"code":{"target":"","value":"Qsc.Eval.QubitUniqueness"},"end_pos":89260,"message":"runtime error: qubits in gate invocation are not unique","severity":"error","start_pos":89243},"success":false,"type":"Result"}"#]]
    .assert_eq(&output.join("\n"));
}
