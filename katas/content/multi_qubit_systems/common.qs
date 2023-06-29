    // ------------------------------------------------------
    operation AssertEqualOnZeroState (testImpl : (Qubit[] => Unit is Ctl), refImpl : (Qubit[] => Unit is Adj+Ctl)) : Unit {
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

        // assert that all qubits end up in |0⟩ state
        AssertAllZero(qs);
    }