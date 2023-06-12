namespace Kata {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation GlobalPhaseIReference(q : Qubit) : Unit is Adj + Ctl {
        X(q);
        Z(q);
        Y(q);
    }

    operation VerifyExercise() : Bool {
        VerifyQubitUnitary(GlobalPhaseI, GlobalPhaseIReference)
    }
}