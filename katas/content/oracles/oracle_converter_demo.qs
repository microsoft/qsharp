namespace Kata {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Arrays;

    @EntryPoint()
    operation OracleConverterDemo () : Unit {
        // Allocate the qubits in the state |000⟩
        use register = Qubit[3];
        // Prepare an equal superposition state
        ApplyToEachA(H, register);

        Message("The equal superposition state:");
        DumpMachine();

        // Apply the `IsSeven_PhaseOracle` from the task on implementing a phase oracle
        IsSeven_PhaseOracle(register);

        // Dump the state after application of the oracle
        Message("The state after applying the phase oracle IsSeven_PhaseOracle:");
        DumpMachine();

        // Reset the qubits for deallocation
        ResetAll(register);

        // Prepare an equal superposition state again
        ApplyToEachA(H, register);

        // Apply the `IsSeven_MarkingOracle` from the task on implementing a marking oracle
        // as a phase oracle
        ApplyMarkingOracleAsPhaseOracle(IsSeven_MarkingOracle, register);

        // Dump the state after application of the oracle
        Message("The state after applying the converted marking oracle IsSeven_MarkingOracle:");
        DumpMachine();

        // reset the qubits for deallocation
        ResetAll(register);
    }

    operation IsSeven_PhaseOracle(x : Qubit[]) : Unit is Adj + Ctl {
        Controlled Z(Most(x), Tail(x));
    }

    operation IsSeven_MarkingOracle(x: Qubit[], y: Qubit): Unit is Adj + Ctl {
        Controlled X(x, y);
    }

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
}
