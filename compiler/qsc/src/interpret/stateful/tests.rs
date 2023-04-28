// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod given_interpreter {
    use crate::interpret::stateful::{Interpreter, LineError};
    use qsc_eval::{output::CursorReceiver, val::Value};
    use qsc_frontend::compile::SourceMap;
    use std::{error::Error, fmt::Write, io::Cursor, iter};

    fn line(interpreter: &mut Interpreter, line: &str) -> (Result<Value, Vec<LineError>>, String) {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        let mut receiver = CursorReceiver::new(&mut cursor);
        (
            interpreter.interpret_line(&mut receiver, line),
            receiver.dump(),
        )
    }

    mod without_sources {
        use super::*;

        mod without_stdlib {
            use super::*;

            #[test]
            fn stdlib_members_should_be_unavailable() {
                let mut interpreter = Interpreter::new(false, SourceMap::default())
                    .expect("interpreter should be created");

                let (result, output) = line(&mut interpreter, "Message(\"_\")");
                is_only_error(
                    &result,
                    &output,
                    "name error: `Message` not found in this scope",
                );
            }
        }

        #[test]
        fn stdlib_members_should_be_available() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(&mut interpreter, "Message(\"_\")");
            is_unit_with_output(&result, &output, "_");
        }

        #[test]
        fn let_bindings_update_interpreter() {
            let mut interpreter = get_interpreter();
            line(&mut interpreter, "let y = 7;")
                .0
                .expect("line should succeed");
            let (result, output) = line(&mut interpreter, "y");
            is_only_value(&result, &output, &Value::Int(7));
        }

        #[test]
        fn let_bindings_can_be_shadowed() {
            let mut interpreter = get_interpreter();

            let (result, output) = line(&mut interpreter, "let y = 7;");
            is_only_value(&result, &output, &Value::unit());

            let (result, output) = line(&mut interpreter, "y");
            is_only_value(&result, &output, &Value::Int(7));

            let (result, output) = line(&mut interpreter, "let y = \"Hello\";");
            is_only_value(&result, &output, &Value::unit());

            let (result, output) = line(&mut interpreter, "y");
            is_only_value(&result, &output, &Value::String("Hello".into()));
        }

        #[test]
        fn invalid_statements_return_error() {
            let mut interpreter = get_interpreter();

            let (result, output) = line(&mut interpreter, "let y = 7");
            is_only_error(&result, &output, "syntax error: expected `;`, found EOF");

            let (result, output) = line(&mut interpreter, "y");
            is_only_error(&result, &output, "name error: `y` not found in this scope");
        }

        #[test]
        fn invalid_statements_and_unbound_vars_return_error() {
            let mut interpreter = get_interpreter();

            let (result, output) = line(&mut interpreter, "let y = x;");
            is_only_error(
                &result,
                &output,
                "name error: `x` not found in this scope",
            );

            let (result, output) = line(&mut interpreter, "y");
            is_only_error(
                &result,
                &output,
                "runtime error: variable is not bound",
            );
        }

        #[test]
        fn failing_statements_return_early_error() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(&mut interpreter, "let y = 7;y/0;y");
            is_only_error(&result, &output, "runtime error: division by zero");
        }
    }

    #[cfg(test)]
    mod with_sources {
        use super::*;
        use indoc::indoc;
        use qsc_frontend::compile::SourceMap;

        #[test]
        fn stdlib_members_can_be_accessed_from_sources() {
            let source = indoc! { r#"
            namespace Test {
                operation Main() : Unit {
                    Message("hello there...")
                }
            }"#};

            let sources = SourceMap::new([("test".into(), source.into())], None);
            let mut interpreter =
                Interpreter::new(true, sources).expect("interpreter should be created");
            let (result, output) = line(&mut interpreter, "Test.Main()");
            is_unit_with_output(&result, &output, "hello there...");
        }

        #[test]
        fn members_from_namespaced_sources_are_in_context() {
            let source = indoc! { r#"
            namespace Test {
                function Hello() : String {
                    "hello there..."
                }

                operation Main() : String {
                    Hello()
                }
            }"#};

            let sources = SourceMap::new([("test".into(), source.into())], None);
            let mut interpreter =
                Interpreter::new(true, sources).expect("interpreter should be created");
            let (result, output) = line(&mut interpreter, "Test.Hello()");
            is_only_value(&result, &output, &Value::String("hello there...".into()));
            let (result, output) = line(&mut interpreter, "Test.Main()");
            is_only_value(&result, &output, &Value::String("hello there...".into()));
        }

        #[test]
        fn multiple_namespaces_are_loaded_from_sources_into_eval_context() {
            let source = indoc! { r#"
            namespace Test {
                function Hello() : String {
                    "hello there..."
                }
            }
            namespace Test2 {
                open Test;
                operation Main() : String {
                    Hello()
                }
            }"#};

            let sources = SourceMap::new([("test".into(), source.into())], None);
            let mut interpreter =
                Interpreter::new(true, sources).expect("interpreter should be created");
            let (result, output) = line(&mut interpreter, "Test.Hello()");
            is_only_value(&result, &output, &Value::String("hello there...".into()));
            let (result, output) = line(&mut interpreter, "Test2.Main()");
            is_only_value(&result, &output, &Value::String("hello there...".into()));
        }
    }

    fn get_interpreter() -> Interpreter {
        Interpreter::new(true, SourceMap::default()).expect("interpreter should be created")
    }

    fn is_only_value(result: &Result<Value, Vec<LineError>>, output: &str, value: &Value) {
        assert_eq!("", output);

        match result {
            Ok(v) => assert_eq!(value, v),
            Err(e) => panic!("Expected unit value, got {e:?}"),
        }
    }

    fn is_unit_with_output(
        result: &Result<Value, Vec<LineError>>,
        output: &str,
        expected_output: &str,
    ) {
        assert_eq!(expected_output, output);

        match result {
            Ok(value) => assert_eq!(Value::unit(), *value),
            Err(e) => panic!("Expected unit value, got {e:?}"),
        }
    }

    fn is_only_error(result: &Result<Value, Vec<LineError>>, output: &str, error: &str) {
        assert_eq!("", output);

        match result {
            Ok(value) => panic!("Expected error , got {value:?}"),
            Err(errors) => {
                let mut message = errors[0].to_string();
                for source in iter::successors(errors[0].source(), |&e| e.source()) {
                    write!(message, ": {source}").expect("string should be writable");
                }
                assert_eq!(error, message);
            }
        }
    }
}
