namespace Kata {
    import Std.Convert.*;
    import Std.Math.*;
    import Std.Measurement.*;

    operation ThreeQubitMeasurement(qs : Qubit[]) : Int {
        R1(-2.0 * PI() / 3.0, qs[1]);
        R1(-4.0 * PI() / 3.0, qs[2]);

        Adjoint WState_Arbitrary(qs);

        return MeasureInteger(qs) == 0 ? 0 | 1;
    }

    operation WState_Arbitrary(qs : Qubit[]) : Unit is Adj + Ctl {
        let N = Length(qs);

        if N == 1 {
            X(qs[0]);
        } else {
            let theta = ArcSin(1.0 / Sqrt(IntAsDouble(N)));
            Ry(2.0 * theta, qs[0]);

            X(qs[0]);
            Controlled WState_Arbitrary(qs[0..0], qs[1..N - 1]);
            X(qs[0]);
        }
    }
}
