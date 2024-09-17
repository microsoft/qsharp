// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;
use qsc_data_structures::{language_features::LanguageFeatures, target::TargetCapabilityFlags};
use qsc_frontend::compile::SourceMap;

use crate::codegen::qir::get_qir;

#[test]
fn code_with_errors_returns_errors() {
    let source = "namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit()
                let pi_over_two = 4.0 / 2.0;
            }
        }";
    let sources = SourceMap::new([("test.qs".into(), source.into())], None);
    let language_features = LanguageFeatures::default();
    let capabilities = TargetCapabilityFlags::empty();
    let (std_id, store) = crate::compile::package_store_with_stdlib(capabilities);

    expect![[r#"
        Err(
            [
                Compile(
                    WithSource {
                        sources: [
                            Source {
                                name: "test.qs",
                                contents: "namespace Test {\n            @EntryPoint()\n            operation Main() : Unit {\n                use q = Qubit()\n                let pi_over_two = 4.0 / 2.0;\n            }\n        }",
                                offset: 0,
                            },
                        ],
                        error: Frontend(
                            Error(
                                Parse(
                                    Error(
                                        Token(
                                            Semi,
                                            Keyword(
                                                Let,
                                            ),
                                            Span {
                                                lo: 129,
                                                hi: 132,
                                            },
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    },
                ),
            ],
        )
    "#]]
    .assert_debug_eq(&get_qir(sources, language_features, capabilities, store, &[(std_id, None)]));
}

mod base_profile {
    use expect_test::expect;
    use qsc_data_structures::{language_features::LanguageFeatures, target::TargetCapabilityFlags};
    use qsc_frontend::compile::SourceMap;

    use crate::codegen::qir::get_qir;

    #[test]
    fn simple() {
        let source = "namespace Test {
            import Std.Math.*;
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Result {
                use q = Qubit();
                let pi_over_two = 4.0 / 2.0;
                __quantum__qis__rz__body(pi_over_two, q);
                mutable some_angle = ArcSin(0.0);
                __quantum__qis__rz__body(some_angle, q);
                set some_angle = ArcCos(-1.0) / PI();
                __quantum__qis__rz__body(some_angle, q);
                __quantum__qis__mresetz__body(q)
            }
        }";
        let sources = SourceMap::new([("test.qs".into(), source.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::empty();

        let (std_id, store) = crate::compile::package_store_with_stdlib(capabilities);
        let qir = get_qir(
            sources,
            language_features,
            capabilities,
            store,
            &[(std_id, None)],
        )
        .expect("Failed to generate QIR");
        expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
            block_0:
              call void @__quantum__qis__rz__body(double 2.0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__rz__body(double 0.0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__rz__body(double 1.0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__rz__body(double, %Qubit*)

            declare void @__quantum__rt__result_record_output(%Result*, i8*)

            declare void @__quantum__qis__m__body(%Qubit*, %Result*) #1

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="1" "required_num_results"="1" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]]
        .assert_eq(&qir);
    }

    #[test]
    fn qubit_reuse_triggers_reindexing() {
        let source = "namespace Test {
            @EntryPoint()
            operation Main() : (Result, Result) {
                use q = Qubit();
                (MResetZ(q), MResetZ(q))
            }
        }";
        let sources = SourceMap::new([("test.qs".into(), source.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::empty();

        let (std_id, store) = crate::compile::package_store_with_stdlib(capabilities);
        let qir = get_qir(
            sources,
            language_features,
            capabilities,
            store,
            &[(std_id, None)],
        )
        .expect("Failed to generate QIR");
        expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
            block_0:
              call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
              call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
              call void @__quantum__rt__tuple_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__rt__tuple_record_output(i64, i8*)

            declare void @__quantum__rt__result_record_output(%Result*, i8*)

            declare void @__quantum__qis__m__body(%Qubit*, %Result*) #1

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]].assert_eq(&qir);
    }

    #[test]
    fn qubit_measurements_get_deferred() {
        let source = "namespace Test {
            @EntryPoint()
            operation Main() : Result[] {
                use (q0, q1) = (Qubit(), Qubit());
                X(q0);
                let r0 = MResetZ(q0);
                X(q1);
                let r1 = MResetZ(q1);
                [r0, r1]
            }
        }";
        let sources = SourceMap::new([("test.qs".into(), source.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::empty();

        let (std_id, store) = crate::compile::package_store_with_stdlib(capabilities);
        let qir = get_qir(
            sources,
            language_features,
            capabilities,
            store,
            &[(std_id, None)],
        )
        .expect("Failed to generate QIR");
        expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
            block_0:
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
              call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
              call void @__quantum__rt__array_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__x__body(%Qubit*)

            declare void @__quantum__rt__array_record_output(i64, i8*)

            declare void @__quantum__rt__result_record_output(%Result*, i8*)

            declare void @__quantum__qis__m__body(%Qubit*, %Result*) #1

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]].assert_eq(&qir);
    }

    #[test]
    fn qubit_id_swap_results_in_different_id_usage() {
        let source = "namespace Test {
            @EntryPoint()
            operation Main() : (Result, Result) {
                use (q0, q1) = (Qubit(), Qubit());
                X(q0);
                Relabel([q0, q1], [q1, q0]);
                X(q1);
                (MResetZ(q0), MResetZ(q1))
            }
        }";
        let sources = SourceMap::new([("test.qs".into(), source.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::empty();

        let (std_id, store) = crate::compile::package_store_with_stdlib(capabilities);
        let qir = get_qir(
            sources,
            language_features,
            capabilities,
            store,
            &[(std_id, None)],
        )
        .expect("Failed to generate QIR");
        expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
            block_0:
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
              call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
              call void @__quantum__rt__tuple_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__x__body(%Qubit*)

            declare void @__quantum__rt__tuple_record_output(i64, i8*)

            declare void @__quantum__rt__result_record_output(%Result*, i8*)

            declare void @__quantum__qis__m__body(%Qubit*, %Result*) #1

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]].assert_eq(&qir);
    }

    #[test]
    fn qubit_id_swap_across_reset_uses_updated_ids() {
        let source = "namespace Test {
            @EntryPoint()
            operation Main() : (Result, Result) {
                {
                    use (q0, q1) = (Qubit(), Qubit());
                    X(q0);
                    Relabel([q0, q1], [q1, q0]);
                    X(q1);
                    Reset(q0);
                    Reset(q1);
                }
                use (q0, q1) = (Qubit(), Qubit());
                (MResetZ(q0), MResetZ(q1))
            }
        }";
        let sources = SourceMap::new([("test.qs".into(), source.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::empty();

        let (std_id, store) = crate::compile::package_store_with_stdlib(capabilities);
        let qir = get_qir(
            sources,
            language_features,
            capabilities,
            store,
            &[(std_id, None)],
        )
        .expect("Failed to generate QIR");
        expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
            block_0:
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
              call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
              call void @__quantum__rt__tuple_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__x__body(%Qubit*)

            declare void @__quantum__rt__tuple_record_output(i64, i8*)

            declare void @__quantum__rt__result_record_output(%Result*, i8*)

            declare void @__quantum__qis__m__body(%Qubit*, %Result*) #1

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="4" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]].assert_eq(&qir);
    }
}

mod adaptive_profile {
    use expect_test::expect;
    use qsc_data_structures::{language_features::LanguageFeatures, target::TargetCapabilityFlags};
    use qsc_frontend::compile::SourceMap;

    use crate::codegen::qir::get_qir;

    #[test]
    fn simple() {
        let source = "namespace Test {
            import Std.Math.*;
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Result {
                use q = Qubit();
                let pi_over_two = 4.0 / 2.0;
                __quantum__qis__rz__body(pi_over_two, q);
                mutable some_angle = ArcSin(0.0);
                __quantum__qis__rz__body(some_angle, q);
                set some_angle = ArcCos(-1.0) / PI();
                __quantum__qis__rz__body(some_angle, q);
                __quantum__qis__mresetz__body(q)
            }
        }";
        let sources = SourceMap::new([("test.qs".into(), source.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::Adaptive;

        let (std_id, store) = crate::compile::package_store_with_stdlib(capabilities);
        let qir = get_qir(
            sources,
            language_features,
            capabilities,
            store,
            &[(std_id, None)],
        )
        .expect("Failed to generate QIR");
        expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
            block_0:
              call void @__quantum__qis__rz__body(double 2.0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__rz__body(double 0.0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__rz__body(double 1.0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__rz__body(double, %Qubit*)

            declare void @__quantum__rt__result_record_output(%Result*, i8*)

            declare void @__quantum__qis__m__body(%Qubit*, %Result*) #1

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="adaptive_profile" "required_num_qubits"="1" "required_num_results"="1" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3, !4, !5, !6, !7, !8, !9, !10}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
            !4 = !{i32 1, !"classical_ints", i1 false}
            !5 = !{i32 1, !"classical_floats", i1 false}
            !6 = !{i32 1, !"backwards_branching", i1 false}
            !7 = !{i32 1, !"qubit_resetting", i1 false}
            !8 = !{i32 1, !"classical_fixed_points", i1 false}
            !9 = !{i32 1, !"user_functions", i1 false}
            !10 = !{i32 1, !"multiple_target_branching", i1 false}
        "#]]
        .assert_eq(&qir);
    }

    #[test]
    fn qubit_reuse_triggers_reindexing() {
        let source = "namespace Test {
            @EntryPoint()
            operation Main() : (Result, Result) {
                use q = Qubit();
                (MResetZ(q), MResetZ(q))
            }
        }";
        let sources = SourceMap::new([("test.qs".into(), source.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::Adaptive;

        let (std_id, store) = crate::compile::package_store_with_stdlib(capabilities);
        let qir = get_qir(
            sources,
            language_features,
            capabilities,
            store,
            &[(std_id, None)],
        )
        .expect("Failed to generate QIR");
        expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
            block_0:
              call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
              call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
              call void @__quantum__rt__tuple_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__rt__tuple_record_output(i64, i8*)

            declare void @__quantum__rt__result_record_output(%Result*, i8*)

            declare void @__quantum__qis__m__body(%Qubit*, %Result*) #1

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="adaptive_profile" "required_num_qubits"="2" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3, !4, !5, !6, !7, !8, !9, !10}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
            !4 = !{i32 1, !"classical_ints", i1 false}
            !5 = !{i32 1, !"classical_floats", i1 false}
            !6 = !{i32 1, !"backwards_branching", i1 false}
            !7 = !{i32 1, !"qubit_resetting", i1 false}
            !8 = !{i32 1, !"classical_fixed_points", i1 false}
            !9 = !{i32 1, !"user_functions", i1 false}
            !10 = !{i32 1, !"multiple_target_branching", i1 false}
        "#]].assert_eq(&qir);
    }

    #[test]
    fn qubit_measurements_not_deferred() {
        let source = "namespace Test {
            @EntryPoint()
            operation Main() : Result[] {
                use (q0, q1) = (Qubit(), Qubit());
                X(q0);
                let r0 = MResetZ(q0);
                X(q1);
                let r1 = MResetZ(q1);
                [r0, r1]
            }
        }";
        let sources = SourceMap::new([("test.qs".into(), source.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::Adaptive;

        let (std_id, store) = crate::compile::package_store_with_stdlib(capabilities);
        let qir = get_qir(
            sources,
            language_features,
            capabilities,
            store,
            &[(std_id, None)],
        )
        .expect("Failed to generate QIR");
        expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
            block_0:
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
              call void @__quantum__rt__array_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__x__body(%Qubit*)

            declare void @__quantum__rt__array_record_output(i64, i8*)

            declare void @__quantum__rt__result_record_output(%Result*, i8*)

            declare void @__quantum__qis__m__body(%Qubit*, %Result*) #1

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="adaptive_profile" "required_num_qubits"="2" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3, !4, !5, !6, !7, !8, !9, !10}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
            !4 = !{i32 1, !"classical_ints", i1 false}
            !5 = !{i32 1, !"classical_floats", i1 false}
            !6 = !{i32 1, !"backwards_branching", i1 false}
            !7 = !{i32 1, !"qubit_resetting", i1 false}
            !8 = !{i32 1, !"classical_fixed_points", i1 false}
            !9 = !{i32 1, !"user_functions", i1 false}
            !10 = !{i32 1, !"multiple_target_branching", i1 false}
        "#]].assert_eq(&qir);
    }
}

mod adaptive_ri_profile {

    use expect_test::expect;
    use qsc_data_structures::{language_features::LanguageFeatures, target::TargetCapabilityFlags};
    use qsc_frontend::compile::SourceMap;

    use crate::codegen::qir::get_qir;

    #[test]
    fn simple() {
        let source = "namespace Test {
            import Std.Math.*;
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Result {
                use q = Qubit();
                let pi_over_two = 4.0 / 2.0;
                __quantum__qis__rz__body(pi_over_two, q);
                mutable some_angle = ArcSin(0.0);
                __quantum__qis__rz__body(some_angle, q);
                set some_angle = ArcCos(-1.0) / PI();
                __quantum__qis__rz__body(some_angle, q);
                __quantum__qis__mresetz__body(q)
            }
        }";
        let sources = SourceMap::new([("test.qs".into(), source.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::Adaptive
            | TargetCapabilityFlags::QubitReset
            | TargetCapabilityFlags::IntegerComputations;

        let (std_id, store) = crate::compile::package_store_with_stdlib(capabilities);
        let qir = get_qir(
            sources,
            language_features,
            capabilities,
            store,
            &[(std_id, None)],
        )
        .expect("Failed to generate QIR");
        expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
            block_0:
              call void @__quantum__qis__rz__body(double 2.0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__rz__body(double 0.0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__rz__body(double 1.0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__rz__body(double, %Qubit*)

            declare void @__quantum__qis__mresetz__body(%Qubit*, %Result*) #1

            declare void @__quantum__rt__result_record_output(%Result*, i8*)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="adaptive_profile" "required_num_qubits"="1" "required_num_results"="1" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3, !4, !5, !6, !7, !8, !9, !10}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
            !4 = !{i32 1, !"classical_ints", i1 true}
            !5 = !{i32 1, !"qubit_resetting", i1 true}
            !6 = !{i32 1, !"classical_floats", i1 false}
            !7 = !{i32 1, !"backwards_branching", i1 false}
            !8 = !{i32 1, !"classical_fixed_points", i1 false}
            !9 = !{i32 1, !"user_functions", i1 false}
            !10 = !{i32 1, !"multiple_target_branching", i1 false}
        "#]]
        .assert_eq(&qir);
    }

    #[test]
    fn qubit_reuse_allowed() {
        let source = "namespace Test {
            @EntryPoint()
            operation Main() : (Result, Result) {
                use q = Qubit();
                (MResetZ(q), MResetZ(q))
            }
        }";
        let sources = SourceMap::new([("test.qs".into(), source.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::Adaptive
            | TargetCapabilityFlags::QubitReset
            | TargetCapabilityFlags::IntegerComputations;

        let (std_id, store) = crate::compile::package_store_with_stdlib(capabilities);
        let qir = get_qir(
            sources,
            language_features,
            capabilities,
            store,
            &[(std_id, None)],
        )
        .expect("Failed to generate QIR");
        expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
            block_0:
              call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
              call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
              call void @__quantum__rt__tuple_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__mresetz__body(%Qubit*, %Result*) #1

            declare void @__quantum__rt__tuple_record_output(i64, i8*)

            declare void @__quantum__rt__result_record_output(%Result*, i8*)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="adaptive_profile" "required_num_qubits"="1" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3, !4, !5, !6, !7, !8, !9, !10}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
            !4 = !{i32 1, !"classical_ints", i1 true}
            !5 = !{i32 1, !"qubit_resetting", i1 true}
            !6 = !{i32 1, !"classical_floats", i1 false}
            !7 = !{i32 1, !"backwards_branching", i1 false}
            !8 = !{i32 1, !"classical_fixed_points", i1 false}
            !9 = !{i32 1, !"user_functions", i1 false}
            !10 = !{i32 1, !"multiple_target_branching", i1 false}
        "#]].assert_eq(&qir);
    }

    #[test]
    fn qubit_measurements_not_deferred() {
        let source = "namespace Test {
            @EntryPoint()
            operation Main() : Result[] {
                use (q0, q1) = (Qubit(), Qubit());
                X(q0);
                let r0 = MResetZ(q0);
                X(q1);
                let r1 = MResetZ(q1);
                [r0, r1]
            }
        }";
        let sources = SourceMap::new([("test.qs".into(), source.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::Adaptive
            | TargetCapabilityFlags::QubitReset
            | TargetCapabilityFlags::IntegerComputations;

        let (std_id, store) = crate::compile::package_store_with_stdlib(capabilities);
        let qir = get_qir(
            sources,
            language_features,
            capabilities,
            store,
            &[(std_id, None)],
        )
        .expect("Failed to generate QIR");
        expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
            block_0:
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
              call void @__quantum__rt__array_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__x__body(%Qubit*)

            declare void @__quantum__qis__mresetz__body(%Qubit*, %Result*) #1

            declare void @__quantum__rt__array_record_output(i64, i8*)

            declare void @__quantum__rt__result_record_output(%Result*, i8*)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="adaptive_profile" "required_num_qubits"="2" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3, !4, !5, !6, !7, !8, !9, !10}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
            !4 = !{i32 1, !"classical_ints", i1 true}
            !5 = !{i32 1, !"qubit_resetting", i1 true}
            !6 = !{i32 1, !"classical_floats", i1 false}
            !7 = !{i32 1, !"backwards_branching", i1 false}
            !8 = !{i32 1, !"classical_fixed_points", i1 false}
            !9 = !{i32 1, !"user_functions", i1 false}
            !10 = !{i32 1, !"multiple_target_branching", i1 false}
        "#]].assert_eq(&qir);
    }

    #[test]
    fn qubit_id_swap_results_in_different_id_usage() {
        let source = "namespace Test {
            @EntryPoint()
            operation Main() : (Result, Result) {
                use (q0, q1) = (Qubit(), Qubit());
                X(q0);
                Relabel([q0, q1], [q1, q0]);
                X(q1);
                (MResetZ(q0), MResetZ(q1))
            }
        }";
        let sources = SourceMap::new([("test.qs".into(), source.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::Adaptive
            | TargetCapabilityFlags::QubitReset
            | TargetCapabilityFlags::IntegerComputations;

        let (std_id, store) = crate::compile::package_store_with_stdlib(capabilities);
        let qir = get_qir(
            sources,
            language_features,
            capabilities,
            store,
            &[(std_id, None)],
        )
        .expect("Failed to generate QIR");
        expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
            block_0:
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
              call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
              call void @__quantum__rt__tuple_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__x__body(%Qubit*)

            declare void @__quantum__qis__mresetz__body(%Qubit*, %Result*) #1

            declare void @__quantum__rt__tuple_record_output(i64, i8*)

            declare void @__quantum__rt__result_record_output(%Result*, i8*)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="adaptive_profile" "required_num_qubits"="2" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3, !4, !5, !6, !7, !8, !9, !10}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
            !4 = !{i32 1, !"classical_ints", i1 true}
            !5 = !{i32 1, !"qubit_resetting", i1 true}
            !6 = !{i32 1, !"classical_floats", i1 false}
            !7 = !{i32 1, !"backwards_branching", i1 false}
            !8 = !{i32 1, !"classical_fixed_points", i1 false}
            !9 = !{i32 1, !"user_functions", i1 false}
            !10 = !{i32 1, !"multiple_target_branching", i1 false}
        "#]].assert_eq(&qir);
    }

    #[test]
    fn qubit_id_swap_across_reset_uses_updated_ids() {
        let source = "namespace Test {
            @EntryPoint()
            operation Main() : (Result, Result) {
                {
                    use (q0, q1) = (Qubit(), Qubit());
                    X(q0);
                    Relabel([q0, q1], [q1, q0]);
                    X(q1);
                    Reset(q0);
                    Reset(q1);
                }
                use (q0, q1) = (Qubit(), Qubit());
                (MResetZ(q0), MResetZ(q1))
            }
        }";
        let sources = SourceMap::new([("test.qs".into(), source.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::Adaptive
            | TargetCapabilityFlags::QubitReset
            | TargetCapabilityFlags::IntegerComputations;

        let (std_id, store) = crate::compile::package_store_with_stdlib(capabilities);
        let qir = get_qir(
            sources,
            language_features,
            capabilities,
            store,
            &[(std_id, None)],
        )
        .expect("Failed to generate QIR");
        expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
            block_0:
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__reset__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__reset__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
              call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
              call void @__quantum__rt__tuple_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__x__body(%Qubit*)

            declare void @__quantum__qis__reset__body(%Qubit*)

            declare void @__quantum__qis__mresetz__body(%Qubit*, %Result*) #1

            declare void @__quantum__rt__tuple_record_output(i64, i8*)

            declare void @__quantum__rt__result_record_output(%Result*, i8*)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="adaptive_profile" "required_num_qubits"="2" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3, !4, !5, !6, !7, !8, !9, !10}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
            !4 = !{i32 1, !"classical_ints", i1 true}
            !5 = !{i32 1, !"qubit_resetting", i1 true}
            !6 = !{i32 1, !"classical_floats", i1 false}
            !7 = !{i32 1, !"backwards_branching", i1 false}
            !8 = !{i32 1, !"classical_fixed_points", i1 false}
            !9 = !{i32 1, !"user_functions", i1 false}
            !10 = !{i32 1, !"multiple_target_branching", i1 false}
        "#]].assert_eq(&qir);
    }

    #[test]
    fn qubit_id_swap_with_out_of_order_release_uses_correct_ids() {
        let source = "namespace Test {
            @EntryPoint()
            operation Main() : (Result, Result) {
                let q0 = QIR.Runtime.__quantum__rt__qubit_allocate();
                let q1 = QIR.Runtime.__quantum__rt__qubit_allocate();
                let q2 = QIR.Runtime.__quantum__rt__qubit_allocate();
                X(q0);
                X(q1);
                X(q2);
                Relabel([q0, q1], [q1, q0]);
                QIR.Runtime.__quantum__rt__qubit_release(q0);
                let q3 = QIR.Runtime.__quantum__rt__qubit_allocate();
                X(q3);
                (MResetZ(q3), MResetZ(q1))
            }
        }";
        let sources = SourceMap::new([("test.qs".into(), source.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::Adaptive
            | TargetCapabilityFlags::QubitReset
            | TargetCapabilityFlags::IntegerComputations;

        let (std_id, store) = crate::compile::package_store_with_stdlib(capabilities);
        let qir = get_qir(
            sources,
            language_features,
            capabilities,
            store,
            &[(std_id, None)],
        )
        .expect("Failed to generate QIR");
        expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
            block_0:
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
              call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
              call void @__quantum__rt__tuple_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
              ret void
            }

            declare void @__quantum__qis__x__body(%Qubit*)

            declare void @__quantum__qis__mresetz__body(%Qubit*, %Result*) #1

            declare void @__quantum__rt__tuple_record_output(i64, i8*)

            declare void @__quantum__rt__result_record_output(%Result*, i8*)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="adaptive_profile" "required_num_qubits"="3" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3, !4, !5, !6, !7, !8, !9, !10}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
            !4 = !{i32 1, !"classical_ints", i1 true}
            !5 = !{i32 1, !"qubit_resetting", i1 true}
            !6 = !{i32 1, !"classical_floats", i1 false}
            !7 = !{i32 1, !"backwards_branching", i1 false}
            !8 = !{i32 1, !"classical_fixed_points", i1 false}
            !9 = !{i32 1, !"user_functions", i1 false}
            !10 = !{i32 1, !"multiple_target_branching", i1 false}
        "#]].assert_eq(&qir);
    }

    #[test]
    fn dynamic_integer_with_branch_and_phi_supported() {
        let source = "namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                H(q);
                MResetZ(q) == Zero ? 0 | 1
            }
        }";
        let sources = SourceMap::new([("test.qs".into(), source.into())], None);
        let language_features = LanguageFeatures::default();
        let capabilities = TargetCapabilityFlags::Adaptive
            | TargetCapabilityFlags::QubitReset
            | TargetCapabilityFlags::IntegerComputations;

        let (std_id, store) = crate::compile::package_store_with_stdlib(capabilities);
        let qir = get_qir(
            sources,
            language_features,
            capabilities,
            store,
            &[(std_id, None)],
        )
        .expect("Failed to generate QIR");
        expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
            block_0:
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
              %var_0 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 0 to %Result*))
              %var_1 = icmp eq i1 %var_0, false
              br i1 %var_1, label %block_1, label %block_2
            block_1:
              br label %block_3
            block_2:
              br label %block_3
            block_3:
              %var_3 = phi i64 [0, %block_1], [1, %block_2]
              call void @__quantum__rt__int_record_output(i64 %var_3, i8* null)
              ret void
            }

            declare void @__quantum__qis__h__body(%Qubit*)

            declare void @__quantum__qis__mresetz__body(%Qubit*, %Result*) #1

            declare i1 @__quantum__qis__read_result__body(%Result*)

            declare void @__quantum__rt__int_record_output(i64, i8*)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="adaptive_profile" "required_num_qubits"="1" "required_num_results"="1" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3, !4, !5, !6, !7, !8, !9, !10}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
            !4 = !{i32 1, !"classical_ints", i1 true}
            !5 = !{i32 1, !"qubit_resetting", i1 true}
            !6 = !{i32 1, !"classical_floats", i1 false}
            !7 = !{i32 1, !"backwards_branching", i1 false}
            !8 = !{i32 1, !"classical_fixed_points", i1 false}
            !9 = !{i32 1, !"user_functions", i1 false}
            !10 = !{i32 1, !"multiple_target_branching", i1 false}
        "#]].assert_eq(&qir);
    }
}
