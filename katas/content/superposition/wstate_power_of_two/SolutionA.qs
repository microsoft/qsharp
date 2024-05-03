namespace Kata {
    operation WState_PowerOfTwo (qs : Qubit[]) : Unit is Adj + Ctl {
        let N = Length(qs);

        if N == 1 {
            X(qs[0]);
        } else {
            let K = N / 2;
            use anc = Qubit();
            H(anc);

            ApplyControlledOnInt(0, WState_PowerOfTwo, [anc], qs[0 .. K - 1]);
            ApplyControlledOnInt(1, WState_PowerOfTwo, [anc], qs[K .. N - 1]);

            for i in K .. N - 1 {
                CNOT(qs[i], anc);
            }
        }
    }
}
