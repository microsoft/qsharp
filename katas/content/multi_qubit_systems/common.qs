namespace Kata.Verification {
    open Microsoft.Quantum.Diagnostics;

    // ------------------------------------------------------
    operation AssertEqualOnZeroState(
        testImpl: (Qubit[] => Unit is Ctl),
        refImpl : (Qubit[] => Unit is Adj+Ctl)) : Bool {

        use qs = Qubit[3];
        within {
            H(qs[0]);
        }
        apply {
            // apply operation that needs to be tested
            Controlled testImpl([qs[0]], qs[1..2]);

            // apply adjoint reference operation
            Adjoint Controlled refImpl([qs[0]], qs[1..2]);
        }

        // Implementation is correct when all qubits end up in |0⟩ state
        let isCorrect = CheckAllZero(qs);
        if not isCorrect {
            Message("The prepared state is not the same as reference state.");
        }
        ResetAll(qs);
        isCorrect
    }

}
