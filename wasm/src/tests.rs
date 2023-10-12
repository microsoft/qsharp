// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;
use indoc::indoc;

use super::run_internal;

use crate::get_qir;

#[test]
fn test_missing_type() {
    let code = "namespace input { operation Foo(a) : Unit {} }";
    let expr = "";
    let count = std::cell::Cell::new(0);

    let _ = run_internal(
        code,
        expr,
        |msg| {
            expect![[r#"{"result":{"code":"Qsc.TypeCk.MissingItemTy","end_pos":33,"message":"type error: missing type in item signature\n\nhelp: types cannot be inferred for global declarations","severity":"error","start_pos":32},"success":false,"type":"Result"}"#]].assert_eq(msg);
            count.set(count.get() + 1);
        },
        1,
    );
    assert_eq!(count.get(), 1);
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
    let expr = "";
    let count = std::cell::Cell::new(0);

    let _result = run_internal(
        code,
        expr,
        |msg| {
            expect![[r#"{"result":{"code":"Qsc.TypeCk.TyMismatch","end_pos":105,"message":"type error: expected (Double, Qubit), found Qubit","severity":"error","start_pos":99},"success":false,"type":"Result"}"#]].assert_eq(msg);
            count.set(count.get() + 1);
        },
        1,
    );
    assert_eq!(count.get(), 1);
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
        {"result":{"code":"Qsc.Eval.QubitUniqueness","end_pos":1,"message":"runtime error: qubits in gate invocation are not unique","severity":"error","start_pos":0},"success":false,"type":"Result"}
        {"result":{"code":"Qsc.Eval.QubitUniqueness","end_pos":1,"message":"runtime error: qubits in gate invocation are not unique","severity":"error","start_pos":0},"success":false,"type":"Result"}
        {"result":{"code":"Qsc.Eval.QubitUniqueness","end_pos":1,"message":"runtime error: qubits in gate invocation are not unique","severity":"error","start_pos":0},"success":false,"type":"Result"}"#]]
    .assert_eq(&output.join("\n"));
}

#[test]
fn test_runtime_error_with_span() {
    let mut output = Vec::new();
    run_internal(
        indoc! {r#"
            namespace Test {
                @EntryPoint()
                operation Main() : Unit {
                    fail "hello"
                }
            }"#
        },
        "",
        |s| output.push(s.to_string()),
        3,
    )
    .expect("code should compile and run");
    expect![[r#"
        {"result":{"code":"Qsc.Eval.UserFail","end_pos":85,"message":"runtime error: program failed: hello","related":[{"end_pos":85,"message":"explicit fail","source":"code","start_pos":73}],"severity":"error","start_pos":73},"success":false,"type":"Result"}
        {"result":{"code":"Qsc.Eval.UserFail","end_pos":85,"message":"runtime error: program failed: hello","related":[{"end_pos":85,"message":"explicit fail","source":"code","start_pos":73}],"severity":"error","start_pos":73},"success":false,"type":"Result"}
        {"result":{"code":"Qsc.Eval.UserFail","end_pos":85,"message":"runtime error: program failed: hello","related":[{"end_pos":85,"message":"explicit fail","source":"code","start_pos":73}],"severity":"error","start_pos":73},"success":false,"type":"Result"}"#]]
    .assert_eq(&output.join("\n"));
}

#[test]
fn test_compile_error_related_spans() {
    let mut output = Vec::new();
    run_internal(
        indoc! {r#"
            namespace Other { operation DumpMachine() : Unit { } }
            namespace Test {
                open Other;
                open Microsoft.Quantum.Diagnostics;
                @EntryPoint()
                operation Main() : Unit {
                    DumpMachine()
                }
            }
        "#
        },
        "",
        |s| output.push(s.to_string()),
        1,
    )
    .expect_err("code should fail to compile");
    expect![[r#"{"result":{"code":"Qsc.Resolve.Ambiguous","end_pos":195,"message":"name error: `DumpMachine` could refer to the item in `Other` or `Microsoft.Quantum.Diagnostics`","related":[{"end_pos":195,"message":"ambiguous name","source":"code","start_pos":184},{"end_pos":86,"message":"found in this namespace","source":"code","start_pos":81},{"end_pos":126,"message":"and also in this namespace","source":"code","start_pos":97}],"severity":"error","start_pos":184},"success":false,"type":"Result"}"#]]
    .assert_eq(&output.join("\n"));
}

#[test]
fn test_runtime_error_related_spans() {
    let mut output = Vec::new();
    run_internal(
        indoc! {r#"
            namespace Test {
                @EntryPoint()
                operation Main() : Unit {
                    use q = Qubit();
                    X(q);
                }
            }
        "#
        },
        "",
        |s| output.push(s.to_string()),
        1,
    )
    .expect("code should compile and run");
    expect![[r#"{"result":{"code":"Qsc.Eval.ReleasedQubitNotZero","end_pos":89,"message":"runtime error: Qubit0 released while not in |0⟩ state\n\nhelp: qubits should be returned to the |0⟩ state before being released to satisfy the assumption that allocated qubits start in the |0⟩ state","related":[{"end_pos":89,"message":"Qubit0","source":"code","start_pos":73}],"severity":"error","start_pos":73},"success":false,"type":"Result"}"#]]
    .assert_eq(&output.join("\n"));
}

#[test]
fn test_runtime_error_default_span() {
    let mut output = Vec::new();
    run_internal(
        indoc! {r#"
            namespace Test {
                @EntryPoint()
                operation Main() : Unit {
                    use qs = Qubit[-1];
                }
            }
        "#
        },
        "",
        |s| output.push(s.to_string()),
        1,
    )
    .expect("code should compile and run");
    expect![[r#"{"result":{"code":"Qsc.Eval.UserFail","end_pos":1,"message":"runtime error: program failed: Cannot allocate qubit array with a negative length","related":[{"end_pos":429,"message":"explicit fail","source":"core/qir.qs","start_pos":372}],"severity":"error","start_pos":0},"success":false,"type":"Result"}"#]]
    .assert_eq(&output.join("\n"));
}
