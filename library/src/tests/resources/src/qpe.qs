namespace Test {
    import Std.Diagnostics.*;

    operation TestQPE_Phase(phaseGate : Qubit => Unit is Ctl + Adj, power : Int) : Unit {
        use state = Qubit();
        use phase = Qubit[4];

        // Estimate eigenvalue of a phase gate on eigenvector of |0⟩
        ApplyQPE(ApplyOperationPowerCA(_, qs => phaseGate(qs[0]), _), [state], phase);
        // Eigenvalue should be 1 = exp(i * 2π * 0.0000), so the estimaped phase
        // should be 0.0000 fraction of 2π.
        Fact(CheckAllZero(phase), "Expected state |0000⟩");

        // Estimate eigenvalue of a phase gate on eigenvector of |1⟩
        X(state);
        ApplyQPE(ApplyOperationPowerCA(_, qs => phaseGate(qs[0]), _), [state], phase);
        // The eigenvalue of a phase gate on eigenvector |1⟩ is exp(i * 2π / 2^power),
        // so the eigenvalue phase is the binary fraction 0.0…01 of 2π,
        // where the 1 is at the position `power` after the point (counting from 1).
        // So the a little-endian register `phase` should be
        // phase[0] = 0, phase[1] = 0, …, phase[N-power] = 1, phase[N-power+1] = 0, …
        X(phase[Length(phase)-power]);
        Fact(CheckAllZero(phase), $"Incorrect phase for exp(i * 2π / 2^{power}).");

        Reset(state);
    }

    operation TestQPE_Z() : Unit {
        TestQPE_Phase(Z, 1); // eigenvalue = exp(i * 2π / 2^1)
    }

    operation TestQPE_S() : Unit {
        TestQPE_Phase(S, 2); // eigenvalue = exp(i * 2π / 2^2)

    }

    operation TestQPE_T() : Unit {
        TestQPE_Phase(T, 3); // eigenvalue = exp(i * 2π / 2^3)
    }

    // Phase gate is a rotation around the Z axis and an ajustment for the global phase.
    operation P(phase : Double, q : Qubit) : Unit is Ctl + Adj {
        Rz(phase, q);
        Exp([], phase / 2.0, []);
    }

    operation TestQPE_P() : Unit {
        TestQPE_Phase(P(Std.Math.PI() / 8.0, _), 4); // eigenvalue = exp(i * 2π / 2^4)
    }
}
