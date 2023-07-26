namespace Kata.Verification {
    open Microsoft.Quantum.Diagnostics;

    // ------------------------------------------------------
    // Exercise 8: State preparation using partial measurements
    // ------------------------------------------------------

    operation RefImpl_T4(qs: Qubit[]): Unit is Adj {
        // Rotate first qubit to (sqrt(2) |0⟩ + |1⟩) / sqrt(3) (task 1.4 from BasicGates kata)
        let theta = ArcSin(1.0 / Sqrt(3.0));
        Ry(2.0 * theta, qs[0]);

        // Split the state sqrt(2) |0⟩ ⊗ |0⟩ into |00⟩ + |01⟩
        ApplyControlledOnInt(0, H, [qs[0]], qs[1]);
    }


    @EntryPoint()
    operation CheckSolution(): Bool {
        use qs = Qubit[2];

        // operate the test implementation
        Kata.PostSelection(qs);

        // apply adjoint reference operation and check that the result is |0⟩
        Adjoint RefImpl_T4(qs);
        return CheckAllZero(qs);
    }

}
