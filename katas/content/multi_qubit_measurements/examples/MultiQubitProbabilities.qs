namespace Kata {
    import Std.Diagnostics.*;
    import Std.Math.*;

    @EntryPoint()
    operation CalculateProbabilities() : Unit {
        use qs = Qubit[2];

        Ry(2. * ArcCos(2. / 3.), qs[1]);
        Controlled Ry([qs[1]], (2. * ArcCos(1. / Sqrt(5.)), qs[0]));

        Message("The initial state of the system");
        Message("(includes measurement outcome probabilities in computational basis):");
        DumpMachine();

        // Change the basis
        H(qs[0]);
        H(qs[1]);

        Message("Final state of the two-qubit system:");
        Message("(includes measurement outcome probabilities in Pauli X basis):");
        DumpMachine();

        // Return the qubit array into state |00❭.
        ResetAll(qs);
    }
}
