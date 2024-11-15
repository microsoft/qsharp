namespace Kata {
    import Std.Diagnostics.*;
    import Std.Math.*;

    operation PhaseOracle_Zero(x : Qubit[]) : Unit {
        // Do nothing...
    }

    operation PhaseOracle_One(x : Qubit[]) : Unit {
        // Apply a global phase of -1
        R(PauliI, 2.0 * PI(), x[0]);
    }

    operation PhaseOracle_Xmod2(x : Qubit[]) : Unit {
        let N = Length(x);
        // Array elements are indexed 0 through Length(x) - 1, inclusive.
        Z(x[N - 1]);
    }

    @EntryPoint()
    operation OracleImplementationDemo() : Unit {
        use qs = Qubit[2];
        Ry(2.0 * ArcCos(0.5), qs[0]);
        Ry(2.0 * ArcCos(0.6), qs[1]);
        Message("The state before oracle application:");
        DumpMachine();

        // Apply the oracle.
        // Experiment with using other oracles to see their behavior!
        // (Note that the -1 global phase might not show up in simulation)
        PhaseOracle_Xmod2(qs);

        Message("The state after oracle application:");
        DumpMachine();

        ResetAll(qs);
    }
}
