// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod given_interpreter {
    use crate::interactive::{Interpreter, InterpreterResult};

    mod without_sources {
        use super::*;

        mod without_stdlib {
            use super::*;
            #[test]
            fn stdlib_members_should_be_unavailable() {
                const SOURCES: [&str; 0] = [];
                let mut interpreter =
                    Interpreter::new(true, SOURCES).expect("Failed to compile base environment.");

                let result = &interpreter.line("Message(\"_\")")[0];
                is_only_error(result, "`Message` not found in this scope");
            }
        }

        #[test]
        fn stdlib_members_should_be_available() {
            let mut interpreter = get_interpreter();
            let result = &interpreter.line("Message(\"_\")")[0];
            is_unit_with_output(result, "_");
        }

        #[test]
        fn let_bindings_update_interpreter() {
            let mut interpreter = get_interpreter();
            let _ = &interpreter.line("let y = 7;")[0];
            let result = &interpreter.line("y")[0];
            assert_eq!("7", result.value);
        }

        #[test]
        fn let_bindings_can_be_shadowed() {
            let mut interpreter = get_interpreter();

            let result = &interpreter.line("let y = 7;")[0];
            is_only_value(result, "()");

            let result = &interpreter.line("y")[0];
            is_only_value(result, "7");

            let result = &interpreter.line("let y = \"Hello\";")[0];
            is_only_value(result, "()");

            let result = &interpreter.line("y")[0];
            is_only_value(result, "Hello");
        }

        #[test]
        fn invalid_statements_return_error() {
            let mut interpreter = get_interpreter();

            let result = &interpreter.line("let y = 7")[0];
            is_only_error(result, "expected `;`, found EOF");

            let result = &interpreter.line("y")[0];
            is_only_error(result, "`y` not found in this scope");
        }

        #[test]
        fn failing_statements_return_early_error() {
            let mut interpreter = get_interpreter();

            let results = &interpreter.line("let y = 7;y/0;y");
            assert_eq!(results.len(), 2);
            let result = &results[0];
            is_only_value(result, "()");

            let result = &results[1];
            is_only_error(result, "division by zero");
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
            let result = &interpreter.line("Test.Main()")[0];
            is_unit_with_output(result, "hello there...");
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
            let result = &interpreter.line("Test.Hello()")[0];
            is_only_value(result, "hello there...");
            let result = &interpreter.line("Test.Main()")[0];
            is_only_value(result, "hello there...");
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
            let result = &interpreter.line("Test.Hello()")[0];
            is_only_value(result, "hello there...");
            let result = &interpreter.line("Test2.Main()")[0];
            is_only_value(result, "hello there...");
        }
    }

    fn get_interpreter() -> Interpreter {
        const SOURCES: [&str; 0] = [];
        Interpreter::new(false, SOURCES).expect("Failed to compile base environment.")
    }

    fn is_only_value(result: &InterpreterResult, value: &str) {
        assert_eq!(value, result.value);
        assert_eq!("", result.output);
        assert_eq!(0, result.errors.len());
    }

    fn is_unit_with_output(result: &InterpreterResult, output: &str) {
        assert_eq!("()", result.value);
        assert_eq!(output, result.output);
        assert_eq!(0, result.errors.len());
    }

    fn is_only_error(result: &InterpreterResult, error: &str) {
        assert_eq!("", result.value);
        assert_eq!("", result.output);
        assert_eq!(1, result.errors.len());
        assert_eq!(error, result.errors[0].to_string());
    }
}
