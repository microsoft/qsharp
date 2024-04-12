namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Katas;

    operation ZeroAndBitstringSuperposition_Reference (qs : Qubit[], bits : Bool[]) : Unit is Adj + Ctl {
        H(qs[0]);

        for i in 1 .. Length(qs) - 1 {
            if bits[i] {
                CNOT(qs[0], qs[i]);
            }
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let qubits = 3;
        for i in 0 .. (2 ^ qubits) - 1 {
            let bits = IntAsBoolArray(i, qubits);
            Message($"Testing for bits = {bits}...");
            if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.ZeroAndBitstringSuperposition(_, bits),
            ZeroAndBitstringSuperposition_Reference(_, bits),
            qubits) {
                return false;
            }
        }

        true
    }
}
