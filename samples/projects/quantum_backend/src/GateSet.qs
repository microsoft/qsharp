operation Rx(theta : Double, q: Qubit) : Unit {
    __quantum__qis__rx__body(theta, q);
}

operation Rz(theta : Double, q: Qubit) : Unit {
    __quantum__qis__rz__body(theta, q);
}

operation Rzz(theta : Double, qubit0: Qubit, qubit1: Qubit) : Unit {
    __quantum__qis__rzz__body(theta, qubit0, qubit1);
}

operation Mz(q : Qubit) : Result {
    __quantum__qis__mz__body(q)
}

operation Reset(q : Qubit) : Unit {
    __quantum__qis__reset__body(q);
}

@SimulatableIntrinsic()
operation __quantum__qis__rx__body(theta : Double, q: Qubit) : Unit {
    Std.Intrinsic.Rx(theta, q);
}

@SimulatableIntrinsic()
operation __quantum__qis__rz__body(theta : Double, q: Qubit) : Unit {
    Std.Intrinsic.Rz(theta, q);
}

@SimulatableIntrinsic()
operation __quantum__qis__rzz__body(
    theta : Double, 
    qubit0: Qubit, 
    qubit1: Qubit)
    : Unit {
    Std.Intrinsic.Rzz(theta, qubit0, qubit1);
}

@SimulatableIntrinsic()
operation __quantum__qis__mz__body(q: Qubit) : Result {
    Std.Intrinsic.M(q)
}

@SimulatableIntrinsic()
operation __quantum__qis__reset__body(q: Qubit) : Unit {
    Std.Intrinsic.Reset(q);
}

export Rx, Rz, Rzz, Mz, Reset;
