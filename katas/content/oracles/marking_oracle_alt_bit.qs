﻿namespace Quantum.Kata.Reference {

    open Microsoft.Quantum.Diagnostics;

    // This operation implements the oracle; we will learn how to implement oracles later in the tutorial
    operation AlternatingBitPattern_MarkingOracle(x: Qubit[], y: Qubit) : Unit is Adj + Ctl {
        let PatternOne = ControlledOnBitString([false, true, false], X);
        let PatternTwo = ControlledOnBitString([true, false, true], X);
        PatternOne(x, y);
        PatternTwo(x, y);
    }

    @EntryPoint()
    operation MarkingOracle_Demo() : Unit {
        // Allocate the qubits in the |000⟩|0⟩ state
        use (x, y) = (Qubit[3], Qubit());
        // Prepare an equal superposition of all basis states in the input register
        ApplyToEachA(H, x);

        // Print the current state of the system; notice the amplitudes of each basis state
        Message("Starting state (equal superposition of all basis states ⊗ |0⟩):");
        DumpMachine();

        // Apply the oracle
        AlternatingBitPattern_MarkingOracle(x, y);

        // Print the resulting state; notice which amplitudes changed
        Message("State after applying the marking oracle:");
        DumpMachine();

        // Reset our state back to all zeros for deallocation
        ResetAll(x + [y]);
    }

}
