namespace Kata.Verification {
    import Std.Diagnostics.*;
    import KatasUtils.*;
    import Std.Math.*;

    // Reference solution that does not use measurements (to be adjointable)
    operation StatePrep_Rotations(qs : Qubit[]) : Unit is Adj {
        // Rotate first qubit to (sqrt(2) |0⟩ + |1⟩) / sqrt(3)
        let theta = ArcSin(1.0 / Sqrt(3.0));
        Ry(2.0 * theta, qs[0]);

        // Split the state sqrt(2) |0⟩ ⊗ |0⟩ into |00⟩ + |01⟩
        ApplyControlledOnInt(0, H, [qs[0]], qs[1]);
    }


    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckOperationsEquivalenceOnZeroStateWithFeedback(Kata.PostSelection, StatePrep_Rotations, 2)
    }
}
