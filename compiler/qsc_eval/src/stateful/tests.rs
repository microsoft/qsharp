// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod given_interpreter {
    use std::io::Cursor;

    use crate::{
        output::CursorReceiver,
        stateful::{Interpreter, InterpreterResult},
        val::Value,
    };
    fn line(
        interpreter: &mut Interpreter,
        line: impl AsRef<str>,
    ) -> (impl Iterator<Item = InterpreterResult>, String) {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        let mut receiver = CursorReceiver::new(&mut cursor);
        (interpreter.line(&mut receiver, line), receiver.dump())
    }

    mod without_sources {
        use super::*;

        mod without_stdlib {
            use super::*;
            #[test]
            fn stdlib_members_should_be_unavailable() {
                const SOURCES: [&str; 0] = [];
                let mut interpreter =
                    Interpreter::new(true, SOURCES).expect("Failed to compile base environment.");

                let results = line(&mut interpreter, "Message(\"_\")");
                is_only_error(results, "`Message` not found in this scope");
            }
        }

        #[test]
        fn stdlib_members_should_be_available() {
            let mut interpreter = get_interpreter();
            let results = line(&mut interpreter, "Message(\"_\")");
            is_unit_with_output(results, "_");
        }

        #[test]
        fn let_bindings_update_interpreter() {
            let mut interpreter = get_interpreter();
            let _ = line(&mut interpreter, "let y = 7;");
            let results = line(&mut interpreter, "y");
            is_only_value(results, &Value::Int(7));
        }

        #[test]
        fn let_bindings_can_be_shadowed() {
            let mut interpreter = get_interpreter();

            let results = line(&mut interpreter, "let y = 7;");
            is_only_value(results, &Value::UNIT);

            let results = line(&mut interpreter, "y");
            is_only_value(results, &Value::Int(7));

            let results = line(&mut interpreter, "let y = \"Hello\";");
            is_only_value(results, &Value::UNIT);

            let results = line(&mut interpreter, "y");
            is_only_value(results, &Value::String("Hello".to_string()));
        }

        #[test]
        fn invalid_statements_return_error() {
            let mut interpreter = get_interpreter();

            let results = line(&mut interpreter, "let y = 7");
            is_only_error(results, "expected `;`, found EOF");

            let results = line(&mut interpreter, "y");
            is_only_error(results, "`y` not found in this scope");
        }

        #[test]
        fn failing_statements_return_early_error() {
            let mut interpreter = get_interpreter();

            let (results, output) = line(&mut interpreter, "let y = 7;y/0;y");
            let results = results.collect::<Vec<_>>();
            assert_eq!(results.len(), 2);

            is_only_value(
                ([results[0].clone()].into_iter(), output.clone()),
                &Value::UNIT,
            );

            is_only_error(
                ([results[1].clone()].into_iter(), output),
                "division by zero",
            );
        }
    }

    #[cfg(test)]
    mod with_sources {
        use super::*;
        use indoc::indoc;

        #[test]
        fn stdlib_members_can_be_accessed_from_sources() {
            let source = indoc! { r#"
            namespace Test {
                operation Main() : Unit {
                    Message("hello there...")
                }
            }"#};

            let mut interpreter =
                Interpreter::new(false, [source]).expect("Failed to compile base environment.");
            let results = line(&mut interpreter, "Test.Main()");
            is_unit_with_output(results, "hello there...");
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

            let mut interpreter =
                Interpreter::new(false, [source]).expect("Failed to compile base environment.");
            let results = line(&mut interpreter, "Test.Hello()");
            is_only_value(results, &Value::String("hello there...".to_string()));
            let results = line(&mut interpreter, "Test.Main()");
            is_only_value(results, &Value::String("hello there...".to_string()));
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

            let mut interpreter =
                Interpreter::new(false, [source]).expect("Failed to compile base environment.");
            let results = line(&mut interpreter, "Test.Hello()");
            is_only_value(results, &Value::String("hello there...".to_string()));
            let results = line(&mut interpreter, "Test2.Main()");
            is_only_value(results, &Value::String("hello there...".to_string()));
        }
    }

    fn get_interpreter() -> Interpreter {
        const SOURCES: [&str; 0] = [];
        Interpreter::new(false, SOURCES).expect("Failed to compile base environment.")
    }

    fn is_only_value(results: (impl Iterator<Item = InterpreterResult>, String), value: &Value) {
        assert_eq!("", results.1);

        let results = results.0.collect::<Vec<_>>();
        let result = &results[0];
        assert_eq!(value, &result.value);
        assert_eq!(0, result.errors.len());
    }

    fn is_unit_with_output(
        results: (impl Iterator<Item = InterpreterResult>, String),
        output: &str,
    ) {
        assert_eq!(output, results.1);

        let results = results.0.collect::<Vec<_>>();
        let result = &results[0];
        assert_eq!(Value::UNIT, result.value);
        assert_eq!(0, result.errors.len());
    }

    fn is_only_error(results: (impl Iterator<Item = InterpreterResult>, String), error: &str) {
        assert_eq!("", results.1);

        let results = results.0.collect::<Vec<_>>();
        let result = &results[0];
        assert_eq!(Value::UNIT, result.value);
        assert_eq!(1, result.errors.len());
        assert_eq!(error, result.errors[0].to_string());
    }
}
