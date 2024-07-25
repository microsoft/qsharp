operation Rx(theta : Double, q : Qubit) : Unit {
    __quantum__qis__rx__body(theta, q);
}

operation Rz(theta : Double, q : Qubit) : Unit {
    __quantum__qis__rz__body(theta, q);
}

operation Rzz(theta : Double, qubit0 : Qubit, qubit1 : Qubit) : Unit {
    __quantum__qis__rzz__body(theta, qubit0, qubit1);
}

operation MResetZ(q : Qubit) : Result {
    QIR.Intrinsic.__quantum__qis__mresetz__body(q)
}

operation Reset(q : Qubit) : Unit {
    QIR.Intrinsic.__quantum__qis__reset__body(q);
}

@SimulatableIntrinsic()
operation __quantum__qis__rx__body(theta : Double, q : Qubit) : Unit {
    Std.Intrinsic.Rx(theta, q);
}

@SimulatableIntrinsic()
operation __quantum__qis__rz__body(theta : Double, q : Qubit) : Unit {
    Std.Intrinsic.Rz(theta, q);
}

@SimulatableIntrinsic()
operation __quantum__qis__rzz__body(
    theta : Double,
    qubit0 : Qubit,
    qubit1 : Qubit
) : Unit {
    Std.Intrinsic.Rzz(theta, qubit0, qubit1);
}

export Rx, Rz, Rzz, MResetZ, Reset;
