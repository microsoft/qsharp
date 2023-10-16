// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;
use indoc::indoc;

use super::run_internal;

use crate::get_qir;

#[test]
fn test_missing_type() {
    let code = "namespace input { operation Foo(a) : Unit {} }";
    let (_, mut diag) = crate::compile(code);
    assert_eq!(diag.len(), 2, "{diag:#?}");
    let err_1 = diag.pop().unwrap();
    let err_2 = diag.pop().unwrap();

    assert_eq!(err_1.start_pos, 32);
    assert_eq!(err_1.end_pos, 33);
    assert_eq!(err_1.message, "type error: insufficient type information to infer type\n\nhelp: provide a type annotation");
    assert_eq!(err_2.start_pos, 32);
    assert_eq!(err_2.end_pos, 33);
    assert_eq!(err_2.message, "type error: missing type in item signature\n\nhelp: types cannot be inferred for global declarations");
}

#[test]
fn test_compile() {
    let code = "namespace test { @EntryPoint() operation Foo(): Result {
    use q = Qubit();
    H(q);
    M(q)
    }}";
    let result = get_qir(code);
    assert!(result.is_ok());
}

#[test]
fn test_run_two_shots() {
    let code = "
        namespace Test {
            function Answer() : Int {
                return 42;
            }
        }
    ";
    let expr = "Test.Answer()";
    let count = std::cell::Cell::new(0);

    let _result = crate::run_internal(
        code,
        expr,
        |_msg| {
            assert!(_msg.contains("42"));
            count.set(count.get() + 1);
        },
        2,
    );
    assert_eq!(count.get(), 2);
}

#[test]
fn fail_ry() {
    let code = "namespace Sample {
        operation main() : Result[] {
            use q1 = Qubit();
            Ry(q1);
            let m1 = M(q1);
            return [m1];
        }
    }";

    let (_, errors) = crate::compile(code);
    assert_eq!(errors.len(), 1, "{errors:#?}");

    let error = errors.first().unwrap();
    assert_eq!(error.start_pos, 99);
    assert_eq!(error.end_pos, 105);
    assert_eq!(
        error.message,
        "type error: expected (Double, Qubit), found Qubit"
    );
}

#[test]
fn test_message() {
    let code = r#"namespace Sample {
        open Microsoft.Quantum.Diagnostics;

        operation main() : Unit {
            Message("hi");
            return ();
        }
    }"#;
    let expr = "Sample.main()";
    let result = crate::run_internal(
        code,
        expr,
        |_msg_| {
            assert!(_msg_.contains("hi") || _msg_.contains("result"));
        },
        1,
    );
    assert!(result.is_ok());
}
#[test]
fn message_with_escape_sequences() {
    let code = r#"namespace Sample {
        open Microsoft.Quantum.Diagnostics;

        operation main() : Unit {
            Message("\ta\n\t");

            return ();
        }
    }"#;
    let expr = "Sample.main()";
    let result = crate::run_internal(
        code,
        expr,
        |_msg_| {
            assert!(_msg_.contains(r"\ta\n\t") || _msg_.contains("result"));
        },
        1,
    );
    assert!(result.is_ok());
}
#[test]
fn message_with_backslashes() {
    let code = r#"namespace Sample {
        open Microsoft.Quantum.Diagnostics;

        operation main() : Unit {
            Message("hi \\World");
            Message("hello { \\World [");

            return ();
        }
    }"#;
    let expr = "Sample.main()";
    let result = crate::run_internal(
        code,
        expr,
        |_msg_| {
            assert!(
                _msg_.contains("hello { \\\\World [")
                    || _msg_.contains("hi \\\\World")
                    || _msg_.contains("result")
            );
        },
        1,
    );
    assert!(result.is_ok());
}

#[test]
fn test_entrypoint() {
    let code = r#"namespace Sample {
        @EntryPoint()
        operation main() : Unit {
            Message("hi");
            return ();
        }
    }"#;
    let expr = "";
    let result = crate::run_internal(
        code,
        expr,
        |_msg_| {
            assert!(_msg_.contains("hi") || _msg_.contains("result"));
        },
        1,
    );
    assert!(result.is_ok());
}

#[test]
fn test_missing_entrypoint() {
    let code = "namespace Sample {
        operation main() : Result[] {
            use q1 = Qubit();
            let m1 = M(q1);
            return [m1];
        }
    }";
    let expr = "";
    let result = crate::run_internal(
        code,
        expr,
        |msg| {
            assert!(msg.contains(r#""success":false"#));
            assert!(msg.contains(r#""message":"entry point not found"#));
            assert!(msg.contains(r#""start_pos":0"#));
        },
        1,
    );
    assert!(result.is_err());
}

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
