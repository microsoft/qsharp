// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod given_interpreter {
    use crate::interpret::stateful::{Interpreter, LineError};
    use qsc_eval::{output::CursorReceiver, val::Value};
    use qsc_frontend::compile::{SourceMap, TargetProfile};
    use qsc_passes::PackageType;
    use std::{error::Error, fmt::Write, io::Cursor, iter};

    fn line(interpreter: &mut Interpreter, line: &str) -> (Result<Value, Vec<LineError>>, String) {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        let mut receiver = CursorReceiver::new(&mut cursor);
        (
            interpreter.interpret_line(&mut receiver, line),
            receiver.dump(),
        )
    }

    fn entry(
        interpreter: &mut Interpreter,
    ) -> (
        Result<Value, Vec<crate::interpret::stateful::Error>>,
        String,
    ) {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        let mut receiver = CursorReceiver::new(&mut cursor);
        (interpreter.eval_entry(&mut receiver), receiver.dump())
    }

    mod without_sources {
        use expect_test::expect;
        use indoc::indoc;

        use super::*;

        mod without_stdlib {
            use qsc_frontend::compile::{SourceMap, TargetProfile};
            use qsc_passes::PackageType;

            use super::*;

            #[test]
            fn stdlib_members_should_be_unavailable() {
                let mut interpreter = Interpreter::new(
                    false,
                    SourceMap::default(),
                    PackageType::Lib,
                    TargetProfile::Full,
                )
                .expect("interpreter should be created");

                let (result, output) = line(&mut interpreter, "Message(\"_\")");
                is_only_error(&result, &output, "name error: `Message` not found");
            }
        }

        #[test]
        fn stdlib_members_should_be_available() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(&mut interpreter, "Message(\"_\")");
            is_unit_with_output(&result, &output, "_");
        }

        #[test]
        fn core_members_should_be_available() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(&mut interpreter, "Length([1, 2, 3])");
            is_only_value(&result, &output, &Value::Int(3));
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
            is_only_error(&result, &output, "name error: `y` not found");
        }

        #[test]
        fn invalid_statements_and_unbound_vars_return_error() {
            let mut interpreter = get_interpreter();

            let (result, output) = line(&mut interpreter, "let y = x;");
            is_only_error(&result, &output, "name error: `x` not found");

            let (result, output) = line(&mut interpreter, "y");
            is_only_error(&result, &output, "runtime error: name is not bound");
        }

        #[test]
        fn failing_statements_return_early_error() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(&mut interpreter, "let y = 7;y/0;y");
            is_only_error(&result, &output, "runtime error: division by zero");
        }

        #[test]
        fn passes_are_run_on_incremental() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(
                &mut interpreter,
                "within {Message(\"A\");} apply {Message(\"B\");}",
            );
            is_unit_with_output(&result, &output, "A\nB\nA");
        }

        #[test]
        fn declare_function() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(&mut interpreter, "function Foo() : Int { 2 }");
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "Foo()");
            is_only_value(&result, &output, &Value::Int(2));
        }

        #[test]
        fn invalid_declare_function_and_unbound_call_return_error() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(&mut interpreter, "function Foo() : Int { invalid }");
            is_only_error(&result, &output, "name error: `invalid` not found");
            let (result, output) = line(&mut interpreter, "Foo()");
            is_only_error(&result, &output, "runtime error: name is not bound");
        }

        #[test]
        fn declare_function_call_same_line() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(&mut interpreter, "function Foo() : Int { 2 }; Foo()");
            is_only_value(&result, &output, &Value::Int(2));
        }

        #[test]
        fn let_binding_function_declaration_call_same_line() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(
                &mut interpreter,
                "let x = 1; function Foo() : Int { 2 }; Foo() + 1",
            );
            is_only_value(&result, &output, &Value::Int(3));
        }

        #[test]
        fn nested_function() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(
                &mut interpreter,
                "function Foo() : Int { function Bar() : Int { 1 }; Bar() + 1 }; Foo() + 1",
            );
            is_only_value(&result, &output, &Value::Int(3));
        }

        #[test]
        fn open_namespace() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(&mut interpreter, "open Microsoft.Quantum.Diagnostics;");
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "DumpMachine()");
            is_unit_with_output(&result, &output, "STATE:\n|0⟩: 1+0i");
        }

        #[test]
        fn open_namespace_call_same_line() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(
                &mut interpreter,
                "open Microsoft.Quantum.Diagnostics; DumpMachine()",
            );
            is_unit_with_output(&result, &output, "STATE:\n|0⟩: 1+0i");
        }

        #[test]
        fn declare_namespace_call() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(
                &mut interpreter,
                "namespace Foo { function Bar() : Int { 5 } }",
            );
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "Foo.Bar()");
            is_only_value(&result, &output, &Value::Int(5));
        }

        #[test]
        fn declare_namespace_open_call() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(
                &mut interpreter,
                "namespace Foo { function Bar() : Int { 5 } }",
            );
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "open Foo;");
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "Bar()");
            is_only_value(&result, &output, &Value::Int(5));
        }

        #[test]
        fn declare_namespace_open_call_same_line() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(
                &mut interpreter,
                "namespace Foo { function Bar() : Int { 5 } } open Foo; Bar()",
            );
            is_only_value(&result, &output, &Value::Int(5));
        }

        #[test]
        fn mix_stmts_and_namespace_same_line() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(
                &mut interpreter,
                "Message(\"before\"); namespace Foo { function Bar() : Int { 5 } } Message(\"after\")",
            );
            is_unit_with_output(&result, &output, "before\nafter");
        }

        #[test]
        fn global_qubits() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(&mut interpreter, "open Microsoft.Quantum.Diagnostics;");
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "DumpMachine()");
            is_unit_with_output(&result, &output, "STATE:\n|0⟩: 1+0i");
            let (result, output) = line(&mut interpreter, "use (q0, qs) = (Qubit(), Qubit[3]);");
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "DumpMachine()");
            is_unit_with_output(&result, &output, "STATE:\n|0000⟩: 1+0i");
            let (result, output) = line(&mut interpreter, "X(q0); X(qs[1]);");
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "DumpMachine()");
            is_unit_with_output(&result, &output, "STATE:\n|0101⟩: 1+0i");
        }

        #[test]
        fn ambiguous_type_error_in_top_level_stmts() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(&mut interpreter, "let x = [];");
            is_only_error(
                &result,
                &output,
                "type error: insufficient type information to infer type",
            );
            let (result, output) = line(&mut interpreter, "let x = []; let y = [0] + x;");
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "function Foo() : Unit { let x = []; }");
            is_only_error(
                &result,
                &output,
                "type error: insufficient type information to infer type",
            );
        }

        #[test]
        fn resolved_type_persists_across_stmts() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(&mut interpreter, "let x = []; let y = [0] + x;");
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "let z = [0.0] + x;");
            is_only_error(&result, &output, "type error: expected Double, found Int");
        }

        #[test]
        fn incremental_lambas_work() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(&mut interpreter, "let x = 1; let f = (y) -> x + y;");
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "f(1)");
            is_only_value(&result, &output, &Value::Int(2));
        }

        #[test]
        fn mutability_persists_across_stmts() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(
                &mut interpreter,
                "mutable x : Int[] = []; let y : Int[] = [];",
            );
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "set x += [0];");
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "set y += [0];");
            is_only_error(&result, &output, "cannot update immutable variable");
            let (result, output) = line(&mut interpreter, "let lam = () -> y + [0];");
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "let lam = () -> x + [0];");
            is_only_error(
                &result,
                &output,
                "lambdas cannot close over mutable variables",
            );
        }

        #[test]
        fn items_usable_before_definition() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(
                &mut interpreter,
                indoc! {r#"
                    function A() : Unit {
                        B();
                    }
                    function B() : Unit {}
                    A()
                "#},
            );
            is_only_value(&result, &output, &Value::unit());
        }

        #[test]
        fn namespace_usable_before_definition() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(
                &mut interpreter,
                indoc! {r#"
                    A.B();
                    namespace A {
                        function B() : Unit {}
                    }
                "#},
            );
            is_only_value(&result, &output, &Value::unit());
        }

        #[test]
        fn mutually_recursive_namespaces_work() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(
                &mut interpreter,
                indoc! {r#"
                    A.B();
                    namespace A {
                        open C;
                        function B() : Unit {
                            D();
                        }
                        function E() : Unit {}
                    }
                    namespace C {
                        open A;
                        function D() : Unit {
                            E();
                        }
                    }
                "#},
            );
            is_only_value(&result, &output, &Value::unit());
        }

        #[test]
        fn normal_qirgen() {
            let mut interpreter = Interpreter::new(
                true,
                SourceMap::default(),
                PackageType::Lib,
                TargetProfile::Base,
            )
            .expect("interpreter should be created");
            let (result, output) = line(
                &mut interpreter,
                indoc! {"operation Foo() : Result { use q = Qubit(); let r = M(q); Reset(q); return r; } "},
            );
            is_only_value(&result, &output, &Value::unit());
            let res = interpreter.qirgen("Foo()").expect("expected success");
            expect![[r#"
                %Result = type opaque
                %Qubit = type opaque

                define void @ENTRYPOINT__main() #0 {
                  call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
                  call void @__quantum__qis__reset__body(%Qubit* inttoptr (i64 0 to %Qubit*))
                  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
                  ret void
                }

                declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)
                declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__cy__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__rx__body(double, %Qubit*)
                declare void @__quantum__qis__rxx__body(double, %Qubit*, %Qubit*)
                declare void @__quantum__qis__ry__body(double, %Qubit*)
                declare void @__quantum__qis__ryy__body(double, %Qubit*, %Qubit*)
                declare void @__quantum__qis__rz__body(double, %Qubit*)
                declare void @__quantum__qis__rzz__body(double, %Qubit*, %Qubit*)
                declare void @__quantum__qis__h__body(%Qubit*)
                declare void @__quantum__qis__s__body(%Qubit*)
                declare void @__quantum__qis__s__adj(%Qubit*)
                declare void @__quantum__qis__t__body(%Qubit*)
                declare void @__quantum__qis__t__adj(%Qubit*)
                declare void @__quantum__qis__x__body(%Qubit*)
                declare void @__quantum__qis__y__body(%Qubit*)
                declare void @__quantum__qis__z__body(%Qubit*)
                declare void @__quantum__qis__swap__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__reset__body(%Qubit*)
                declare void @__quantum__qis__mresetz__body(%Qubit*, %Result* writeonly) #1
                declare void @__quantum__qis__m__body(%Qubit*, %Result* writeonly) #1
                declare void @__quantum__rt__result_record_output(%Result*, i8*)
                declare void @__quantum__rt__array_record_output(i64, i8*)
                declare void @__quantum__rt__tuple_record_output(i64, i8*)

                attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" }
                attributes #1 = { "irreversible" }

                ; module flags

                !llvm.module.flags = !{!0, !1, !2, !3}

                !0 = !{i32 1, !"qir_major_version", i32 1}
                !1 = !{i32 7, !"qir_minor_version", i32 0}
                !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
                !3 = !{i32 1, !"dynamic_result_management", i1 false}
            "#]].assert_eq(&res);
        }

        #[test]
        fn qirgen_entry_expr_in_block() {
            let mut interpreter = Interpreter::new(
                true,
                SourceMap::default(),
                PackageType::Lib,
                TargetProfile::Base,
            )
            .expect("interpreter should be created");
            let (result, output) = line(
                &mut interpreter,
                indoc! {"operation Foo() : Result { use q = Qubit(); let r = M(q); Reset(q); return r; } "},
            );
            is_only_value(&result, &output, &Value::unit());
            let res = interpreter.qirgen("{Foo()}").expect("expected success");
            expect![[r#"
                %Result = type opaque
                %Qubit = type opaque

                define void @ENTRYPOINT__main() #0 {
                  call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
                  call void @__quantum__qis__reset__body(%Qubit* inttoptr (i64 0 to %Qubit*))
                  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
                  ret void
                }

                declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)
                declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__cy__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__rx__body(double, %Qubit*)
                declare void @__quantum__qis__rxx__body(double, %Qubit*, %Qubit*)
                declare void @__quantum__qis__ry__body(double, %Qubit*)
                declare void @__quantum__qis__ryy__body(double, %Qubit*, %Qubit*)
                declare void @__quantum__qis__rz__body(double, %Qubit*)
                declare void @__quantum__qis__rzz__body(double, %Qubit*, %Qubit*)
                declare void @__quantum__qis__h__body(%Qubit*)
                declare void @__quantum__qis__s__body(%Qubit*)
                declare void @__quantum__qis__s__adj(%Qubit*)
                declare void @__quantum__qis__t__body(%Qubit*)
                declare void @__quantum__qis__t__adj(%Qubit*)
                declare void @__quantum__qis__x__body(%Qubit*)
                declare void @__quantum__qis__y__body(%Qubit*)
                declare void @__quantum__qis__z__body(%Qubit*)
                declare void @__quantum__qis__swap__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__reset__body(%Qubit*)
                declare void @__quantum__qis__mresetz__body(%Qubit*, %Result* writeonly) #1
                declare void @__quantum__qis__m__body(%Qubit*, %Result* writeonly) #1
                declare void @__quantum__rt__result_record_output(%Result*, i8*)
                declare void @__quantum__rt__array_record_output(i64, i8*)
                declare void @__quantum__rt__tuple_record_output(i64, i8*)

                attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" }
                attributes #1 = { "irreversible" }

                ; module flags

                !llvm.module.flags = !{!0, !1, !2, !3}

                !0 = !{i32 1, !"qir_major_version", i32 1}
                !1 = !{i32 7, !"qir_minor_version", i32 0}
                !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
                !3 = !{i32 1, !"dynamic_result_management", i1 false}
            "#]].assert_eq(&res);
        }

        #[test]
        fn qirgen_entry_expr_defines_operation() {
            let mut interpreter = Interpreter::new(
                true,
                SourceMap::default(),
                PackageType::Lib,
                TargetProfile::Base,
            )
            .expect("interpreter should be created");
            let (result, output) = line(
                &mut interpreter,
                indoc! {"operation Foo() : Result { use q = Qubit(); let r = M(q); Reset(q); return r; } "},
            );
            is_only_value(&result, &output, &Value::unit());
            let res = interpreter
                .qirgen("{operation Bar() : Unit {}; Foo()}")
                .expect("expected success");
            expect![[r#"
                %Result = type opaque
                %Qubit = type opaque

                define void @ENTRYPOINT__main() #0 {
                  call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
                  call void @__quantum__qis__reset__body(%Qubit* inttoptr (i64 0 to %Qubit*))
                  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
                  ret void
                }

                declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)
                declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__cy__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__rx__body(double, %Qubit*)
                declare void @__quantum__qis__rxx__body(double, %Qubit*, %Qubit*)
                declare void @__quantum__qis__ry__body(double, %Qubit*)
                declare void @__quantum__qis__ryy__body(double, %Qubit*, %Qubit*)
                declare void @__quantum__qis__rz__body(double, %Qubit*)
                declare void @__quantum__qis__rzz__body(double, %Qubit*, %Qubit*)
                declare void @__quantum__qis__h__body(%Qubit*)
                declare void @__quantum__qis__s__body(%Qubit*)
                declare void @__quantum__qis__s__adj(%Qubit*)
                declare void @__quantum__qis__t__body(%Qubit*)
                declare void @__quantum__qis__t__adj(%Qubit*)
                declare void @__quantum__qis__x__body(%Qubit*)
                declare void @__quantum__qis__y__body(%Qubit*)
                declare void @__quantum__qis__z__body(%Qubit*)
                declare void @__quantum__qis__swap__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__reset__body(%Qubit*)
                declare void @__quantum__qis__mresetz__body(%Qubit*, %Result* writeonly) #1
                declare void @__quantum__qis__m__body(%Qubit*, %Result* writeonly) #1
                declare void @__quantum__rt__result_record_output(%Result*, i8*)
                declare void @__quantum__rt__array_record_output(i64, i8*)
                declare void @__quantum__rt__tuple_record_output(i64, i8*)

                attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" }
                attributes #1 = { "irreversible" }

                ; module flags

                !llvm.module.flags = !{!0, !1, !2, !3}

                !0 = !{i32 1, !"qir_major_version", i32 1}
                !1 = !{i32 7, !"qir_minor_version", i32 0}
                !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
                !3 = !{i32 1, !"dynamic_result_management", i1 false}
            "#]].assert_eq(&res);

            // Operation should not be visible from global scope
            let (result, output) = line(&mut interpreter, indoc! {"Bar()"});
            is_only_error(&result, &output, "name error: `Bar` not found");
        }

        #[test]
        fn qirgen_multiple_exprs_parse_fail() {
            let mut interpreter = Interpreter::new(
                true,
                SourceMap::default(),
                PackageType::Lib,
                TargetProfile::Base,
            )
            .expect("interpreter should be created");
            let (result, output) = line(
                &mut interpreter,
                indoc! {"operation Foo() : Result { use q = Qubit(); let r = M(q); Reset(q); return r; } "},
            );
            is_only_value(&result, &output, &Value::unit());
            let res = interpreter
                .qirgen("Foo(); operation Bar() : Unit {}; Foo()")
                .expect_err("expected error");
            expect![[r#"
                [
                    LineError(
                        WithSource {
                            source: Some(
                                "Foo(); operation Bar() : Unit {}; Foo()",
                            ),
                            error: Compile(
                                Error(
                                    Parse(
                                        Error(
                                            Token(
                                                Eof,
                                                Semi,
                                                Span {
                                                    lo: 5,
                                                    hi: 6,
                                                },
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                            stack_trace: None,
                        },
                    ),
                ]
            "#]]
            .assert_debug_eq(&res);
        }

        #[test]
        fn qirgen_entry_expr_defines_operation_then_more_operations() {
            let mut interpreter = Interpreter::new(
                true,
                SourceMap::default(),
                PackageType::Lib,
                TargetProfile::Base,
            )
            .expect("interpreter should be created");
            let (result, output) = line(
                &mut interpreter,
                indoc! {"operation Foo() : Result { use q = Qubit(); let r = M(q); Reset(q); return r; } "},
            );
            is_only_value(&result, &output, &Value::unit());
            let res = interpreter
                .qirgen("{operation Bar() : Unit {}; Foo()}")
                .expect("expected success");
            expect![[r#"
                %Result = type opaque
                %Qubit = type opaque

                define void @ENTRYPOINT__main() #0 {
                  call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
                  call void @__quantum__qis__reset__body(%Qubit* inttoptr (i64 0 to %Qubit*))
                  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
                  ret void
                }

                declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)
                declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__cy__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__rx__body(double, %Qubit*)
                declare void @__quantum__qis__rxx__body(double, %Qubit*, %Qubit*)
                declare void @__quantum__qis__ry__body(double, %Qubit*)
                declare void @__quantum__qis__ryy__body(double, %Qubit*, %Qubit*)
                declare void @__quantum__qis__rz__body(double, %Qubit*)
                declare void @__quantum__qis__rzz__body(double, %Qubit*, %Qubit*)
                declare void @__quantum__qis__h__body(%Qubit*)
                declare void @__quantum__qis__s__body(%Qubit*)
                declare void @__quantum__qis__s__adj(%Qubit*)
                declare void @__quantum__qis__t__body(%Qubit*)
                declare void @__quantum__qis__t__adj(%Qubit*)
                declare void @__quantum__qis__x__body(%Qubit*)
                declare void @__quantum__qis__y__body(%Qubit*)
                declare void @__quantum__qis__z__body(%Qubit*)
                declare void @__quantum__qis__swap__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__reset__body(%Qubit*)
                declare void @__quantum__qis__mresetz__body(%Qubit*, %Result* writeonly) #1
                declare void @__quantum__qis__m__body(%Qubit*, %Result* writeonly) #1
                declare void @__quantum__rt__result_record_output(%Result*, i8*)
                declare void @__quantum__rt__array_record_output(i64, i8*)
                declare void @__quantum__rt__tuple_record_output(i64, i8*)

                attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" }
                attributes #1 = { "irreversible" }

                ; module flags

                !llvm.module.flags = !{!0, !1, !2, !3}

                !0 = !{i32 1, !"qir_major_version", i32 1}
                !1 = !{i32 7, !"qir_minor_version", i32 0}
                !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
                !3 = !{i32 1, !"dynamic_result_management", i1 false}
            "#]].assert_eq(&res);

            // Can't validate here in this test, but note that the following call will cause
            // `lower_fragments` (https://github.com/microsoft/qsharp/blob/e0c4dd334a81a0b0be82f4fbff0d850acc0a5fc8/compiler/qsc_frontend/src/incremental.rs#L235)
            // to yield items that were previously declared in the qirgen call.
            // this is because `lowerer.drain_items` is being called (it wasn't called
            // previously as part of `qirgen()`).
            let (result, output) = line(
                &mut interpreter,
                indoc! {"operation Baz() : Result { use q = Qubit(); let r = M(q); Reset(q); return r; } "},
            );
            is_only_value(&result, &output, &Value::unit());

            let (result, output) = line(&mut interpreter, indoc! {"Bar()"});
            is_only_error(&result, &output, "name error: `Bar` not found");
        }

        #[test]
        fn qirgen_define_operation_use_it() {
            let mut interpreter = Interpreter::new(
                true,
                SourceMap::default(),
                PackageType::Lib,
                TargetProfile::Base,
            )
            .expect("interpreter should be created");
            // this panics today. It shouldn't
            let res = interpreter
                .qirgen("{ operation Foo() : Result { use q = Qubit(); let r = M(q); Reset(q); return r; }; Foo() }")
                .expect("expected success");
            expect![[r#"
                %Result = type opaque
                %Qubit = type opaque

                define void @ENTRYPOINT__main() #0 {
                  call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
                  call void @__quantum__qis__reset__body(%Qubit* inttoptr (i64 0 to %Qubit*))
                  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
                  ret void
                }

                declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)
                declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__cy__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__rx__body(double, %Qubit*)
                declare void @__quantum__qis__rxx__body(double, %Qubit*, %Qubit*)
                declare void @__quantum__qis__ry__body(double, %Qubit*)
                declare void @__quantum__qis__ryy__body(double, %Qubit*, %Qubit*)
                declare void @__quantum__qis__rz__body(double, %Qubit*)
                declare void @__quantum__qis__rzz__body(double, %Qubit*, %Qubit*)
                declare void @__quantum__qis__h__body(%Qubit*)
                declare void @__quantum__qis__s__body(%Qubit*)
                declare void @__quantum__qis__s__adj(%Qubit*)
                declare void @__quantum__qis__t__body(%Qubit*)
                declare void @__quantum__qis__t__adj(%Qubit*)
                declare void @__quantum__qis__x__body(%Qubit*)
                declare void @__quantum__qis__y__body(%Qubit*)
                declare void @__quantum__qis__z__body(%Qubit*)
                declare void @__quantum__qis__swap__body(%Qubit*, %Qubit*)
                declare void @__quantum__qis__reset__body(%Qubit*)
                declare void @__quantum__qis__mresetz__body(%Qubit*, %Result* writeonly) #1
                declare void @__quantum__qis__m__body(%Qubit*, %Result* writeonly) #1
                declare void @__quantum__rt__result_record_output(%Result*, i8*)
                declare void @__quantum__rt__array_record_output(i64, i8*)
                declare void @__quantum__rt__tuple_record_output(i64, i8*)

                attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" }
                attributes #1 = { "irreversible" }

                ; module flags

                !llvm.module.flags = !{!0, !1, !2, !3}

                !0 = !{i32 1, !"qir_major_version", i32 1}
                !1 = !{i32 7, !"qir_minor_version", i32 0}
                !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
                !3 = !{i32 1, !"dynamic_result_management", i1 false}
            "#]].assert_eq(&res);
        }
    }

    #[cfg(test)]
    mod with_sources {
        use super::*;
        use indoc::indoc;
        use qsc_frontend::compile::{SourceMap, TargetProfile};
        use qsc_passes::PackageType;

        #[test]
        fn entry_expr_is_executed() {
            let source = indoc! { r#"
            namespace Test {
                @EntryPoint()
                operation Main() : Unit {
                    Message("hello there...")
                }
            }"#};

            let sources = SourceMap::new([("test".into(), source.into())], None);
            let mut interpreter =
                Interpreter::new(true, sources, PackageType::Exe, TargetProfile::Full)
                    .expect("interpreter should be created");

            let (result, output) = entry(&mut interpreter);
            is_unit_with_output_eval_entry(&result, &output, "hello there...");
        }

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
                Interpreter::new(true, sources, PackageType::Lib, TargetProfile::Full)
                    .expect("interpreter should be created");

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
                Interpreter::new(true, sources, PackageType::Lib, TargetProfile::Full)
                    .expect("interpreter should be created");

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
                Interpreter::new(true, sources, PackageType::Lib, TargetProfile::Full)
                    .expect("interpreter should be created");
            let (result, output) = line(&mut interpreter, "Test.Hello()");
            is_only_value(&result, &output, &Value::String("hello there...".into()));
            let (result, output) = line(&mut interpreter, "Test2.Main()");
            is_only_value(&result, &output, &Value::String("hello there...".into()));
        }
    }

    fn get_interpreter() -> Interpreter {
        Interpreter::new(
            true,
            SourceMap::default(),
            PackageType::Lib,
            TargetProfile::Full,
        )
        .expect("interpreter should be created")
    }

    fn is_only_value(result: &Result<Value, Vec<LineError>>, output: &str, value: &Value) {
        assert_eq!("", output);

        match result {
            Ok(v) => assert_eq!(value, v),
            Err(e) => panic!("Expected unit value, got {e:?}"),
        }
    }

    fn is_unit_with_output_eval_entry(
        result: &Result<Value, Vec<crate::interpret::stateful::Error>>,
        output: &str,
        expected_output: &str,
    ) {
        assert_eq!(expected_output, output);

        match result {
            Ok(value) => assert_eq!(Value::unit(), *value),
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
