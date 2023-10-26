namespace Kata.Verification {
    open Microsoft.Quantum.Arrays;

    operation IsSeven_PhaseOracle_Reference(x : Qubit[]) : Unit is Adj + Ctl {
        Controlled Z(Most(x), Tail(x));
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let N = 3;
        let isCorrect = CheckOperationsEqualReferenced(
            N,
            Kata.IsSeven_PhaseOracle,
            IsSeven_PhaseOracle_Reference);
        if isCorrect {
            Message("All tests passed.");
        } else {
            Message("Test failed: Operation is not the same as the reference operation.");
        }
        isCorrect
    }
}
