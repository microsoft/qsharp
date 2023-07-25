// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use indoc::indoc;
use miette::Result;
use qsc_eval::{output::CursorReceiver, val::Value};
use qsc_frontend::compile::SourceMap;
use std::io::Cursor;

use crate::interpret::{
    stateful::{Interpreter, LineError},
    stateless::{self},
};

fn line(
    interpreter: &mut Interpreter,
    line: impl AsRef<str>,
) -> (Result<Value, Vec<LineError>>, String) {
    let mut cursor = Cursor::new(Vec::<u8>::new());
    let mut receiver = CursorReceiver::new(&mut cursor);
    (
        interpreter.interpret_line(&mut receiver, line.as_ref()),
        receiver.dump(),
    )
}

fn eval(interpreter: &stateless::Interpreter) -> (Result<Value, Vec<stateless::Error>>, String) {
    let mut cursor = Cursor::new(Vec::<u8>::new());
    let mut receiver = CursorReceiver::new(&mut cursor);
    let mut eval_ctx = interpreter.new_eval_context();
    (eval_ctx.eval_entry(&mut receiver), receiver.dump())
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
    let mut interpreter =
        Interpreter::new(true, source_map).expect("Failed to compile base environment.");

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
                .as_ref()
                .expect("code should have a valid stack trace");
            let expectation = indoc! {r#"
                         Error: division by zero
                         Call stack:
                             at Adjoint Test.C in 1.qs
                             at Adjoint Test.B in 1.qs
                             at Adjoint Test2.A in 2.qs
                             at Z in <expression>
                    "#};
            assert_eq!(expectation, stack_trace);
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
        Some("Adjoint Test2.A(0);".into()),
    );
    let interpreter =
        stateless::Interpreter::new(true, source_map).expect("Failed to compile base environment.");

    let (result, _) = eval(&interpreter);

    match result {
        Ok(_) => panic!("Expected error"),
        Err(e) => {
            let stack_trace = e[0]
                .stack_trace()
                .as_ref()
                .expect("code should have a valid stack trace");
            let expectation = indoc! {r#"
                         Error: division by zero
                         Call stack:
                             at Adjoint Test.C in 1.qs
                             at Adjoint Test.B in 1.qs
                             at Adjoint Test2.A in 2.qs
                    "#};
            assert_eq!(expectation, stack_trace);
        }
    }
}
