namespace Kata {
    import Std.Diagnostics.*;

    // This operation implements the oracle; we will learn how to implement oracles later in the kata
    operation AlternatingBitPattern_PhaseOracle(x : Qubit[]) : Unit is Adj + Ctl {
        use q = Qubit();
        X(q);
        ApplyControlledOnBitString([false, true, false], Z, x, q);
        ApplyControlledOnBitString([true, false, true], Z, x, q);
        X(q);
    }

    @EntryPoint()
    operation PhaseOracle_Demo() : Unit {
        use q = Qubit[3];
        ApplyToEachA(H, q);

        Message("Starting state (equal superposition of all basis states):");
        DumpMachine();

        // Apply the oracle
        AlternatingBitPattern_PhaseOracle(q);

        // Print the resulting state; notice which phases changed
        Message("State after applying the phase oracle:");
        DumpMachine();

        ResetAll(q);
    }
}
