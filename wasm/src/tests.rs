// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;
use indoc::indoc;
use qsc::SourceMap;

use super::run_internal;

#[test]
fn test_missing_type() {
    let code = "namespace input { operation Foo(a) : Unit {} }";
    let expr = "";
    let count = std::cell::Cell::new(0);

    let _ = run_internal(
        SourceMap::new([("test.qs".into(), code.into())], Some(expr.into())),
        |msg| {
            expect![[r#"{"result":{"code":"Qsc.TypeCk.MissingItemTy","message":"type error: missing type in item signature\n\nhelp: types cannot be inferred for global declarations","range":{"end":{"character":33,"line":0},"start":{"character":32,"line":0}},"severity":"error"},"success":false,"type":"Result"}"#]].assert_eq(msg);
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

    let result = crate::_get_qir(SourceMap::new([("test.qs".into(), code.into())], None));
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
        SourceMap::new([("test.qs".into(), code.into())], Some(expr.into())),
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
        SourceMap::new([("test.qs".into(), code.into())], Some(expr.into())),
        |msg| {
            expect![[r#"{"result":{"code":"Qsc.TypeCk.TyMismatch","message":"type error: expected (Double, Qubit), found Qubit","range":{"end":{"character":18,"line":3},"start":{"character":12,"line":3}},"severity":"error"},"success":false,"type":"Result"}"#]].assert_eq(msg);
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
        SourceMap::new([("test.qs".into(), code.into())], Some(expr.into())),
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
        SourceMap::new([("test.qs".into(), code.into())], Some(expr.into())),
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
        SourceMap::new([("test.qs".into(), code.into())], Some(expr.into())),
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
        SourceMap::new([("test.qs".into(), code.into())], Some(expr.into())),
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
        SourceMap::new([("test.qs".into(), code.into())], Some(expr.into())),
        |msg| {
            expect![[r#"{"result":{"code":"Qsc.EntryPoint.NotFound","message":"entry point not found\n\nhelp: a single callable with the `@EntryPoint()` attribute must be present if no entry expression is provided","range":{"end":{"character":1,"line":0},"start":{"character":0,"line":0}},"severity":"error"},"success":false,"type":"Result"}"#]].assert_eq(msg)
        },
        1,
    );
    assert!(result.is_err());
}

#[test]
fn test_run_simple_program_multiple_shots() {
    let mut output = Vec::new();
    let code = indoc! {"
            namespace Test {
                @EntryPoint()
                operation Main() : Int { 4 }
            }"
    };
    run_internal(
        SourceMap::new([("code".into(), code.into())], None),
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
    let code = indoc! {"
            namespace Test {
                @EntryPoint()
                operation Main() : Unit {
                    use q = Qubit();
                    CNOT(q, q)
                }
            }"
    };
    run_internal(
        SourceMap::new([("test.qs".into(), code.into())], None),
        |s| output.push(s.to_string()),
        3,
    )
    .expect("code should compile and run");
    expect![[r#"
        {"result":{"code":"Qsc.Eval.QubitUniqueness","message":"runtime error: qubits in gate invocation are not unique","range":{"end":{"character":1,"line":0},"start":{"character":0,"line":0}},"severity":"error"},"success":false,"type":"Result"}
        {"result":{"code":"Qsc.Eval.QubitUniqueness","message":"runtime error: qubits in gate invocation are not unique","range":{"end":{"character":1,"line":0},"start":{"character":0,"line":0}},"severity":"error"},"success":false,"type":"Result"}
        {"result":{"code":"Qsc.Eval.QubitUniqueness","message":"runtime error: qubits in gate invocation are not unique","range":{"end":{"character":1,"line":0},"start":{"character":0,"line":0}},"severity":"error"},"success":false,"type":"Result"}"#]]
    .assert_eq(&output.join("\n"));
}

#[test]
fn test_run_error_program_multiple_shots_qubit_leak() {
    // If qubits are leaked from execution, the runtime will fail with an out of memory
    // error pretty quickly.
    let mut output = Vec::new();
    let code = indoc! {"
            namespace Test {
                @EntryPoint()
                operation Main() : Unit {
                    use q = Qubit();
                    H(q);
                }
            }"
    };
    run_internal(
        SourceMap::new([("code".into(), code.into())], None),
        |s| output.push(s.to_string()),
        100,
    )
    .expect("code should compile and run");

    // Spot check the results to make sure we're getting the right error.
    expect![[r#"{"result":{"code":"Qsc.Eval.ReleasedQubitNotZero","message":"runtime error: Qubit0 released while not in |0⟩ state\n\nhelp: qubits should be returned to the |0⟩ state before being released to satisfy the assumption that allocated qubits start in the |0⟩ state","range":{"end":{"character":24,"line":3},"start":{"character":8,"line":3}},"related":[{"location":{"source":"code","span":{"end":{"character":24,"line":3},"start":{"character":8,"line":3}}},"message":"Qubit0"}],"severity":"error"},"success":false,"type":"Result"}"#]]
        .assert_eq(&output[0]);
    expect![[r#"{"result":{"code":"Qsc.Eval.ReleasedQubitNotZero","message":"runtime error: Qubit0 released while not in |0⟩ state\n\nhelp: qubits should be returned to the |0⟩ state before being released to satisfy the assumption that allocated qubits start in the |0⟩ state","range":{"end":{"character":24,"line":3},"start":{"character":8,"line":3}},"related":[{"location":{"source":"code","span":{"end":{"character":24,"line":3},"start":{"character":8,"line":3}}},"message":"Qubit0"}],"severity":"error"},"success":false,"type":"Result"}"#]]
        .assert_eq(&output[50]);
    expect![[r#"{"result":{"code":"Qsc.Eval.ReleasedQubitNotZero","message":"runtime error: Qubit0 released while not in |0⟩ state\n\nhelp: qubits should be returned to the |0⟩ state before being released to satisfy the assumption that allocated qubits start in the |0⟩ state","range":{"end":{"character":24,"line":3},"start":{"character":8,"line":3}},"related":[{"location":{"source":"code","span":{"end":{"character":24,"line":3},"start":{"character":8,"line":3}}},"message":"Qubit0"}],"severity":"error"},"success":false,"type":"Result"}"#]]
        .assert_eq(&output[99]);
}

#[test]
fn test_runtime_error_with_span() {
    let mut output = Vec::new();
    let code = indoc! {r#"
            namespace Test {
                @EntryPoint()
                operation Main() : Unit {
                    fail "hello"
                }
            }"#
    };
    run_internal(
        SourceMap::new([("test.qs".into(), code.into())], None),
        |s| output.push(s.to_string()),
        3,
    )
    .expect("code should compile and run");
    expect![[r#"
        {"result":{"code":"Qsc.Eval.UserFail","message":"runtime error: program failed: hello","range":{"end":{"character":20,"line":3},"start":{"character":8,"line":3}},"related":[{"location":{"source":"test.qs","span":{"end":{"character":20,"line":3},"start":{"character":8,"line":3}}},"message":"explicit fail"}],"severity":"error"},"success":false,"type":"Result"}
        {"result":{"code":"Qsc.Eval.UserFail","message":"runtime error: program failed: hello","range":{"end":{"character":20,"line":3},"start":{"character":8,"line":3}},"related":[{"location":{"source":"test.qs","span":{"end":{"character":20,"line":3},"start":{"character":8,"line":3}}},"message":"explicit fail"}],"severity":"error"},"success":false,"type":"Result"}
        {"result":{"code":"Qsc.Eval.UserFail","message":"runtime error: program failed: hello","range":{"end":{"character":20,"line":3},"start":{"character":8,"line":3}},"related":[{"location":{"source":"test.qs","span":{"end":{"character":20,"line":3},"start":{"character":8,"line":3}}},"message":"explicit fail"}],"severity":"error"},"success":false,"type":"Result"}"#]]
    .assert_eq(&output.join("\n"));
}

// Need to revisit spans in output: https://github.com/microsoft/qsharp/issues/944
#[test]
fn test_runtime_error_in_another_file_with_project() {
    let mut output = Vec::new();
    let first = indoc! {r#"
            namespace Test {
                @EntryPoint()
                operation Main() : Unit {
                    Test.other()
                }
            }"#
    };
    let second = indoc! {r#"
            namespace Test {
                operation other() : Unit {
                    fail "hello"
                }
            }"#
    };
    run_internal(
        SourceMap::new(
            [
                ("test1.qs".into(), first.into()),
                ("test2.qs".into(), second.into()),
            ],
            None,
        ),
        |s| output.push(s.to_string()),
        1,
    )
    .expect("code should compile and run");
    expect![[r#"{"result":{"code":"Qsc.Eval.UserFail","message":"runtime error: program failed: hello","range":{"end":{"character":1,"line":0},"start":{"character":0,"line":0}},"related":[{"location":{"source":"test2.qs","span":{"end":{"character":20,"line":2},"start":{"character":8,"line":2}}},"message":"explicit fail"}],"severity":"error"},"success":false,"type":"Result"}"#]]
    .assert_eq(&output.join("\n"));
}

#[test]
fn test_runtime_error_with_failure_in_main_file_project() {
    let mut output = Vec::new();
    let first = indoc! {r#"
            namespace Test {
                @EntryPoint()
                operation Main() : Unit {
                    Test.other()
                }
                operation failing_call() : Unit {
                    fail "hello"
                }
            }"#
    };
    let second = indoc! {r#"
            namespace Test {
                operation other() : Unit {
                    Test.failing_call()
                }
            }"#
    };
    run_internal(
        SourceMap::new(
            [
                ("test1.qs".into(), first.into()),
                ("test2.qs".into(), second.into()),
            ],
            None,
        ),
        |s| output.push(s.to_string()),
        1,
    )
    .expect("code should compile and run");
    expect![[r#"{"result":{"code":"Qsc.Eval.UserFail","message":"runtime error: program failed: hello","range":{"end":{"character":20,"line":6},"start":{"character":8,"line":6}},"related":[{"location":{"source":"test1.qs","span":{"end":{"character":20,"line":6},"start":{"character":8,"line":6}}},"message":"explicit fail"}],"severity":"error"},"success":false,"type":"Result"}"#]]
    .assert_eq(&output.join("\n"));
}

#[test]
fn test_compile_error_related_spans() {
    let mut output = Vec::new();
    let code = indoc! {r#"
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
    };
    run_internal(
        SourceMap::new([("test.qs".into(), code.into())], None),
        |s| output.push(s.to_string()),
        1,
    )
    .expect_err("code should fail to compile");
    expect![[r#"{"result":{"code":"Qsc.Resolve.Ambiguous","message":"name error: `DumpMachine` could refer to the item in `Other` or `Microsoft.Quantum.Diagnostics`","range":{"end":{"character":19,"line":6},"start":{"character":8,"line":6}},"related":[{"location":{"source":"test.qs","span":{"end":{"character":19,"line":6},"start":{"character":8,"line":6}}},"message":"ambiguous name"},{"location":{"source":"test.qs","span":{"end":{"character":14,"line":2},"start":{"character":9,"line":2}}},"message":"found in this namespace"},{"location":{"source":"test.qs","span":{"end":{"character":38,"line":3},"start":{"character":9,"line":3}}},"message":"and also in this namespace"}],"severity":"error"},"success":false,"type":"Result"}"#]]
    .assert_eq(&output.join("\n"));
}

#[test]
fn test_runtime_error_related_spans() {
    let mut output = Vec::new();
    let code = indoc! {r#"
            namespace Test {
                @EntryPoint()
                operation Main() : Unit {
                    use q = Qubit();
                    X(q);
                }
            }
        "#
    };
    run_internal(
        SourceMap::new([("test.qs".into(), code.into())], None),
        |s| output.push(s.to_string()),
        1,
    )
    .expect("code should compile and run");
    expect![[r#"{"result":{"code":"Qsc.Eval.ReleasedQubitNotZero","message":"runtime error: Qubit0 released while not in |0⟩ state\n\nhelp: qubits should be returned to the |0⟩ state before being released to satisfy the assumption that allocated qubits start in the |0⟩ state","range":{"end":{"character":24,"line":3},"start":{"character":8,"line":3}},"related":[{"location":{"source":"test.qs","span":{"end":{"character":24,"line":3},"start":{"character":8,"line":3}}},"message":"Qubit0"}],"severity":"error"},"success":false,"type":"Result"}"#]]
    .assert_eq(&output.join("\n"));
}

#[test]
fn test_runtime_error_default_span() {
    let mut output = Vec::new();
    let code = indoc! {r#"
            namespace Test {
                @EntryPoint()
                operation Main() : Unit {
                    use qs = Qubit[-1];
                }
            }
        "#
    };
    run_internal(
        SourceMap::new([("test.qs".into(), code.into())], None),
        |s| output.push(s.to_string()),
        1,
    )
    .expect("code should compile and run");
    expect![[r#"{"result":{"code":"Qsc.Eval.UserFail","message":"runtime error: program failed: Cannot allocate qubit array with a negative length","range":{"end":{"character":1,"line":0},"start":{"character":0,"line":0}},"related":[{"location":{"source":"core/qir.qs","span":{"end":{"character":69,"line":14},"start":{"character":12,"line":14}}},"message":"explicit fail"}],"severity":"error"},"success":false,"type":"Result"}"#]]
    .assert_eq(&output.join("\n"));
}
