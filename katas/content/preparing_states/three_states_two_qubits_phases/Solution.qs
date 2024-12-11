namespace Kata {
    import Std.Math.*;

    operation ThreeStates_TwoQubits_Phases(qs : Qubit[]) : Unit {
        // First create (|00⟩ + |01⟩ + |10⟩) / sqrt(3) state
        ThreeStates_TwoQubits(qs);

        R1(4.0 * PI() / 3.0, qs[0]);
        R1(2.0 * PI() / 3.0, qs[1]);
    }

    operation ThreeStates_TwoQubits(qs : Qubit[]) : Unit is Adj {
        let theta = ArcSin(1.0 / Sqrt(3.0));
        Ry(2.0 * theta, qs[0]);
        ApplyControlledOnInt(0, H, [qs[0]], qs[1]);
    }
}
