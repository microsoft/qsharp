namespace Test {
    import Std.Diagnostics.*;

    operation TestQPE_Phase(gate: Qubit => Unit is Ctl + Adj, power: Int) : Unit {
        use state = Qubit();
        use phase = Qubit[4];

        // Estimate eigenvalue of gate on eigenvector of |0⟩
        ApplyQPE(ApplyOperationPowerCA(_, qs => gate(qs[0]), _), [state], phase);
        // It should be 1, which corresponds to the binary fraction 0.0000 of 2π.
        Fact(CheckAllZero(phase), "Expected state |0000⟩");

        // Estimate eigenvalue of gate on eigenvector of |1⟩
        X(state);
        ApplyQPE(ApplyOperationPowerCA(_, qs => gate(qs[0]), _), [state], phase);
        // The eigenvalue of gate on |1⟩ is exp(i * π / 2^power),
        // which corresponds to the binary fraction 0.0...01 of 2π,
        // where the 1 is at the position power counting from 0.
        // The state should be |0...01⟩, or, in a little-endian register
        // phase[0] = 0, phase[1] = 0, ..., phase[N-power-1] = 1
        X(phase[Length(phase)-power-1]);
        Fact(CheckAllZero(phase), $"Incorrect phase for fraction index {power}.");

        Reset(state);
    }

    operation TestQPE_Z() : Unit {
        TestQPE_Phase(Z, 0); // exp(i * π / 2^0)
    }

    operation TestQPE_S() : Unit {
        TestQPE_Phase(S, 1); // exp(i * π / 2^1)

    }

    operation TestQPE_T() : Unit {
        TestQPE_Phase(T, 2); // exp(i * π / 2^2)
    }

}
