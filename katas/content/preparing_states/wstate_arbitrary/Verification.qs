namespace Kata.Verification {
    import Std.Convert.*;
    import KatasUtils.*;
    import Std.Math.*;

    operation WState_Arbitrary_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        let N = Length(qs);
        Ry(2.0 * ArcSin(Sqrt(1.0 / IntAsDouble(N))), qs[0]);
        for i in 1..N - 1 {
            ApplyControlledOnInt(0, Ry(2.0 * ArcSin(Sqrt(1.0 / IntAsDouble(N - i))), _), qs[0..i-1], qs[i]);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 1..6 {
            Message($"Testing for N = {n}...");
            if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
                Kata.WState_Arbitrary,
                WState_Arbitrary_Reference,
                n
            ) {
                return false;
            }
        }

        return true;
    }
}
