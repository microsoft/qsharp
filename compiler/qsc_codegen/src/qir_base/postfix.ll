  ret void
}}

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

attributes #0 = {{ "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="{}" "required_num_results"="{}" }}
attributes #1 = {{ "irreversible" }}

; module flags

!llvm.module.flags = !{{!0, !1, !2, !3}}

!0 = !{{i32 1, !"qir_major_version", i32 1}}
!1 = !{{i32 7, !"qir_minor_version", i32 0}}
!2 = !{{i32 1, !"dynamic_qubit_management", i1 false}}
!3 = !{{i32 1, !"dynamic_result_management", i1 false}}
