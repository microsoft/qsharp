namespace Microsoft.Quantum.LibraryTests {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Characterization;

    /// # Summary
    /// Implementation of T-gate for Quantum Phase Estimation Oracle
    operation ApplyTOracle (power : Int, target : Qubit[]) : Unit is Adj + Ctl {
        body (...) {
            for idxPower in 0 .. power - 1 {
                T(target[0]);
            }
        }
        controlled (ctls, ...) {
            for idxPower in 0 .. power - 1 {
                Controlled T(ctls, target[0]);
            }
        }
    }

    /// # Summary
    /// Assert that the QuantumPhaseEstimation operation for the T gate
    /// return 0000 in the controlRegister when targetState is 0 and
    /// return 0010 when the targetState is 1
    operation TestQuantumPhaseEstimation() : Unit {
        use phase = Qubit[4];
        use state = Qubit();
        QuantumPhaseEstimation(ApplyTOracle, [state], phase);
        for i in 0..3 {
            Fact(CheckZero(phase[i]), "Must be 0.");
        }
        Fact(CheckZero(state), "Must be 0.");
        X(state);
        QuantumPhaseEstimation(ApplyTOracle, [state], phase);
        X(phase[2]);
        for i in 0..3 {
            Fact(CheckZero(phase[i]), "Must be 0.");
        }
        X(state);
        Fact(CheckZero(state), "Must be 0.");

        DumpMachine();
    }
}
