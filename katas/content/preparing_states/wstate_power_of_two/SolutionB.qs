namespace Kata {
    operation WState_PowerOfTwo (qs : Qubit[]) : Unit is Adj + Ctl {
        let N = Length(qs);

        if N == 1 {
            X(qs[0]);
        } else {
            let K = N / 2;
            WState_PowerOfTwo(qs[0 .. K - 1]);

            use anc = Qubit();
            H(anc);

            for i in 0 .. K - 1 {
                Controlled SWAP([anc], (qs[i], qs[i + K]));
            }
            for i in K .. N - 1 {
                CNOT(qs[i], anc);
            }
        }
    }
}
