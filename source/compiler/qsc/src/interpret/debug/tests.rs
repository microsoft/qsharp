// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;
use indoc::indoc;
use miette::Result;
use qsc_data_structures::{language_features::LanguageFeatures, target::TargetCapabilityFlags};
use qsc_eval::{output::CursorReceiver, val::Value};
use qsc_frontend::compile::SourceMap;
use qsc_passes::PackageType;
use std::io::Cursor;

use crate::interpret::{Error, InterpretResult, Interpreter};

fn line(interpreter: &mut Interpreter, line: impl AsRef<str>) -> (InterpretResult, String) {
    let mut cursor = Cursor::new(Vec::<u8>::new());
    let mut receiver = CursorReceiver::new(&mut cursor);
    (
        interpreter.eval_fragments(&mut receiver, line.as_ref()),
        receiver.dump(),
    )
}

fn eval(interpreter: &mut Interpreter) -> (Result<Value, Vec<Error>>, String) {
    let mut cursor = Cursor::new(Vec::<u8>::new());
    let mut receiver = CursorReceiver::new(&mut cursor);
    (interpreter.eval_entry(&mut receiver), receiver.dump())
}

#[test]
fn stack_traces_can_cross_eval_session_and_file_boundaries() {
    let source1 = indoc! { r#"
        namespace Test {
            operation B(input : Int) : Unit is Adj {
                body ... {
                    C(input)
                }
                adjoint invert;
            }

            operation C(input : Int) : Unit is Adj {
                body ... {
                    1 / input;
                }
                adjoint self;
            }
        }
        "#};
    let source2 = indoc! { r#"
        namespace Test2 {
            open Test;
            operation A(input : Int) : Unit is Adj {
                body ... {
                    B(input)
                }
                adjoint invert;
            }
        }
        "#};

    let source_map = SourceMap::new(
        [
            ("1.qs".into(), source1.into()),
            ("2.qs".into(), source2.into()),
        ],
        None,
    );

    let (std_id, store) = crate::compile::package_store_with_stdlib(TargetCapabilityFlags::all());
    let mut interpreter = Interpreter::new(
        source_map,
        PackageType::Lib,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
        store,
        &[(std_id, None)],
    )
    .expect("Failed to compile base environment.");

    let (result, _) = line(
        &mut interpreter,
        "operation Z(input : Int) : Unit { Adjoint Test2.A(input); }",
    );
    result.expect("code should compile");

    let (result, _output) = line(&mut interpreter, "Z(0)");

    match result {
        Ok(_) => panic!("Expected error"),
        Err(e) => {
            let stack_trace = e[0]
                .stack_trace()
                .expect("code should have a valid stack trace");
            expect![[r#"
                Error: division by zero
                Call stack:
                    at Adjoint Test.C in 1.qs:11:12
                    at Adjoint Test.B in 1.qs:4:12
                    at Adjoint Test2.A in 2.qs:10:0
                    at Z in line_0:1:34
            "#]]
            .assert_eq(stack_trace);
        }
    }
}

#[test]
fn test_exact_issue_reproduction() {
    // This matches the exact code structure from the issue description
    let source = indoc! { r#"
        namespace Test {
            function Main() : Unit {
                fail "line 3";
            }
        }
        "#};

    let source_map = SourceMap::new(
        [("test.qs".into(), source.into())],
        Some("Test.Main()".into()),
    );

    let (std_id, store) = crate::compile::package_store_with_stdlib(TargetCapabilityFlags::all());
    let mut interpreter = Interpreter::new(
        source_map,
        PackageType::Exe,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
        store,
        &[(std_id, None)],
    )
    .expect("Failed to compile base environment.");

    let (result, _) = eval(&mut interpreter);

    match result {
        Ok(_) => panic!("Expected error"),
        Err(e) => {
            let stack_trace = e[0]
                .stack_trace()
                .expect("code should have a valid stack trace");
            println!("Stack trace for exact issue reproduction:");
            println!("{}", stack_trace);
            
            // The fail statement is on line 3 (1-based counting):
            // Line 1: namespace Test {
            // Line 2:     function Main() : Unit {
            // Line 3:         fail "line 3";
            // Line 4:     }
            // Line 5: }
            // So it should show "line 3" in the stack trace (not line 2 as before the fix)
            assert!(stack_trace.contains("test.qs:3:"));
            assert!(!stack_trace.contains("test.qs:2:") || !stack_trace.contains("Test.Main"));
        }
    }
}

#[test]
fn test_line_number_off_by_one_issue() {
    let source = indoc! { r#"
        namespace Test {
            function Main() : Unit {
                fail "line 3";
            }
        }
        "#};

    let source_map = SourceMap::new(
        [("test.qs".into(), source.into())],
        Some("Test.Main()".into()),
    );

    let (std_id, store) = crate::compile::package_store_with_stdlib(TargetCapabilityFlags::all());
    let mut interpreter = Interpreter::new(
        source_map,
        PackageType::Exe,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
        store,
        &[(std_id, None)],
    )
    .expect("Failed to compile base environment.");

    let (result, _) = eval(&mut interpreter);

    match result {
        Ok(_) => panic!("Expected error"),
        Err(e) => {
            let stack_trace = e[0]
                .stack_trace()
                .expect("code should have a valid stack trace");
            println!("Current stack trace:");
            println!("{}", stack_trace);
            // The fail statement is on line 3 (1-based), so it should show line 3
            expect![[r#"
                Error: program failed: line 3
                Call stack:
                    at Test.Main in test.qs:3:20
            "#]]
            .assert_eq(stack_trace);
        }
    }
}

#[test]
fn stack_traces_can_cross_file_and_entry_boundaries() {
    let source1 = indoc! { r#"
        namespace Test {
            operation B(input : Int) : Unit is Adj {
                body ... {
                    C(input)
                }
                adjoint invert;
            }

            operation C(input : Int) : Unit is Adj {
                body ... {
                    1 / input;
                }
                adjoint self;
            }
        }
        "#};
    let source2 = indoc! { r#"
        namespace Test2 {
            open Test;
            operation A(input : Int) : Unit is Adj {
                body ... {
                    B(input)
                }
                adjoint invert;
            }
        }
        "#};

    let source_map = SourceMap::new(
        [
            ("1.qs".into(), source1.into()),
            ("2.qs".into(), source2.into()),
        ],
        Some("Adjoint Test2.A(0)".into()),
    );

    let (std_id, store) = crate::compile::package_store_with_stdlib(TargetCapabilityFlags::all());
    let mut interpreter = Interpreter::new(
        source_map,
        PackageType::Exe,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
        store,
        &[(std_id, None)],
    )
    .expect("Failed to compile base environment.");

    let (result, _) = eval(&mut interpreter);

    match result {
        Ok(_) => panic!("Expected error"),
        Err(e) => {
            let stack_trace = e[0]
                .stack_trace()
                .expect("code should have a valid stack trace");
            expect![[r#"
                Error: division by zero
                Call stack:
                    at Adjoint Test.C in 1.qs:12:8
                    at Adjoint Test.B in 1.qs:6:0
                    at Adjoint Test2.A in 2.qs:10:0
            "#]]
            .assert_eq(stack_trace);
        }
    }
}
