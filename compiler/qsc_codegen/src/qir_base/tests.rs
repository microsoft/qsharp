// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap, TargetProfile};
use qsc_passes::{run_core_passes, run_default_passes, PackageType};

use crate::qir_base::generate_qir;

fn check(expr: &str, expect: &Expect) {
    let mut core = compile::core();
    assert!(run_core_passes(&mut core).is_empty());
    let mut store = PackageStore::new(core);
    let mut std = compile::std(&store, TargetProfile::Base);
    assert!(run_default_passes(
        store.core(),
        &mut std,
        PackageType::Lib,
        TargetProfile::Base
    )
    .is_empty());
    let std = store.insert(std);
    let sources = SourceMap::new([("test".into(), "".into())], Some(expr.into()));

    let mut unit = compile(&store, &[std], sources, TargetProfile::Base);
    assert!(unit.errors.is_empty(), "{:?}", unit.errors);
    assert!(run_default_passes(
        store.core(),
        &mut unit,
        PackageType::Exe,
        TargetProfile::Base
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
fn simple_program_is_valid() {
    check(
        indoc! {r#"
        {
            use q = Qubit();
            H(q);
            M(q)
        }
        "#},
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
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
            declare void @__quantum__qis__reset__body(%Qubit*)
            declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1
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
        "#]],
    );
}

#[test]
fn output_recording_array() {
    check(
        indoc! {"{use q = Qubit(); [M(q), M(q)]}"},
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)) #1
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
            declare void @__quantum__qis__reset__body(%Qubit*)
            declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1
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
        "#]],
    );
}

#[test]
fn output_recording_tuple() {
    check(
        indoc! {"{use q = Qubit(); (M(q), M(q))}"},
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)) #1
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
            declare void @__quantum__qis__reset__body(%Qubit*)
            declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1
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
        "#]],
    );
}

#[test]
fn verify_all_intrinsics() {
    check(
        indoc! {"{
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
        }"},
        &expect![[r#"
            %Result = type opaque
            %Qubit = type opaque

            define void @ENTRYPOINT__main() #0 {
              call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cy__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__rx__body(double 0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__rxx__body(double 0, %Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__ry__body(double 0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__ryy__body(double 0, %Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__rz__body(double 0, %Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__rzz__body(double 0, %Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__s__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__s__adj(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__y__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__z__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__swap__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__reset__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*)) #1
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)) #1
              call void @__quantum__qis__reset__body(%Qubit* inttoptr (i64 0 to %Qubit*))
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
            declare void @__quantum__qis__reset__body(%Qubit*)
            declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1
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
        "#]],
    );
}

#[test]
fn complex_program_is_valid() {
    check(
        indoc! {"{
            open Microsoft.Quantum.Measurement;
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
        }"},
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
              call void @__quantum__qis__reset__body(%Qubit* inttoptr (i64 0 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)) #1
              call void @__quantum__qis__reset__body(%Qubit* inttoptr (i64 1 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 2 to %Result*)) #1
              call void @__quantum__qis__reset__body(%Qubit* inttoptr (i64 2 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Result* inttoptr (i64 3 to %Result*)) #1
              call void @__quantum__qis__reset__body(%Qubit* inttoptr (i64 3 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Result* inttoptr (i64 4 to %Result*)) #1
              call void @__quantum__qis__reset__body(%Qubit* inttoptr (i64 4 to %Qubit*))
              call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Result* inttoptr (i64 5 to %Result*)) #1
              call void @__quantum__qis__reset__body(%Qubit* inttoptr (i64 5 to %Qubit*))
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
            declare void @__quantum__qis__reset__body(%Qubit*)
            declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1
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
        "#]],
    );
}
