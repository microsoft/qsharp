namespace Kata {
    operation CreateEntangledTriple (qs : Qubit[]) : Unit is Adj {
        X(qs[0]);
        X(qs[1]);

        H(qs[0]);
        H(qs[1]);
        // At this point we have (|000⟩ - |010⟩ - |100⟩ + |110⟩) / 2

        // Flip the sign of the last term
        Controlled Z([qs[0]], qs[1]);

        // Flip the state of the last qubit for the two middle terms
        ApplyControlledOnBitString([false, true], X, [qs[0], qs[1]], qs[2]);
        ApplyControlledOnBitString([true, false], X, [qs[0], qs[1]], qs[2]);
    }

    operation PlayQuantumGHZ (strategies : ((Bool, Qubit) => Bool)[], inputs : Bool[]) : Bool[] {
        use qs = Qubit[3];
        CreateEntangledTriple(qs);

        let r = inputs[0];
        let s = inputs[1];
        let t = inputs[2];
        let a = strategies[0](r, qs[0]);
        let b = strategies[1](s, qs[1]);
        let c = strategies[2](t, qs[2]);

        ResetAll(qs);
        return [a, b, c];
    }
}
