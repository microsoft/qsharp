// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod given_interpreter {
    use crate::interpret::stateful::{Error, InterpretResult, Interpreter};
    use expect_test::Expect;
    use miette::Diagnostic;
    use qsc_eval::{output::CursorReceiver, val::Value};
    use qsc_frontend::compile::{SourceMap, TargetProfile};
    use qsc_passes::PackageType;
    use std::{fmt::Write, io::Cursor, iter, str::from_utf8};

    fn line(interpreter: &mut Interpreter, line: &str) -> (InterpretResult, String) {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        let mut receiver = CursorReceiver::new(&mut cursor);
        (
            interpreter.eval_fragments(&mut receiver, line),
            receiver.dump(),
        )
    }

    fn run(
        interpreter: &mut Interpreter,
        expr: &str,
        shots: u32,
    ) -> (Result<Vec<InterpretResult>, Vec<Error>>, String) {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        let mut receiver = CursorReceiver::new(&mut cursor);
        (interpreter.run(&mut receiver, expr, shots), receiver.dump())
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
                is_only_error(
                    &result,
                    &output,
                    &expect![[r#"
                        name error: `Message` not found
                           [line_0] [Message]
                        type error: insufficient type information to infer type
                           [line_0] [Message("_")]
                    "#]],
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
            is_only_error(
                &result,
                &output,
                &expect![[r#"
                    syntax error: expected `;`, found EOF
                       [line_0] []
                "#]],
            );

            let (result, output) = line(&mut interpreter, "y");
            is_only_error(
                &result,
                &output,
                &expect![[r#"
                    name error: `y` not found
                       [line_1] [y]
                "#]],
            );
        }

        #[test]
        fn invalid_statements_and_unbound_vars_return_error() {
            let mut interpreter = get_interpreter();

            let (result, output) = line(&mut interpreter, "let y = x;");
            is_only_error(
                &result,
                &output,
                &expect![[r#"
                    name error: `x` not found
                       [line_0] [x]
                    type error: insufficient type information to infer type
                       [line_0] [y]
                "#]],
            );

            let (result, output) = line(&mut interpreter, "y");
            is_only_error(
                &result,
                &output,
                &expect![[r#"
                    runtime error: name is not bound
                       [line_1] [y]
                "#]],
            );
        }

        #[test]
        fn failing_statements_return_early_error() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(&mut interpreter, "let y = 7;y/0;y");
            is_only_error(
                &result,
                &output,
                &expect![[r#"
                    runtime error: division by zero
                      cannot divide by zero [line_0] [0]
                "#]],
            );
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
            is_only_error(
                &result,
                &output,
                &expect![[r#"
                    name error: `invalid` not found
                       [line_0] [invalid]
                "#]],
            );
            let (result, output) = line(&mut interpreter, "Foo()");
            is_only_error(
                &result,
                &output,
                &expect![[r#"
                    runtime error: name is not bound
                       [line_1] [Foo]
                "#]],
            );
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
                &expect![[r#"
                    type error: insufficient type information to infer type
                       [line_0] [[]]
                "#]],
            );
            let (result, output) = line(&mut interpreter, "let x = []; let y = [0] + x;");
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "function Foo() : Unit { let x = []; }");
            is_only_error(
                &result,
                &output,
                &expect![[r#"
                    type error: insufficient type information to infer type
                       [line_2] [[]]
                "#]],
            );
        }

        #[test]
        fn resolved_type_persists_across_stmts() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(&mut interpreter, "let x = []; let y = [0] + x;");
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "let z = [0.0] + x;");
            is_only_error(
                &result,
                &output,
                &expect![[r#"
                    type error: expected Double, found Int
                       [line_1] [[0.0] + x]
                "#]],
            );
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
            is_only_error(
                &result,
                &output,
                &expect![[r#"
                    cannot update immutable variable
                       [line_2] [y]
                "#]],
            );
            let (result, output) = line(&mut interpreter, "let lam = () -> y + [0];");
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "let lam = () -> x + [0];");
            is_only_error(
                &result,
                &output,
                &expect![[r#"
                    lambdas cannot close over mutable variables
                       [line_4] [() -> x + [0]]
                "#]],
            );
        }

        #[test]
        fn runtime_error_across_lines() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(
                &mut interpreter,
                "operation Main() : Unit { Microsoft.Quantum.Random.DrawRandomInt(2,1); }",
            );
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "Main()");
            is_only_error(
                &result,
                &output,
                &expect![[r#"
                    runtime error: empty range
                      the range cannot be empty [line_0] [(2,1)]
                "#]],
            );
        }

        #[test]
        fn compiler_error_across_lines() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(
                &mut interpreter,
                "namespace Other { operation DumpMachine() : Unit { } }",
            );
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "open Other;");
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "open Microsoft.Quantum.Diagnostics;");
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "DumpMachine();");
            is_only_error(
                &result,
                &output,
                &expect![[r#"
                    name error: `DumpMachine` could refer to the item in `Other` or `Microsoft.Quantum.Diagnostics`
                      ambiguous name [line_3] [DumpMachine]
                      found in this namespace [line_1] [Other]
                      and also in this namespace [line_2] [Microsoft.Quantum.Diagnostics]
                    type error: insufficient type information to infer type
                       [line_3] [DumpMachine()]
                "#]],
            );
        }

        #[test]
        fn runtime_error_from_stdlib() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(&mut interpreter, "use q = Qubit(); CNOT(q,q)");
            is_only_error(
                &result,
                &output,
                &expect![[r#"
                    runtime error: qubits in gate invocation are not unique
                       [intrinsic.qs] [(control, target)]
                "#]],
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
        fn items_usable_before_definition_top_level() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(
                &mut interpreter,
                indoc! {r#"
                    B();
                    function B() : Unit {}
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
                  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
                  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
                  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
                  call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
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
                declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1
                declare void @__quantum__rt__result_record_output(%Result*, i8*)
                declare void @__quantum__rt__array_record_output(i64, i8*)
                declare void @__quantum__rt__tuple_record_output(i64, i8*)

                attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="1" }
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
                  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
                  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
                  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
                  call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
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
                declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1
                declare void @__quantum__rt__result_record_output(%Result*, i8*)
                declare void @__quantum__rt__array_record_output(i64, i8*)
                declare void @__quantum__rt__tuple_record_output(i64, i8*)

                attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="1" }
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
                  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
                  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
                  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
                  call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
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
                declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1
                declare void @__quantum__rt__result_record_output(%Result*, i8*)
                declare void @__quantum__rt__array_record_output(i64, i8*)
                declare void @__quantum__rt__tuple_record_output(i64, i8*)

                attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="1" }
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
            is_only_error(
                &result,
                &output,
                &expect![[r#"
                    name error: `Bar` not found
                       [line_1] [Bar]
                    type error: insufficient type information to infer type
                       [line_1] [Bar()]
                "#]],
            );
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
            is_error(
                &res,
                &expect![[r#"
                syntax error: expected EOF, found `;`
                   [<entry>] [;]
            "#]],
            );
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
                  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
                  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
                  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
                  call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
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
                declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1
                declare void @__quantum__rt__result_record_output(%Result*, i8*)
                declare void @__quantum__rt__array_record_output(i64, i8*)
                declare void @__quantum__rt__tuple_record_output(i64, i8*)

                attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="1" }
                attributes #1 = { "irreversible" }

                ; module flags

                !llvm.module.flags = !{!0, !1, !2, !3}

                !0 = !{i32 1, !"qir_major_version", i32 1}
                !1 = !{i32 7, !"qir_minor_version", i32 0}
                !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
                !3 = !{i32 1, !"dynamic_result_management", i1 false}
            "#]].assert_eq(&res);

            let (result, output) = line(
                &mut interpreter,
                indoc! {"operation Baz() : Result { use q = Qubit(); let r = M(q); Reset(q); return r; } "},
            );
            is_only_value(&result, &output, &Value::unit());

            let (result, output) = line(&mut interpreter, indoc! {"Bar()"});
            is_only_error(
                &result,
                &output,
                &expect![[r#"
                    name error: `Bar` not found
                       [line_2] [Bar]
                    type error: insufficient type information to infer type
                       [line_2] [Bar()]
                "#]],
            );
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
            let res = interpreter
                .qirgen("{ operation Foo() : Result { use q = Qubit(); let r = M(q); Reset(q); return r; }; Foo() }")
                .expect("expected success");
            expect![[r#"
                %Result = type opaque
                %Qubit = type opaque

                define void @ENTRYPOINT__main() #0 {
                  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
                  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
                  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
                  call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
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
                declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1
                declare void @__quantum__rt__result_record_output(%Result*, i8*)
                declare void @__quantum__rt__array_record_output(i64, i8*)
                declare void @__quantum__rt__tuple_record_output(i64, i8*)

                attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="1" }
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
        fn qirgen_entry_expr_profile_incompatible() {
            let mut interpreter = Interpreter::new(
                true,
                SourceMap::default(),
                PackageType::Lib,
                TargetProfile::Base,
            )
            .expect("interpreter should be created");
            let res = interpreter
                .qirgen("1")
                .expect_err("expected qirgen to fail");
            is_error(
                &res,
                &expect![[r#"
                non-Result return type in entry expression
                   [<entry>] [1]
            "#]],
            );
        }

        #[test]
        fn run_with_shots() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(&mut interpreter, "operation Foo() : Int { 1 }");
            is_only_value(&result, &output, &Value::unit());
            let (results, output) = run(&mut interpreter, "Foo()", 5);
            assert_eq!(output, String::new());
            let results = results.expect("run() should succeed");
            assert_eq!(results.len(), 5);
            for r in results {
                let val = r.expect("individual run should succeed");
                assert_eq!(val, Value::Int(1));
            }
        }

        #[test]
        fn run_parse_error() {
            let mut interpreter = get_interpreter();
            let (results, _) = run(&mut interpreter, "Foo)", 5);
            results.expect_err("run() should fail");
        }

        #[test]
        fn run_compile_error() {
            let mut interpreter = get_interpreter();
            let (results, _) = run(&mut interpreter, "Foo()", 5);
            results.expect_err("run() should fail");
        }

        #[test]
        fn run_multiple_statements() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(&mut interpreter, "operation Foo() : Int { 1 }");
            is_only_value(&result, &output, &Value::unit());
            let (result, output) = line(&mut interpreter, "operation Bar() : Int { 2 }");
            is_only_value(&result, &output, &Value::unit());
            let (results, output) = run(&mut interpreter, "{ Foo(); Bar() }", 5);
            assert_eq!(output, String::new());
            let results = results.expect("run() should succeed");
            assert_eq!(results.len(), 5);
            for r in results {
                let val = r.expect("individual run should succeed");
                assert_eq!(val, Value::Int(2));
            }
        }

        #[test]
        fn run_runtime_failure() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(
                &mut interpreter,
                r#"operation Foo() : Int { fail "failed" }"#,
            );
            is_only_value(&result, &output, &Value::unit());
            let (results, output) = run(&mut interpreter, "Foo()", 5);
            assert_eq!(output, String::new());
            let results = results.expect("run() should succeed");
            assert_eq!(results.len(), 5);
            for r in results {
                r.expect_err("individual run should fail");
            }
        }

        #[test]
        fn run_output_merged() {
            let mut interpreter = get_interpreter();
            let (result, output) = line(
                &mut interpreter,
                r#"operation Foo() : Unit { Message("hello!") }"#,
            );
            is_only_value(&result, &output, &Value::unit());
            let (results, output) = run(&mut interpreter, "Foo()", 5);
            expect![[r#"
                hello!
                hello!
                hello!
                hello!
                hello!"#]]
            .assert_eq(&output);
            let results = results.expect("run() should succeed");
            assert_eq!(results.len(), 5);
            for r in results {
                let val = r.expect("individual run should succeed");
                assert_eq!(val, Value::unit());
            }
        }
    }

    #[cfg(test)]
    mod with_sources {
        use super::*;
        use expect_test::expect;
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

        #[test]
        fn runtime_error_from_stdlib() {
            let sources = SourceMap::new(
                [(
                    "test".into(),
                    "namespace Foo {
                        operation Bar(): Unit {
                            let x = -1;
                            use qs = Qubit[x];
                        }
                    }
                    "
                    .into(),
                )],
                Some("Foo.Bar()".into()),
            );

            let mut interpreter =
                Interpreter::new(true, sources, PackageType::Lib, TargetProfile::Full)
                    .expect("interpreter should be created");
            let (result, output) = entry(&mut interpreter);
            is_only_error(
                &result,
                &output,
                &expect![[r#"
                    runtime error: program failed: Cannot allocate qubit array with a negative length
                      explicit fail [core/qir.qs] [fail "Cannot allocate qubit array with a negative length"]
                "#]],
            );
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

    fn is_only_value(result: &InterpretResult, output: &str, value: &Value) {
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

    fn is_unit_with_output(result: &InterpretResult, output: &str, expected_output: &str) {
        assert_eq!(expected_output, output);

        match result {
            Ok(value) => assert_eq!(Value::unit(), *value),
            Err(e) => panic!("Expected unit value, got {e:?}"),
        }
    }

    fn is_only_error<E>(result: &Result<Value, Vec<E>>, output: &str, expected_errors: &Expect)
    where
        E: Diagnostic,
    {
        assert_eq!("", output);

        match result {
            Ok(value) => panic!("Expected error , got {value:?}"),
            Err(errors) => is_error(errors, expected_errors),
        }
    }

    fn is_error<E>(errors: &Vec<E>, expected_errors: &Expect)
    where
        E: Diagnostic,
    {
        let mut actual = String::new();
        for error in errors {
            write!(actual, "{error}").expect("writing should succeed");
            for s in iter::successors(error.source(), |&s| s.source()) {
                write!(actual, ": {s}").expect("writing should succeed");
            }
            for label in error.labels().into_iter().flatten() {
                let span = error
                    .source_code()
                    .expect("expected valid source code")
                    .read_span(label.inner(), 0, 0)
                    .expect("expected to be able to read span");

                write!(
                    actual,
                    "\n  {} [{}] [{}]",
                    label.label().unwrap_or(""),
                    span.name().expect("expected source file name"),
                    from_utf8(span.data()).expect("expected valid utf-8 string"),
                )
                .expect("writing should succeed");
            }
            writeln!(actual).expect("writing should succeed");
        }

        expected_errors.assert_eq(&actual);
    }
}
