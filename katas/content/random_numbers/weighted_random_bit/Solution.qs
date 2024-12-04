namespace Kata {
    import Std.Math.*;

    operation WeightedRandomBit(x : Double) : Int {
        let theta = 2.0 * ArcCos(Sqrt(x));  // or 2.0 * ArcSin(Sqrt(1.0 - x));

        use q = Qubit();
        Ry(theta, q);

        let result = MResetZ(q);
        return result == Zero ? 0 | 1;
    }
}
