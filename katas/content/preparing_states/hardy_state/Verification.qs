namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Math.*;

    operation Hardy_State_Reference(qs : Qubit[]) : Unit is Adj {
        let theta = ArcCos(Sqrt(10.0 / 12.0));
        Ry(2.0 * theta, qs[0]);
        ApplyControlledOnInt(0, Ry, [qs[0]], (2.0 * ArcCos(3.0 / Sqrt(10.0)), qs[1]));
        ApplyControlledOnInt(1, Ry, [qs[0]], (2.0 * PI() / 4.0, qs[1]));
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        return CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.Hardy_State,
            Hardy_State_Reference,
            2
        );
    }
}
