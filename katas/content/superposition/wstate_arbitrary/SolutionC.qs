namespace Kata {
    operation WState_Arbitrary (qs : Qubit[]) : Unit {
        let N = Length(qs);
        if N == 1 {
            X(qs[0]);
        } else {
            mutable P = 1;
            for i in 1 .. 6 {
                if P < N {
                    set P *= 2;
                }
            }

            use anc = Qubit[P - N];
            mutable allZeros = true;
            repeat {
                WState_PowerOfTwo(qs + anc);

                set allZeros = true;
                for i in 0 .. (P - N) - 1 {
                    if MResetZ(anc[i]) == One {
                        set allZeros = false;
                    }
                }
            }
            until (allZeros);
        }
    }

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
