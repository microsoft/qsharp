namespace Kata {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation ApplyYReference(q : Qubit) : Unit is Adj + Ctl {
        Y(q);
    }

    operation VerifyExercise() : Bool {
        VerifyQubitUnitary(ApplyY, ApplyYReference)
    }
}