%Result = type opaque
%Qubit = type opaque

define void @ENTRYPOINT__main() #0 {
block_0:
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 8 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Qubit* inttoptr (i64 8 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 8 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 8 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
  %var_8 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 0 to %Result*))
  br i1 %var_8, label %block_1, label %block_2
block_1:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*))
  br label %block_2
block_2:
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
  %var_11 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 1 to %Result*))
  br i1 %var_11, label %block_3, label %block_4
block_3:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
  br label %block_4
block_4:
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Result* inttoptr (i64 2 to %Result*))
  %var_13 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 2 to %Result*))
  br i1 %var_13, label %block_5, label %block_6
block_5:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
  br label %block_6
block_6:
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Result* inttoptr (i64 3 to %Result*))
  %var_24 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 3 to %Result*))
  br i1 %var_24, label %block_7, label %block_8
block_7:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
  br label %block_8
block_8:
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Result* inttoptr (i64 4 to %Result*))
  %var_26 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 4 to %Result*))
  br i1 %var_26, label %block_9, label %block_10
block_9:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
  br label %block_10
block_10:
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 8 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Qubit* inttoptr (i64 8 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 8 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 8 to %Qubit*), %Result* inttoptr (i64 5 to %Result*))
  %var_36 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 5 to %Result*))
  br i1 %var_36, label %block_11, label %block_12
block_11:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*))
  br label %block_12
block_12:
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Result* inttoptr (i64 6 to %Result*))
  %var_39 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 6 to %Result*))
  br i1 %var_39, label %block_13, label %block_14
block_13:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
  br label %block_14
block_14:
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Result* inttoptr (i64 7 to %Result*))
  %var_41 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 7 to %Result*))
  br i1 %var_41, label %block_15, label %block_16
block_15:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
  br label %block_16
block_16:
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Result* inttoptr (i64 8 to %Result*))
  %var_52 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 8 to %Result*))
  br i1 %var_52, label %block_17, label %block_18
block_17:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
  br label %block_18
block_18:
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Result* inttoptr (i64 9 to %Result*))
  %var_54 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 9 to %Result*))
  br i1 %var_54, label %block_19, label %block_20
block_19:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
  br label %block_20
block_20:
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 8 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Qubit* inttoptr (i64 8 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 8 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 8 to %Qubit*), %Result* inttoptr (i64 10 to %Result*))
  %var_64 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 10 to %Result*))
  br i1 %var_64, label %block_21, label %block_22
block_21:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*))
  br label %block_22
block_22:
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Result* inttoptr (i64 11 to %Result*))
  %var_67 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 11 to %Result*))
  br i1 %var_67, label %block_23, label %block_24
block_23:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
  br label %block_24
block_24:
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Result* inttoptr (i64 12 to %Result*))
  %var_69 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 12 to %Result*))
  br i1 %var_69, label %block_25, label %block_26
block_25:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
  br label %block_26
block_26:
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Result* inttoptr (i64 13 to %Result*))
  %var_80 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 13 to %Result*))
  br i1 %var_80, label %block_27, label %block_28
block_27:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
  br label %block_28
block_28:
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Result* inttoptr (i64 14 to %Result*))
  %var_82 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 14 to %Result*))
  br i1 %var_82, label %block_29, label %block_30
block_29:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
  br label %block_30
block_30:
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*), %Qubit* inttoptr (i64 8 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Qubit* inttoptr (i64 8 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 8 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 8 to %Qubit*), %Result* inttoptr (i64 15 to %Result*))
  %var_92 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 15 to %Result*))
  br i1 %var_92, label %block_31, label %block_32
block_31:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 7 to %Qubit*))
  br label %block_32
block_32:
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 7 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 7 to %Qubit*), %Result* inttoptr (i64 16 to %Result*))
  %var_95 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 16 to %Result*))
  br i1 %var_95, label %block_33, label %block_34
block_33:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
  br label %block_34
block_34:
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Result* inttoptr (i64 17 to %Result*))
  %var_97 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 17 to %Result*))
  br i1 %var_97, label %block_35, label %block_36
block_35:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
  br label %block_36
block_36:
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*), %Qubit* inttoptr (i64 6 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Qubit* inttoptr (i64 5 to %Qubit*), %Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 6 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 6 to %Qubit*), %Result* inttoptr (i64 18 to %Result*))
  %var_108 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 18 to %Result*))
  br i1 %var_108, label %block_37, label %block_38
block_37:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 3 to %Qubit*))
  br label %block_38
block_38:
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 5 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 5 to %Qubit*), %Result* inttoptr (i64 19 to %Result*))
  %var_110 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 19 to %Result*))
  br i1 %var_110, label %block_39, label %block_40
block_39:
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
  br label %block_40
block_40:
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 3 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 20 to %Result*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 21 to %Result*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 22 to %Result*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Result* inttoptr (i64 23 to %Result*))
  call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Result* inttoptr (i64 24 to %Result*))
  call void @__quantum__rt__array_record_output(i64 5, i8* null)
  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 20 to %Result*), i8* null)
  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 21 to %Result*), i8* null)
  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 22 to %Result*), i8* null)
  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 23 to %Result*), i8* null)
  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 24 to %Result*), i8* null)
  ret void
}

declare void @__quantum__qis__h__body(%Qubit*)

declare void @__quantum__qis__x__body(%Qubit*)

declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)

declare void @__quantum__qis__mresetz__body(%Qubit*, %Result*) #1

declare i1 @__quantum__qis__read_result__body(%Result*)

declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)

declare void @__quantum__rt__array_record_output(i64, i8*)

declare void @__quantum__rt__result_record_output(%Result*, i8*)

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="adaptive_profile" "required_num_qubits"="9" "required_num_results"="25" }
attributes #1 = { "irreversible" }

; module flags

!llvm.module.flags = !{!0, !1, !2, !3, !4}

!0 = !{i32 1, !"qir_major_version", i32 1}
!1 = !{i32 7, !"qir_minor_version", i32 0}
!2 = !{i32 1, !"dynamic_qubit_management", i1 false}
!3 = !{i32 1, !"dynamic_result_management", i1 false}
!4 = !{i32 1, !"int_computations", !"i64"}
