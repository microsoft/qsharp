; ModuleID = '/Users/billti/src/qsharp/vscode/resources/inputData-quantinuum.h1-2'
source_filename = "qat-link"

%Qubit = type opaque
%Result = type opaque

define void @program__main() #0 {
  call void @__quantum__qis__rx__body(double 0xBFF921FB54442D18, %Qubit* null)
  call void @__quantum__qis__rz__body(double 0xBFF921FB54442D18, %Qubit* null)
  call void @__quantum__qis__rx__body(double 0xBFF921FB54442D18, %Qubit* null)
  call void @__quantum__qis__rx__body(double 0xBFF921FB54442D18, %Qubit* nonnull inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__rz__body(double 0xBFF921FB54442D18, %Qubit* nonnull inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__rx__body(double 0xBFF921FB54442D18, %Qubit* nonnull inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__rzz__body(double 0x3FF921FB54442D18, %Qubit* null, %Qubit* nonnull inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__rz__body(double 0xBFF921FB54442D18, %Qubit* null)
  call void @__quantum__qis__rz__body(double 0xBFF921FB54442D18, %Qubit* nonnull inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__rx__body(double 0xBFF921FB54442D18, %Qubit* nonnull inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__rz__body(double 0xBFF921FB54442D18, %Qubit* nonnull inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__rx__body(double 0xBFF921FB54442D18, %Qubit* nonnull inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__mz__body(%Qubit* null, %Result* writeonly null) #1
  call void @__quantum__qis__reset__body(%Qubit* null)
  %1 = call i1 @__quantum__qis__read_result__body(%Result* null)
  br i1 %1, label %flip.i, label %__quantum__qis__m__body.exit

flip.i:                                           ; preds = %0
  call void @__quantum__qis__rx__body(double 0x400921FB54442D18, %Qubit* null)
  br label %__quantum__qis__m__body.exit

__quantum__qis__m__body.exit:                     ; preds = %flip.i, %0
  call void @__quantum__qis__mz__body(%Qubit* nonnull inttoptr (i64 1 to %Qubit*), %Result* nonnull writeonly inttoptr (i64 1 to %Result*)) #1
  call void @__quantum__qis__reset__body(%Qubit* nonnull inttoptr (i64 1 to %Qubit*))
  %2 = call i1 @__quantum__qis__read_result__body(%Result* nonnull inttoptr (i64 1 to %Result*))
  br i1 %2, label %flip.i1, label %__quantum__qis__m__body.exit2

flip.i1:                                          ; preds = %__quantum__qis__m__body.exit
  call void @__quantum__qis__rx__body(double 0x400921FB54442D18, %Qubit* nonnull inttoptr (i64 1 to %Qubit*))
  br label %__quantum__qis__m__body.exit2

__quantum__qis__m__body.exit2:                    ; preds = %flip.i1, %__quantum__qis__m__body.exit
  call void @__quantum__qis__reset__body(%Qubit* null)
  call void @__quantum__qis__reset__body(%Qubit* nonnull inttoptr (i64 1 to %Qubit*))
  call void @__quantum__rt__tuple_record_output(i64 2, i8* null)
  call void @__quantum__rt__result_record_output(%Result* null, i8* null)
  call void @__quantum__rt__result_record_output(%Result* nonnull inttoptr (i64 1 to %Result*), i8* null)
  ret void
}

declare void @__quantum__qis__reset__body(%Qubit*)

declare void @__quantum__rt__tuple_record_output(i64, i8*)

declare void @__quantum__rt__result_record_output(%Result*, i8*)

declare void @__quantum__qis__rz__body(double, %Qubit*)

declare void @__quantum__qis__rzz__body(double, %Qubit*, %Qubit*)

declare void @__quantum__qis__rx__body(double, %Qubit*)

declare i1 @__quantum__qis__read_result__body(%Result*)

declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #1

attributes #0 = { "entry_point" "maxQubitIndex"="1" "maxResultIndex"="1" "output_labeling_schema" "qir_profiles"="custom" "requiredQubits"="2" "requiredResults"="2" }
attributes #1 = { "irreversible" }

!llvm.module.flags = !{!0, !1, !2, !3}

!0 = !{i32 1, !"qir_major_version", i32 1}
!1 = !{i32 7, !"qir_minor_version", i32 0}
!2 = !{i32 1, !"dynamic_qubit_management", i1 false}
!3 = !{i32 1, !"dynamic_result_management", i1 false}
