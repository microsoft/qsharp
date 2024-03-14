// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]
#![allow(clippy::needless_raw_string_hashes)]

use std::sync::Arc;

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_data_structures::language_features::LanguageFeatures;
use qsc_frontend::compile::{self, compile, PackageStore, RuntimeCapabilityFlags, SourceMap};
use qsc_passes::{run_core_passes, run_default_passes, PackageType};

use crate::qir_base::generate_qir;

fn check(program: &str, expr: Option<&str>, expect: &Expect) {
    let mut core = compile::core();
    assert!(run_core_passes(&mut core).is_empty());
    let mut store = PackageStore::new(core);
    let mut std = compile::std(&store, RuntimeCapabilityFlags::empty());
    assert!(run_default_passes(
        store.core(),
        &mut std,
        PackageType::Lib,
        RuntimeCapabilityFlags::empty()
    )
    .is_empty());
    let std = store.insert(std);

    let expr_as_arc: Option<Arc<str>> = expr.map(|s| Arc::from(s.to_string()));
    let sources = SourceMap::new([("test".into(), program.into())], expr_as_arc);

    let mut unit = compile(
        &store,
        &[std],
        sources,
        RuntimeCapabilityFlags::empty(),
        LanguageFeatures::default(),
    );
    assert!(unit.errors.is_empty(), "{:?}", unit.errors);
    assert!(run_default_passes(
        store.core(),
        &mut unit,
        PackageType::Exe,
        RuntimeCapabilityFlags::empty()
    )
    .is_empty());
    let package = store.insert(unit);

    let qir = generate_qir(&store, package);
    match qir {
        Ok(qir) => expect.assert_eq(&qir),
        Err((err, _)) => expect.assert_debug_eq(&err),
    }
}

#[test]
fn simple_entry_program_is_valid() {
    check(
        indoc! {r#"
    namespace Sample {
        @EntryPoint()
        operation Entry() : Result
        {
            use q = Qubit();
            H(q);
            M(q)
        }
    }
        "#},
        None,
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
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
        "#]],
    );
}

#[test]
fn simple_program_is_valid() {
    check(
        "",
        Some(indoc! {r#"
        {
            use q = Qubit();
            H(q);
            M(q)
        }
        "#}),
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
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
        "#]],
    );
}

#[test]
fn output_recording_array() {
    check(
        "",
        Some(indoc! {"{use q = Qubit(); [M(q), M(q)]}"}),
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)) #1
              call void @__quantum__rt__array_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
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

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="3" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn output_recording_tuple() {
    check(
        "",
        Some(indoc! {"{use q = Qubit(); (M(q), M(q))}"}),
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)) #1
              call void @__quantum__rt__tuple_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
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

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="3" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn reset_allocates_new_qubit_id() {
    check(
        "",
        Some(indoc! {"{use q = Qubit(); H(q); Reset(q); H(q); M(q)}"}),
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
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

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="3" "required_num_results"="1" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn reuse_after_measurement_uses_fresh_aux_qubit_id() {
    check(
        "",
        Some(indoc! {"{use q = Qubit(); H(q); M(q); H(q); M(q)}"}),
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)) #1
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
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

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="3" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn qubit_allocation_allows_reuse_of_unmeasured_qubits() {
    check(
        "",
        Some(indoc! {"{
            { use (c, q) = (Qubit(), Qubit()); CNOT(c, q); MResetZ(q); }
            { use (c, q) = (Qubit(), Qubit()); CNOT(c, q); MResetZ(q) }
        }"}),
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)) #1
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
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

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="3" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn verify_all_intrinsics() {
    check(
        "",
        Some(indoc! {"{
            use (q1, q2, q3) = (Qubit(), Qubit(), Qubit());
            CCNOT(q1, q2, q3);
            CX(q1, q2);
            CY(q1, q2);
            CZ(q1, q2);
            Rx(0.0, q1);
            Rxx(0.0, q1, q2);
            Ry(0.0, q1);
            Ryy(0.0, q1, q2);
            Rz(0.0, q1);
            Rzz(0.0, q1, q2);
            H(q1);
            S(q1);
            Adjoint S(q1);
            T(q1);
            Adjoint T(q1);
            X(q1);
            Y(q1);
            Z(q1);
            SWAP(q1, q2);
            Reset(q1);
            (M(q1),
            Microsoft.Quantum.Measurement.MResetZ(q1))
        }"}),
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cy__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__rx__body(double 0.0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__rxx__body(double 0.0, %Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__ry__body(double 0.0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__ryy__body(double 0.0, %Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__rz__body(double 0.0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__rzz__body(double 0.0, %Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__s__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__s__adj(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__y__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__z__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__swap__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)) #1
              call void @__quantum__rt__tuple_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
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

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="5" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn complex_program_is_valid() {
    check(
        "",
        Some(indoc! {"{
            open Microsoft.Quantum.Math;

            operation SWAPfromExp(q1 : Qubit, q2 : Qubit) : Unit is Ctl + Adj {
                let theta  = PI() / 4.0;
                Exp([PauliX, PauliX], theta, [q1, q2]);
                Exp([PauliY, PauliY], theta, [q1, q2]);
                Exp([PauliZ, PauliZ], theta, [q1, q2]);
            }

            use (aux, ctls, qs) = (Qubit(), Qubit[3], Qubit[2]);
            within {
                H(aux);
                ApplyToEachA(CNOT(aux, _), ctls + qs);
            }
            apply {
                Controlled SWAPfromExp(ctls, (qs[0], qs[1]));

                Controlled Adjoint SWAP(ctls, (qs[0], qs[1]));
            }

            MResetEachZ([aux] + ctls + qs)
        }"}),
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__rz__body(double -0.7853981633974483, %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__rz__body(double 0.7853981633974483, %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__s__body(%Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__rz__body(double -0.7853981633974483, %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__rz__body(double 0.7853981633974483, %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__s__adj(%Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__rz__body(double -0.7853981633974483, %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__rz__body(double 0.7853981633974483, %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 2 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Result* inttoptr (i64 3 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Result* inttoptr (i64 4 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Result* inttoptr (i64 5 to %Result*)) #1
              call void @__quantum__rt__array_record_output(i64 6, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 2 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 3 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 4 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 5 to %Result*), i8* null)
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

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="8" "required_num_results"="6" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn qubit_ids_properly_reused() {
    check(
        indoc! {"
        namespace Test {

            open Microsoft.Quantum.Intrinsic;

            // Verifies the use of the CNOT quantum gate from Q#'s Microsoft.Quantum.Intrinsic namespace.
            // Expected simulation output: ([0, 0], [1, 1]).
            @EntryPoint()
            operation IntrinsicCNOT() : (Result[], Result[]) {
                use registerA = Qubit[2];           // |00⟩
                CNOT(registerA[0], registerA[1]);   // |00⟩
                let resultsA = MeasureEachZ(registerA);
                ResetAll(registerA);

                use registerB = Qubit[2];           // |00⟩
                X(registerB[0]);                    // |10⟩
                CNOT(registerB[0], registerB[1]);   // |11⟩
                let resultsB = MeasureEachZ(registerB);
                ResetAll(registerB);

                return (resultsA, resultsB);
            }
        }
        "},
        None,
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Result* inttoptr (i64 2 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Result* inttoptr (i64 3 to %Result*)) #1
              call void @__quantum__rt__tuple_record_output(i64 2, i8* null)
              call void @__quantum__rt__array_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
              call void @__quantum__rt__array_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 2 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 3 to %Result*), i8* null)
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

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="8" "required_num_results"="4" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn custom_intrinsic_on_single_qubit() {
    check(
        indoc! {"
        namespace Test {
            @EntryPoint()
            operation Test() : (Result, Result) {
                use (q1, q2) = (Qubit(), Qubit());
                MyCustomGate(q1);
                MyCustomGate(q2);
                return (MResetZ(q1), MResetZ(q2));
            }

            operation MyCustomGate(q : Qubit) : Unit {
                body intrinsic;
            }
        }
        "},
        None,
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @MyCustomGate(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @MyCustomGate(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)) #1
              call void @__quantum__rt__tuple_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
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
            declare void @MyCustomGate(%Qubit*)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn multiple_custom_intrinsic_calls() {
    check(
        indoc! {"
        namespace Test {
            @EntryPoint()
            operation Test() : (Result, Result) {
                use (q0, q1) = (Qubit(), Qubit());
                MySecondCustomGate(q0, q1);
                MyCustomGate(q0);
                MyCustomFunc(42);
                return (MResetZ(q0), MResetZ(q1));
            }

            operation MyCustomGate(q : Qubit) : Unit {
                body intrinsic;
            }

            operation MySecondCustomGate(q0 : Qubit, q1 : Qubit) : Unit {
                body intrinsic;
            }

            function MyCustomFunc(n : Int) : Unit {
                body intrinsic;
            }
        }
        "},
        None,
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @MySecondCustomGate(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @MyCustomGate(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @MyCustomFunc(i64 42)
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)) #1
              call void @__quantum__rt__tuple_record_output(i64 2, i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
              call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
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
            declare void @MySecondCustomGate(%Qubit*, %Qubit*)
            declare void @MyCustomGate(%Qubit*)
            declare void @MyCustomFunc(i64)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="2" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn custom_intrinsic_on_qubit_and_double() {
    check(
        indoc! {"
        namespace Test {
            @EntryPoint()
            operation Test() : Result {
                use q = Qubit();
                let d = 3.14;
                MyCustomGate(q, d);
                return MResetZ(q);
            }

            operation MyCustomGate(q : Qubit, d : Double) : Unit {
                body intrinsic;
            }
        }
        "},
        None,
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @MyCustomGate(%Qubit* inttoptr (i64 0 to %Qubit*), double 3.14)
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
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
            declare void @MyCustomGate(%Qubit*, double)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="1" "required_num_results"="1" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn custom_intrinsic_on_qubit_and_bool() {
    check(
        indoc! {"
        namespace Test {
            @EntryPoint()
            operation Test() : Result {
                use q = Qubit();
                let b = true;
                MyCustomGate(q, b);
                return MResetZ(q);
            }

            operation MyCustomGate(q : Qubit, b : Bool) : Unit {
                body intrinsic;
            }
        }
        "},
        None,
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @MyCustomGate(%Qubit* inttoptr (i64 0 to %Qubit*), i1 true)
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
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
            declare void @MyCustomGate(%Qubit*, i1)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="1" "required_num_results"="1" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn custom_intrinsic_on_qubit_and_int() {
    check(
        indoc! {"
        namespace Test {
            @EntryPoint()
            operation Test() : Result {
                use q = Qubit();
                let i = 42;
                MyCustomGate(q, i);
                return MResetZ(q);
            }

            operation MyCustomGate(q : Qubit, i : Int) : Unit {
                body intrinsic;
            }
        }
        "},
        None,
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @MyCustomGate(%Qubit* inttoptr (i64 0 to %Qubit*), i64 42)
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
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
            declare void @MyCustomGate(%Qubit*, i64)

            attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="1" "required_num_results"="1" }
            attributes #1 = { "irreversible" }

            ; module flags

            !llvm.module.flags = !{!0, !1, !2, !3}

            !0 = !{i32 1, !"qir_major_version", i32 1}
            !1 = !{i32 7, !"qir_minor_version", i32 0}
            !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
            !3 = !{i32 1, !"dynamic_result_management", i1 false}
        "#]],
    );
}

#[test]
fn custom_intrinsic_fail_on_result_arg() {
    check(
        indoc! {"
        namespace Test {
            @EntryPoint()
            operation Test() : Result {
                use q = Qubit();
                let r = MResetZ(q);
                MyCustomGate(q, r);
                return r;
            }

            operation MyCustomGate(q : Qubit, r : Result) : Unit {
                body intrinsic;
            }
        }
        "},
        None,
        &expect![[r#"
            IntrinsicFail(
                "MyCustomGate",
                "unsupported argument type: Result",
                PackageSpan {
                    package: PackageId(
                        2,
                    ),
                    span: Span {
                        lo: 177,
                        hi: 261,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn custom_intrinsic_fail_on_bigint_arg() {
    check(
        indoc! {"
        namespace Test {
            @EntryPoint()
            operation Test() : Result {
                use q = Qubit();
                let i = 42L;
                MyCustomGate(q, i);
                return MResetZ(q);
            }

            operation MyCustomGate(q : Qubit, i : BigInt) : Unit {
                body intrinsic;
            }
        }
        "},
        None,
        &expect![[r#"
            IntrinsicFail(
                "MyCustomGate",
                "unsupported argument type: BigInt",
                PackageSpan {
                    package: PackageId(
                        2,
                    ),
                    span: Span {
                        lo: 179,
                        hi: 263,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn custom_intrinsic_fail_on_string_arg() {
    check(
        indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Test() : Result {
                use q = Qubit();
                let s = "hello, world";
                MyCustomGate(q, s);
                return MResetZ(q);
            }

            operation MyCustomGate(q : Qubit, s : String) : Unit {
                body intrinsic;
            }
        }
        "#},
        None,
        &expect![[r#"
            IntrinsicFail(
                "MyCustomGate",
                "unsupported argument type: String",
                PackageSpan {
                    package: PackageId(
                        2,
                    ),
                    span: Span {
                        lo: 190,
                        hi: 274,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn custom_intrinsic_fail_on_array_arg() {
    check(
        indoc! {"
        namespace Test {
            @EntryPoint()
            operation Test() : Result {
                use q = Qubit();
                let a = [1, 2, 3];
                MyCustomGate(q, a);
                return MResetZ(q);
            }

            operation MyCustomGate(q : Qubit, a : Int[]) : Unit {
                body intrinsic;
            }
        }
        "},
        None,
        &expect![[r#"
            IntrinsicFail(
                "MyCustomGate",
                "unsupported argument type: Array",
                PackageSpan {
                    package: PackageId(
                        2,
                    ),
                    span: Span {
                        lo: 185,
                        hi: 268,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn custom_intrinsic_fail_on_tuple_arg() {
    check(
        indoc! {"
        namespace Test {
            @EntryPoint()
            operation Test() : Result {
                use q = Qubit();
                let t = (1, 2, 3);
                MyCustomGate(q, t);
                return MResetZ(q);
            }

            operation MyCustomGate(q : Qubit, t : (Int, Int, Int)) : Unit {
                body intrinsic;
            }
        }
        "},
        None,
        &expect![[r#"
            IntrinsicFail(
                "MyCustomGate",
                "unsupported argument type: Tuple",
                PackageSpan {
                    package: PackageId(
                        2,
                    ),
                    span: Span {
                        lo: 185,
                        hi: 278,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn custom_intrinsic_fail_on_non_unit_return() {
    check(
        indoc! {"
        namespace Test {
            @EntryPoint()
            operation Test() : Result {
                use q = Qubit();
                MyCustomGate(q);
                return MResetZ(q);
            }

            function MyCustomGate(q : Qubit) : Int {
                body intrinsic;
            }
        }
        "},
        None,
        &expect![[r#"
            UnsupportedIntrinsicType(
                "MyCustomGate",
                PackageSpan {
                    package: PackageId(
                        2,
                    ),
                    span: Span {
                        lo: 155,
                        hi: 225,
                    },
                },
            )
        "#]],
    );
}
