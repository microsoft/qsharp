namespace Kata.Verification {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Katas;

    operation Entangle_Reference (qAlice : Qubit, qBob : Qubit) : Unit is Adj {
        H(qAlice);
        CNOT(qAlice,qBob);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        use qAlice = Qubit();
        use qBob = Qubit();

        Kata.Entangle(qAlice,qBob);

        Adjoint Entangle_Reference(qAlice,qBob);

        if CheckAllZero([qAlice,qBob]) {
            Message("Correct Solution");
            return true;
        }

        Message("This is not quite right. Try again!");
        ResetAll([qAlice,qBob]);
        return false;
    }
}
