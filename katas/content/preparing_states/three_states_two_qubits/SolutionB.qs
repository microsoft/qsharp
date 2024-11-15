namespace Kata {
    import Std.Math.*;

    operation ThreeStates_TwoQubits(qs : Qubit[]) : Unit is Adj + Ctl {
        let theta = ArcSin(1.0 / Sqrt(3.0));
        Ry(2.0 * theta, qs[0]);
        ApplyControlledOnInt(0, H, [qs[0]], qs[1]);
    }
}
