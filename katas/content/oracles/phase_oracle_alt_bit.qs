﻿namespace Quantum.Kata.Reference {

    open Microsoft.Quantum.Diagnostics;

    // This operation implements the oracle; we will learn how to implement oracles later in the tutorial
    operation AlternatingBitPattern_PhaseOracle (x: Qubit[]) : Unit is Adj + Ctl {
        let PatternOne = ControlledOnBitString([false, true, false], Z);
        let PatternTwo = ControlledOnBitString([true, false, true], Z);
        use q = Qubit();
        X(q);
        PatternOne(x, q);
        PatternTwo(x, q);
        X(q);
    }

    @EntryPoint()
    operation PhaseOracle_Demo() : Unit {
        // Allocate 3 qubits in the |000⟩ state
        use q = Qubit[3];
        // Prepare an equal superposition of all basis states
        ApplyToEachA(H, q);

        // Print the current state of the system; notice the phases of each basis state
        Message("Starting state (equal superposition of all basis states):");
        DumpMachine();

        // Apply the oracle
        AlternatingBitPattern_PhaseOracle(q);

        // Print the resulting state; notice which phases changed
        Message("State after applying the phase oracle:");
        DumpMachine();

        // Reset our state back to all zeros for deallocation
        ResetAll(q);
    }
    
}
