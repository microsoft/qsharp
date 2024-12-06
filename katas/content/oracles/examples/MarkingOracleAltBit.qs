namespace Kata {
    import Std.Diagnostics.*;

    // This operation implements the oracle; we will learn how to implement oracles later in the kata
    operation AlternatingBitPattern_MarkingOracle(x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        ApplyControlledOnBitString([false, true, false], X, x, y);
        ApplyControlledOnBitString([true, false, true], X, x, y);
    }

    @EntryPoint()
    operation MarkingOracle_Demo() : Unit {
        use (x, y) = (Qubit[3], Qubit());
        ApplyToEachA(H, x);

        Message("Starting state (equal superposition of all basis states ⊗ |0⟩):");
        DumpMachine();

        // Apply the oracle
        AlternatingBitPattern_MarkingOracle(x, y);

        // Print the resulting state; notice which amplitudes changed
        Message("State after applying the marking oracle:");
        DumpMachine();

        ResetAll(x + [y]);
    }
}
