namespace Kata.Verification {
    import Std.Convert.*;
    import Std.Diagnostics.*;
    import KatasUtils.*;
    import Std.Math.*;

    operation PhaseflipEncode_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        CNOT(qs[0], qs[1]);
        CNOT(qs[0], qs[2]);
        ApplyToEachCA(H, qs);
    }


    @EntryPoint()
    operation CheckSolution() : Bool {
        let range = 10;
        for i in 0..range - 1 {
            let angle = 2.0 * PI() * IntAsDouble(i) / IntAsDouble(range);
            let initialState = qs => Ry(2.0 * angle, qs[0]);
            let isCorrect = CheckOperationsEquivalenceOnInitialStateStrict(
                initialState,
                Kata.PhaseflipEncode,
                PhaseflipEncode_Reference,
                3
            );
            if not isCorrect {
                Message("Incorrect");
                Message($"Test fails for alpha = {Cos(angle)}, beta = {Sin(angle)}.");
                ShowQuantumStateComparison(3, initialState, Kata.PhaseflipEncode, PhaseflipEncode_Reference);
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
