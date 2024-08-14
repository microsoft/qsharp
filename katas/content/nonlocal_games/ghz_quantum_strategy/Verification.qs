namespace Kata.Verification {

    function WinCondition_Reference (rst : Bool[], abc : Bool[]) : Bool {
        return (rst[0] or rst[1] or rst[2]) == (abc[0] != abc[1] != abc[2]);
    }

    function RefereeBits () : Bool[][] {
        return [[false, false, false],
                [true, true, false],
                [false, true, true],
                [true, false, true]];
    }

    operation CreateEntangledTriple_Reference (qs : Qubit[]) : Unit is Adj {
        X(qs[0]);
        X(qs[1]);
        H(qs[0]);
        H(qs[1]);
        Controlled Z([qs[0]], qs[1]);
        ApplyControlledOnBitString([false, true], X, [qs[0], qs[1]], qs[2]);
        ApplyControlledOnBitString([true, false], X, [qs[0], qs[1]], qs[2]);
    }

    operation PlayQuantumGHZ_Reference (strategies : ((Bool, Qubit) => Bool)[], inputs : Bool[], qubits : Qubit[]) : Bool[] {
        let r = inputs[0];
        let s = inputs[1];
        let t = inputs[2];
        let a = strategies[0](r, qubits[0]);
        let b = strategies[1](s, qubits[1]);
        let c = strategies[2](t, qubits[2]);
        return [a, b, c];
    }

    @EntryPoint()
    operation CheckSolution () : Bool {
        use qs = Qubit[3];
        let inputs = RefereeBits();
        let strategies = [Kata.AliceQuantum, Kata.BobQuantum, Kata.CharlieQuantum];

        let iterations = 1000;
        mutable wins = 0;
        for _ in 0 .. iterations - 1 {
            for bits in inputs {
                CreateEntangledTriple_Reference(qs);
                let abc = PlayQuantumGHZ_Reference(strategies, bits, qs);
                if WinCondition_Reference(bits, abc) {
                    set wins = wins + 1;
                }
		ResetAll(qs);
            }
        }
        if wins < iterations*Length(inputs) {
            Message($"Player's quantum strategies get {wins} wins out of {iterations*Length(inputs)} possible inputs, which is not optimal");
            return false;
        }
        Message("Correct!");
        true
    }
}
