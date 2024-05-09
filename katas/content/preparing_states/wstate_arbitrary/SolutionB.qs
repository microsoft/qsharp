namespace Kata {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Math;

    operation WState_Arbitrary (qs : Qubit[]) : Unit is Adj + Ctl {
        let N = Length(qs);
        Ry(2.0 * ArcSin(Sqrt(1.0 / IntAsDouble(N))), qs[0]);
        if N > 1 {
            ApplyControlledOnInt(0, WState_Arbitrary, [qs[0]], qs[1 ...]);
        }
    }
}
