namespace Kata.Verification {
    import Std.Convert.*;
    import Std.Diagnostics.*;
    import KatasUtils.*;
    import Std.Math.*;

    operation ShorEncode_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        BitflipEncode(qs[0..3..8]);
        ApplyToEachCA(H, qs[0..3..8]);
        for i in 0..2 {
            BitflipEncode(qs[3 * i..3 * i + 2]);
        }
    }

    operation BitflipEncode(qs : Qubit[]) : Unit is Adj + Ctl {
        CNOT(qs[0], qs[1]);
        CNOT(qs[0], qs[2]);
    }


    @EntryPoint()
    operation CheckSolution() : Bool {
        let range = 10;
        for i in 0..range - 1 {
            let angle = 2.0 * PI() * IntAsDouble(i) / IntAsDouble(range);
            let initialState = qs => Ry(2.0 * angle, qs[0]);
            let isCorrect = CheckOperationsEquivalenceOnInitialStateStrict(
                initialState,
                Kata.ShorEncode,
                ShorEncode_Reference,
                9
            );
            if not isCorrect {
                Message("Incorrect");
                Message($"Test fails for alpha = {Cos(angle)}, beta = {Sin(angle)}.");
                ShowQuantumStateComparison(9, initialState, Kata.ShorEncode, ShorEncode_Reference);
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
