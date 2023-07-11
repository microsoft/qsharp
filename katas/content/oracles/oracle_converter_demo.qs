namespace Kata {

    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Arrays;

    // TODO: Consider moving to hidden code
    operation IsSeven_PhaseOracle(x : Qubit[]) : Unit is Adj + Ctl {
        Controlled Z(Most(x), Tail(x));
    }

    // TODO: Consider moving to hidden code
    operation IsSeven_MarkingOracle(x: Qubit[], y: Qubit): Unit is Adj + Ctl {
        Controlled X(x, y);
    }

    // TODO: Consider moving to hidden code
    operation ApplyMarkingOracleAsPhaseOracle(
        markingOracle: ((Qubit[], Qubit) => Unit is Adj + Ctl),
        qubits: Qubit[]) : Unit is Adj + Ctl {
            
        use minus = Qubit();
        within {
            X(minus);
            H(minus);
        } apply {
            markingOracle(qubits, minus);
        }
    }    

    @EntryPoint()
    operation OracleConverterDemo () : Unit {
        // Allocate the qubits in the state |000⟩
        use register = Qubit[3];
        // Prepare an equal superposition state
        ApplyToEachA(H, register);

        Message("The equal superposition state:");
        DumpMachine();

        // Apply the oracle from task 1.2
        IsSeven_PhaseOracle(register);

        // Dump the state after application of the oracle
        Message("The state after applying the phase oracle from task 1.2:");
        DumpMachine();

        // Reset the qubits for deallocation
        ResetAll(register);

        // Prepare an equal superposition state again
        ApplyToEachA(H, register);

        // Apply the marking oracle from task 1.3 as a phase oracle
        ApplyMarkingOracleAsPhaseOracle(IsSeven_MarkingOracle, register);

        // Dump the state after application of the oracle
        Message("The state after applying the converted marking oracle from task 1.3:");
        DumpMachine();

        // reset the qubits for deallocation
        ResetAll(register);
    }

}
