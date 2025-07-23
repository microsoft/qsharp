// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::{check_qasm_to_qsharp, compile_qasm_to_qir};
use expect_test::expect;
use miette::Report;

#[test]
fn profile_pragma_compiles_with_adaptive() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        #pragma qdk.qir.profile Adaptive_RI
        qubit[5] qs;
        qubit aux;
        output bit[5] results;
        ctrl(5) @ x qs[0], qs[1], qs[2], qs[3], qs[4], aux;
        results = measure qs;
    "#;

    let qsharp = compile_qasm_to_qir(source)?;
    expect![[r#"
        %Result = type opaque
        %Qubit = type opaque

        define void @ENTRYPOINT__main() #0 {
        block_0:
          call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
          call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*))
          call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 8 to %Qubit*))
          call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Qubit* inttoptr (i64 8 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
          call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 8 to %Qubit*))
          call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 8 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
          %var_5 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 0 to %Result*))
          br i1 %var_5, label %block_1, label %block_2
        block_1:
          call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*))
          br label %block_2
        block_2:
          call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
          call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
          %var_8 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 1 to %Result*))
          br i1 %var_8, label %block_3, label %block_4
        block_3:
          call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
          br label %block_4
        block_4:
          call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
          call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Result* inttoptr (i64 2 to %Result*))
          %var_10 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 2 to %Result*))
          br i1 %var_10, label %block_5, label %block_6
        block_5:
          call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
          br label %block_6
        block_6:
          call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 3 to %Result*))
          call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 4 to %Result*))
          call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 5 to %Result*))
          call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Result* inttoptr (i64 6 to %Result*))
          call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Result* inttoptr (i64 7 to %Result*))
          call void @__quantum__rt__array_record_output(i64 5, i8* null)
          call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 7 to %Result*), i8* null)
          call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 6 to %Result*), i8* null)
          call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 5 to %Result*), i8* null)
          call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 4 to %Result*), i8* null)
          call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 3 to %Result*), i8* null)
          ret void
        }

        declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)

        declare void @__quantum__qis__h__body(%Qubit*)

        declare void @__quantum__qis__mresetz__body(%Qubit*, %Result*) #1

        declare i1 @__quantum__qis__read_result__body(%Result*)

        declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)

        declare void @__quantum__qis__m__body(%Qubit*, %Result*) #1

        declare void @__quantum__rt__array_record_output(i64, i8*)

        declare void @__quantum__rt__result_record_output(%Result*, i8*)

        attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="adaptive_profile" "required_num_qubits"="9" "required_num_results"="8" }
        attributes #1 = { "irreversible" }

        ; module flags

        !llvm.module.flags = !{!0, !1, !2, !3, !4}

        !0 = !{i32 1, !"qir_major_version", i32 1}
        !1 = !{i32 7, !"qir_minor_version", i32 0}
        !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
        !3 = !{i32 1, !"dynamic_result_management", i1 false}
        !4 = !{i32 1, !"int_computations", !"i64"}
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[allow(clippy::too_many_lines)]
fn profile_pragma_compiles_with_base() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        #pragma qdk.qir.profile Base
        qubit[5] qs;
        qubit aux;
        output bit[5] results;
        ctrl(5) @ x qs[0], qs[1], qs[2], qs[3], qs[4], aux;
        results = measure qs;
    "#;

    let qsharp = compile_qasm_to_qir(source)?;
    expect![[r#"
        %Result = type opaque
        %Qubit = type opaque

        define void @ENTRYPOINT__main() #0 {
        block_0:
          call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
          call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 1 to %Qubit*))
          call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 0 to %Qubit*))
          call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
          call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 1 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
          call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
          call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
          call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 3 to %Qubit*))
          call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 2 to %Qubit*))
          call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 7 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
          call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 3 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
          call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
          call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 8 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 8 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*))
          call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 7 to %Qubit*))
          call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
          call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 8 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 8 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*))
          call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 7 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 8 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*))
          call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 8 to %Qubit*))
          call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Qubit* inttoptr (i64 8 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
          call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 8 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 8 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*))
          call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 7 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 8 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
          call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 8 to %Qubit*))
          call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 6 to %Qubit*))
          call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 7 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 8 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
          call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 8 to %Qubit*))
          call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
          call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 3 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
          call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 7 to %Qubit*))
          call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 2 to %Qubit*))
          call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 3 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
          call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
          call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
          call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 1 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
          call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 6 to %Qubit*))
          call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 0 to %Qubit*))
          call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 1 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
          call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
          call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
          call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
          call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
          call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 2 to %Result*))
          call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Result* inttoptr (i64 3 to %Result*))
          call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Result* inttoptr (i64 4 to %Result*))
          call void @__quantum__rt__array_record_output(i64 5, i8* null)
          call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 4 to %Result*), i8* null)
          call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 3 to %Result*), i8* null)
          call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 2 to %Result*), i8* null)
          call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
          call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
          ret void
        }

        declare void @__quantum__qis__h__body(%Qubit*)

        declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)

        declare void @__quantum__qis__t__body(%Qubit*)

        declare void @__quantum__qis__t__adj(%Qubit*)

        declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)

        declare void @__quantum__qis__m__body(%Qubit*, %Result*) #1

        declare void @__quantum__rt__array_record_output(i64, i8*)

        declare void @__quantum__rt__result_record_output(%Result*, i8*)

        attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="9" "required_num_results"="5" }
        attributes #1 = { "irreversible" }

        ; module flags

        !llvm.module.flags = !{!0, !1, !2, !3}

        !0 = !{i32 1, !"qir_major_version", i32 1}
        !1 = !{i32 7, !"qir_minor_version", i32 0}
        !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
        !3 = !{i32 1, !"dynamic_result_management", i1 false}
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn invalid_profile_target_errors() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        #pragma qdk.qir.profile Foo

        // Allocate qubits
        qubit[5] qs;
        qubit aux;

        // The state we are looking for is returned after execution.
        output bit[5] results;

        ctrl(5) @ x qs[0], qs[1], qs[2], qs[3], qs[4], aux;

        // Measure the qubits
        results = measure qs;
    "#;

    check_qasm_to_qsharp(
        source,
        &expect![[r#"
        Qasm.Compiler.InvalidProfilePragmaTarget

          x Invalid or missing QIR Profile: 'Foo'. Please specify one of:
          | `Unrestricted`, `Base`, `Adaptive_RI`, `Adaptive_RIF`.
           ,-[Test.qasm:3:33]
         2 |         include "stdgates.inc";
         3 |         #pragma qdk.qir.profile Foo
           :                                 ^^^
         4 | 
           `----
    "#]],
    );
    Ok(())
}

#[test]
fn missing_profile_target_errors() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        #pragma qdk.qir.profile

        // Allocate qubits
        qubit[5] qs;
        qubit aux;

        // The state we are looking for is returned after execution.
        output bit[5] results;

        ctrl(5) @ x qs[0], qs[1], qs[2], qs[3], qs[4], aux;

        // Measure the qubits
        results = measure qs;
    "#;

    check_qasm_to_qsharp(
        source,
        &expect![[r#"
        Qasm.Compiler.InvalidProfilePragmaTarget

          x Invalid or missing QIR Profile: ''. Please specify one of: `Unrestricted`,
          | `Base`, `Adaptive_RI`, `Adaptive_RIF`.
           ,-[Test.qasm:3:9]
         2 |         include "stdgates.inc";
         3 |         #pragma qdk.qir.profile
           :         ^^^^^^^^^^^^^^^^^^^^^^^
         4 | 
           `----
    "#]],
    );
    Ok(())
}
