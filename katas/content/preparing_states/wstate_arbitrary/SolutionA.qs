namespace Kata {
    import Std.Convert.*;
    import Std.Math.*;

    operation WState_Arbitrary(qs : Qubit[]) : Unit is Adj + Ctl {
        let N = Length(qs);
        Ry(2.0 * ArcSin(Sqrt(1.0 / IntAsDouble(N))), qs[0]);
        for i in 1..N - 1 {
            ApplyControlledOnInt(
                0,
                Ry(2.0 * ArcSin(Sqrt(1.0 / IntAsDouble(N - i))), _),
                qs[0..i - 1],
                qs[i]
            );
        }
    }
}
