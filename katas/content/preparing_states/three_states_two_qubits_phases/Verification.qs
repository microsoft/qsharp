namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Math.*;

    operation ThreeStates_TwoQubits_Phases_Reference(qs : Qubit[]) : Unit is Adj {
        // First create (|00⟩ + |01⟩ + |10⟩) / sqrt(3) state
        ThreeStates_TwoQubits_Reference(qs);

        R1(4.0 * PI() / 3.0, qs[0]);
        R1(2.0 * PI() / 3.0, qs[1]);
    }

    operation ThreeStates_TwoQubits_Reference(qs : Qubit[]) : Unit is Adj {
        let theta = ArcSin(1.0 / Sqrt(3.0));
        Ry(2.0 * theta, qs[0]);
        ApplyControlledOnInt(0, H, [qs[0]], qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        return CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.ThreeStates_TwoQubits_Phases,
            ThreeStates_TwoQubits_Phases_Reference,
            2
        );
    }
}
