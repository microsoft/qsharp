namespace Kata {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Measurement;

    operation ThreeQubitMeasurement(qs : Qubit[]) : Int {
        R1(-2.0 * PI() / 3.0, qs[1]);
        R1(-4.0 * PI() / 3.0, qs[2]);

        // Apply inverse state prep of 1/sqrt(3) ( |100⟩ + |010⟩ + |001⟩ )
        Adjoint WState_Arbitrary(qs);

        // Measure all qubits: if all of them are 0, we have the first state,
        // if at least one of them is 1, we have the second state
        return MeasureInteger(qs) == 0 ? 0 | 1;
    }

    operation WState_Arbitrary(qs : Qubit[]) : Unit is Adj + Ctl {
        let N = Length(qs);

        if N == 1 {
            // Base case of recursion: |1⟩
            X(qs[0]);
        } else {
            // |W_N⟩ = |0⟩|W_(N-1)⟩ + |1⟩|0...0⟩
            // Do a rotation on the first qubit to split it into |0⟩ and |1⟩ with proper weights
            // |0⟩ -> sqrt((N-1)/N) |0⟩ + 1/sqrt(N) |1⟩
            let theta = ArcSin(1.0 / Sqrt(IntAsDouble(N)));
            Ry(2.0 * theta, qs[0]);

            // Do a zero-controlled W-state generation for qubits 1..N-1
            X(qs[0]);
            Controlled WState_Arbitrary(qs[0..0], qs[1..N - 1]);
            X(qs[0]);
        }
    }
}
