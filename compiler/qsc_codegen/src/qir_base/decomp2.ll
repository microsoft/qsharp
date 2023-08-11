%Result = type opaque
%Qubit = type opaque

declare void @__quantum__qis__rx__body(double, %Qubit*)
declare void @__quantum__qis__rz__body(double, %Qubit*)
declare void @__quantum__qis__rzz__body(double, %Qubit*, %Qubit*)
declare void @__quantum__qis__reset__body(%Qubit*)
declare void @__quantum__qis__mz__body(%Qubit*, %Result* writeonly) #0
declare i1 @__quantum__qis__read_result__body(%Result*)

define void @__quantum__qis__ccx__body(%Qubit* %c1, %Qubit* %c2, %Qubit* %q) alwaysinline {
    call void @__quantum__qis__h__body(%Qubit* %q)
    call void @__quantum__qis__t__adj(%Qubit* %c1)
    call void @__quantum__qis__t__adj(%Qubit* %c2)
    call void @__quantum__qis__cnot__body(%Qubit* %q, %Qubit* %c1)
    call void @__quantum__qis__t__body(%Qubit* %c1)
    call void @__quantum__qis__cnot__body(%Qubit* %c2, %Qubit* %q)
    call void @__quantum__qis__cnot__body(%Qubit* %c2, %Qubit* %c1)
    call void @__quantum__qis__t__body(%Qubit* %q)
    call void @__quantum__qis__t__adj(%Qubit* %c1)
    call void @__quantum__qis__cnot__body(%Qubit* %c2, %Qubit* %q)
    call void @__quantum__qis__cnot__body(%Qubit* %q, %Qubit* %c1)
    call void @__quantum__qis__t__adj(%Qubit* %q)
    call void @__quantum__qis__t__body(%Qubit* %c1)
    call void @__quantum__qis__cnot__body(%Qubit* %c2, %Qubit* %c1)
    call void @__quantum__qis__h__body(%Qubit* %q)
    ret void
}

define void @__quantum__qis__cnot__body(%Qubit* %c, %Qubit* %q) alwaysinline {
    call void @__quantum__qis__h__body(%Qubit* %q)
    call void @__quantum__qis__cz__body(%Qubit* %c, %Qubit* %q)
    call void @__quantum__qis__h__body(%Qubit* %q)
    ret void
}

define void @__quantum__qis__cx__body(%Qubit* %c, %Qubit* %q) alwaysinline {
    call void @__quantum__qis__cnot__body(%Qubit* %c, %Qubit* %q)
    ret void
}

define void @__quantum__qis__cy__body(%Qubit* %c, %Qubit* %q) alwaysinline {
    call void @__quantum__qis__s__adj(%Qubit* %q)
    call void @__quantum__qis__cnot__body(%Qubit* %c, %Qubit* %q)
    call void @__quantum__qis__s__body(%Qubit* %q)
    ret void
}

define void @__quantum__qis__cz__body(%Qubit* %c, %Qubit* %q) alwaysinline {
    call void @__quantum__qis__rzz__body(double 1.5707963267948966, %Qubit* %c, %Qubit* %q)
    call void @__quantum__qis__rz__body(double -1.5707963267948966, %Qubit* %c)
    call void @__quantum__qis__rz__body(double -1.5707963267948966, %Qubit* %q)
    ret void
}

define void @__quantum__qis__rxx__body(double %theta, %Qubit* %q1, %Qubit* %q2) alwaysinline {
    call void @__quantum__qis__h__body(%Qubit* %q1)
    call void @__quantum__qis__h__body(%Qubit* %q2)
    call void @__quantum__qis__rzz__body(double %theta, %Qubit* %q1, %Qubit* %q2)
    call void @__quantum__qis__h__body(%Qubit* %q2)
    call void @__quantum__qis__h__body(%Qubit* %q1)
    ret void
}

define void @__quantum__qis__ry__body(double %theta, %Qubit* %q) alwaysinline {
    call void @__quantum__qis__h__body(%Qubit* %q)
    call void @__quantum__qis__s__body(%Qubit* %q)
    call void @__quantum__qis__h__body(%Qubit* %q)
    call void @__quantum__qis__rz__body(double %theta, %Qubit* %q)
    call void @__quantum__qis__h__body(%Qubit* %q)
    call void @__quantum__qis__s__adj(%Qubit* %q)
    call void @__quantum__qis__h__body(%Qubit* %q)
    ret void
}

define void @__quantum__qis__ryy__body(double %theta, %Qubit* %q1, %Qubit* %q2) alwaysinline {
    call void @__quantum__qis__h__body(%Qubit* %q1)
    call void @__quantum__qis__s__body(%Qubit* %q1)
    call void @__quantum__qis__h__body(%Qubit* %q1)
    call void @__quantum__qis__h__body(%Qubit* %q2)
    call void @__quantum__qis__s__body(%Qubit* %q2)
    call void @__quantum__qis__h__body(%Qubit* %q2)
    call void @__quantum__qis__rzz__body(double %theta, %Qubit* %q1, %Qubit* %q2)
    call void @__quantum__qis__h__body(%Qubit* %q2)
    call void @__quantum__qis__s__adj(%Qubit* %q2)
    call void @__quantum__qis__h__body(%Qubit* %q2)
    call void @__quantum__qis__h__body(%Qubit* %q1)
    call void @__quantum__qis__s__adj(%Qubit* %q1)
    call void @__quantum__qis__h__body(%Qubit* %q1)
    ret void
}

define void @__quantum__qis__h__body(%Qubit* %q) alwaysinline {
    call void @__quantum__qis__rx__body(double -1.5707963267948966, %Qubit* %q)
    call void @__quantum__qis__rz__body(double -1.5707963267948966, %Qubit* %q)
    call void @__quantum__qis__rx__body(double -1.5707963267948966, %Qubit* %q)
    ret void
}

define void @__quantum__qis__s__body(%Qubit* %q) alwaysinline {
    call void @__quantum__qis__rz__body(double 1.5707963267948966, %Qubit* %q)
    ret void
}

define void @__quantum__qis__s__adj(%Qubit* %q) alwaysinline {
    call void @__quantum__qis__rz__body(double -1.5707963267948966, %Qubit* %q)
    ret void
}

define void @__quantum__qis__t__body(%Qubit* %q) alwaysinline {
    call void @__quantum__qis__rz__body(double 0.7853981633974483, %Qubit* %q)
    ret void
}

define void @__quantum__qis__t__adj(%Qubit* %q) alwaysinline {
    call void @__quantum__qis__rz__body(double -0.7853981633974483, %Qubit* %q)
    ret void
}

define void @__quantum__qis__x__body(%Qubit* %q) alwaysinline {
    call void @__quantum__qis__rx__body(double 3.141592653589793, %Qubit* %q)
    ret void
}

define void @__quantum__qis__y__body(%Qubit* %q) alwaysinline {
    call void @__quantum__qis__h__body(%Qubit* %q)
    call void @__quantum__qis__s__body(%Qubit* %q)
    call void @__quantum__qis__h__body(%Qubit* %q)
    call void @__quantum__qis__z__body(%Qubit* %q)
    call void @__quantum__qis__h__body(%Qubit* %q)
    call void @__quantum__qis__s__adj(%Qubit* %q)
    call void @__quantum__qis__h__body(%Qubit* %q)
    ret void
}

define void @__quantum__qis__z__body(%Qubit* %q) alwaysinline {
    call void @__quantum__qis__rz__body(double 3.141592653589793, %Qubit* %q)
    ret void
}

define void @__quantum__qis__swap__body(%Qubit* %q1, %Qubit* %q2) alwaysinline {
    call void @__quantum__qis__cnot__body(%Qubit* %q1, %Qubit* %q2)
    call void @__quantum__qis__cnot__body(%Qubit* %q2, %Qubit* %q1)
    call void @__quantum__qis__cnot__body(%Qubit* %q1, %Qubit* %q2)
    ret void
}

define void @__quantum__qis__m__body(%Qubit* %q, %Result* writeonly %r) alwaysinline #0 {
entry:
    call void @__quantum__qis__mresetz__body(%Qubit* %q, %Result* writeonly %r) #0
    %0 = call i1 @__quantum__qis__read_result__body(%Result* %r)
    br i1 %0, label %flip, label %exit
flip:
    call void @__quantum__qis__x__body(%Qubit* %q)
    br label %exit
exit:
    ret void
}

define void @__quantum__qis__mresetz__body(%Qubit* %q, %Result* writeonly %r) alwaysinline #0{
    call void @__quantum__qis__mz__body(%Qubit* %q, %Result* writeonly %r) #0
    call void @__quantum__qis__reset__body(%Qubit* %q)
    ret void
}

attributes #0 = { "irreversible" }
