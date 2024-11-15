namespace Kata {
    import Std.Math.*;

    operation ThreeStates_TwoQubits_Phases(qs : Qubit[]) : Unit {
        // Implement your solution here...

    }

    // You might find this helper operation from an earlier task useful.
    operation ThreeStates_TwoQubits(qs : Qubit[]) : Unit is Adj {
        let theta = ArcSin(1.0 / Sqrt(3.0));
        Ry(2.0 * theta, qs[0]);
        ApplyControlledOnInt(0, H, [qs[0]], qs[1]);
    }
}
