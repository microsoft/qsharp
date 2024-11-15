namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Math.*;

    // Reference solution that does not use measurements (to be adjointable)
    operation ThreeStates_TwoQubits_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        let theta = ArcSin(1.0 / Sqrt(3.0));
        Ry(2.0 * theta, qs[0]);
        ApplyControlledOnInt(0, H, [qs[0]], qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.ThreeStates_TwoQubits,
            ThreeStates_TwoQubits_Reference,
            2
        )
    }
}
